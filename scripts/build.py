#!/usr/bin/env python3
"""Build the printer page from TOML settings and Markdown project files."""
from __future__ import annotations

import argparse
import hashlib
from html import escape
from html.parser import HTMLParser
from pathlib import Path
import re
import shutil
from string import Template
import sys
import tempfile
import tomllib
from urllib.parse import urlsplit

import markdown


class ContentError(ValueError):
    pass


def text(data: dict, key: str, path: Path) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ContentError(f"{path}: '{key}' must be a non-empty string")
    return value.strip()


def url(value: str, path: Path, key: str) -> str:
    parsed = urlsplit(value)
    if parsed.scheme not in {'https', 'http'} or not parsed.netloc or any(c.isspace() for c in value):
        raise ContentError(f"{path}: '{key}' must be a complete http:// or https:// URL")
    return value


class MarkdownLinks(HTMLParser):
    """Keep ordinary Markdown links functional without giving them action-stamp styling."""
    def __init__(self, namespace, ids):
        super().__init__(convert_charrefs=False)
        self.parts = []
        self.namespace = namespace
        self.ids = ids

    def handle_starttag(self, tag, attrs):
        attributes = dict(attrs)
        if attributes.get('id') in self.ids:
            attributes['id'] = self.namespace + '--' + attributes['id']
        href = attributes.get('href', '')
        if href.startswith('#') and href[1:] in self.ids:
            attributes['href'] = '#' + self.namespace + '--' + href[1:]
        if tag == 'a' and urlsplit(attributes.get('href', '')).scheme in {'https', 'http'}:
            attributes.update(target='_blank', rel='noopener')
        if tag == 'table':
            self.parts.append('<div class="table-scroll" role="region" aria-label="Scrollable table" tabindex="0">')
        rendered = ''.join(f' {key}="{escape(value, quote=True)}"' if value is not None else f' {key}' for key, value in attributes.items())
        self.parts.append(f'<{tag}{rendered}>')

    def handle_startendtag(self, tag, attrs):
        self.handle_starttag(tag, attrs)

    def handle_endtag(self, tag):
        self.parts.append(f'</{tag}>')
        if tag == 'table':
            self.parts.append('</div>')

    def handle_data(self, data):
        self.parts.append(data)

    def handle_entityref(self, name):
        self.parts.append(f'&{name};')

    def handle_charref(self, name):
        self.parts.append(f'&#{name};')

    def handle_comment(self, data):
        self.parts.append(f'<!--{data}-->')


class MarkdownIds(HTMLParser):
    def __init__(self):
        super().__init__()
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        identifier = dict(attrs).get('id')
        if identifier:
            self.ids.add(identifier)


def render_markdown(body: str, namespace: str = 'intro') -> str:
    html = markdown.markdown(body, extensions=[
        'abbr', 'attr_list', 'def_list', 'footnotes', 'md_in_html', 'tables', 'sane_lists', 'pymdownx.betterem', 'pymdownx.tilde',
        'pymdownx.tasklist', 'pymdownx.magiclink', 'pymdownx.superfences',
    ], extension_configs={
        'footnotes': {'BACKLINK_TEXT': '[back]'},
        'pymdownx.tilde': {'subscript': False},
        'pymdownx.superfences': {'disable_indented_code_blocks': False},
    }, output_format='html')
    ids = MarkdownIds()
    ids.feed(html)
    parser = MarkdownLinks(namespace, ids.ids)
    parser.feed(html)
    return ''.join(parser.parts)


def read_project(path: Path) -> tuple[dict, str]:
    lines = path.read_text(encoding='utf-8').splitlines()
    if not lines or lines[0].strip() != '+++':
        raise ContentError(f"{path}: start the file with +++ and a TOML header (see content/project-template.md)")
    try:
        end = next(i for i, line in enumerate(lines[1:], 1) if line.strip() == '+++')
    except StopIteration:
        raise ContentError(f"{path}: missing closing +++ for the TOML header") from None
    try:
        meta = tomllib.loads('\n'.join(lines[1:end]))
    except tomllib.TOMLDecodeError as error:
        raise ContentError(f"{path}: invalid TOML header: {error}") from error
    body = '\n'.join(lines[end + 1:]).strip()
    return meta, body


def project_html(path: Path, meta: dict, body: str, number: int) -> tuple[str, str]:
    title = text(meta, 'title', path)
    category = text(meta, 'category', path)
    if not body:
        raise ContentError(f"{path}: add a Markdown description below the closing +++")
    links = meta.get('links')
    if not isinstance(links, list) or not links:
        raise ContentError(f"{path}: add at least one [[links]] section with a label and URL")
    actions = []
    for index, link in enumerate(links, 1):
        if not isinstance(link, dict):
            raise ContentError(f"{path}: links must use [[links]] sections")
        label = text(link, 'label', path)
        href = url(text(link, 'url', path), path, f'links[{index}].url')
        actions.append(f'<a class="stamp-link" href="{escape(href, quote=True)}" target="_blank" rel="noopener">{escape(label)} <span aria-hidden="true">-&gt;</span></a>')
    slug = re.sub(r'^\d+[-_]', '', path.stem)
    if not re.fullmatch(r'[a-z0-9]+(?:-[a-z0-9]+)*', slug):
        raise ContentError(f"{path}: use a lowercase filename such as 070-my-project.md")
    entry = (
        f'<article class="paper-entry" id="{slug}">\n'
        f'<p class="entry-label">{number:03d} / {escape(category.upper())}</p>\n'
        f'<h2>{escape(title)}</h2>\n'
        f'<div class="markdown">{render_markdown(body, slug)}</div>\n'
        f'<div class="entry-links">{"".join(actions)}</div>\n'
        '</article>'
    )
    return slug, entry


def stamp_link(label: str, href: str, external: bool = False) -> str:
    attributes = ' target="_blank" rel="noopener"' if external else ''
    return f'<a class="stamp-link" href="{href}"{attributes}>{label} <span aria-hidden="true">-&gt;</span></a>'


def build(source: Path, output: Path) -> int:
    source, output = source.resolve(), output.resolve()
    if output == source or output in source.parents or source / 'content' == output or source / 'assets' == output:
        raise ContentError('Choose a separate output directory, normally _site.')
    if output.exists() and (not output.is_dir() or not (output / '.site-build').is_file()):
        raise ContentError(f'{output}: refusing to replace a directory not created by this builder')
    settings_path = source / 'content/site.toml'
    try:
        settings = tomllib.loads(settings_path.read_text(encoding='utf-8'))
    except tomllib.TOMLDecodeError as error:
        raise ContentError(f'{settings_path}: invalid TOML: {error}') from error
    keys = ['name', 'title', 'description', 'url', 'github_url', 'github_label', 'projects_heading', 'end_label', 'all_projects_label', 'all_projects_url']
    values = {key: text(settings, key, settings_path) for key in keys}
    for key in ['url', 'github_url', 'all_projects_url']:
        url(values[key], settings_path, key)
    printer = settings.get('printer')
    if not isinstance(printer, dict):
        raise ContentError(f'{settings_path}: add a [printer] section (see README.md)')
    for key in ['label', 'feed_label', 'next_label', 'next_description', 'ready', 'printing', 'end']:
        values['printer_' + key] = text(printer, key, settings_path)
    for kind, filename in [('css', 'printer.css'), ('js', 'printer.js')]:
        values[kind + '_version'] = hashlib.sha256((source / 'assets' / filename).read_bytes()).hexdigest()[:12]
    values = {key: escape(value, quote=True) for key, value in values.items()}
    values['printer_feed_label'] = values['printer_feed_label'].replace('\n', '<br>')
    values['intro_html'] = render_markdown((source / 'content/intro.md').read_text(encoding='utf-8'))
    guide_path = source / 'content/style-guide.md'
    guide_meta, guide_body = read_project(guide_path)
    guide_title = escape(text(guide_meta, 'title', guide_path))
    back_label = escape(text(guide_meta, 'back_label', guide_path))
    values['header_link_html'] = stamp_link(values['github_label'], values['github_url'], external=True)
    values['footer_links_html'] = (
        stamp_link(values['all_projects_label'], values['all_projects_url'], external=True)
        + '\n' + stamp_link(guide_title, '/style-guide.html')
    )
    projects = []
    slugs = set()
    for path in sorted((source / 'content/projects').glob('*.md')):
        if path.name.startswith('_'):
            continue
        meta, body = read_project(path)
        if 'draft' in meta and not isinstance(meta['draft'], bool):
            raise ContentError(f"{path}: 'draft' must be true or false, without quotation marks")
        if meta.get('draft', False):
            continue
        slug, html = project_html(path, meta, body, len(projects) + 1)
        if slug in slugs:
            raise ContentError(f'{path}: duplicate project name in filenames: {slug}')
        slugs.add(slug)
        projects.append(html)
    if not projects:
        raise ContentError(f'{source / "content/projects"}: add at least one non-draft .md project')
    values['projects_html'] = '\n<div class="fold-line" aria-hidden="true"></div>\n'.join(projects)
    html = Template((source / 'templates/index.html').read_text(encoding='utf-8')).substitute(values)
    samples = []
    for number, body in enumerate(guide_body.split('<!-- sample -->'), 1):
        samples.append(f'<article class="paper-entry" id="sample-{number}"><div class="markdown">'
                       + render_markdown(body.strip(), f'sample-{number}') + '</div></article>')
    guide = dict(values)
    guide.update(
        title=guide_title, name=guide_title, url=values['url'].rstrip('/') + '/style-guide.html',
        description=escape(text(guide_meta, 'description', guide_path), quote=True),
        intro_html=render_markdown(text(guide_meta, 'description', guide_path)),
        projects_heading=escape(text(guide_meta, 'heading', guide_path)),
        projects_html='\n'.join(samples),
        header_link_html=stamp_link(back_label, '/'),
        footer_links_html=stamp_link(back_label, '/'),
        end_label=escape(text(guide_meta, 'end_label', guide_path)),
        printer_next_label=escape(text(guide_meta, 'next_label', guide_path)),
        printer_next_description=escape(text(guide_meta, 'next_description', guide_path)),
    )
    guide_html = Template((source / 'templates/index.html').read_text(encoding='utf-8')).substitute(guide)
    missing = dict(values, title='Page not found', name='404 / Paper not found',
                   url=values['url'].rstrip('/') + '/404.html',
                   intro_html='<p>This page is missing from the printout.</p>',
                   projects_heading='', projects_html='',
                   header_link_html=stamp_link(back_label, '/'),
                   footer_links_html=stamp_link(guide_title, '/style-guide.html'))
    missing_html = Template((source / 'templates/index.html').read_text(encoding='utf-8')).substitute(missing)
    output.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='.printer-build-', dir=output.parent) as temporary:
        staging = Path(temporary) / 'new'
        staging.mkdir()
        (staging / 'index.html').write_text(html, encoding='utf-8')
        (staging / '.site-build').write_text('printer-site\n', encoding='utf-8')
        (staging / '.nojekyll').touch()
        (staging / 'assets').mkdir()
        (staging / 'style-guide.html').write_text(guide_html, encoding='utf-8')
        (staging / '404.html').write_text(missing_html, encoding='utf-8')
        for name in ['printer.css', 'printer.js', 'printer.svg', 'DotMatrix-Bold.woff2', 'DotMatrix-BoldItalic.woff2', 'DotMatrix-OFL.txt']:
            shutil.copy2(source / 'assets' / name, staging / 'assets' / name)
        images = source / 'content/images'
        if images.exists():
            shutil.copytree(images, staging / 'images', ignore=shutil.ignore_patterns('.*'))
        previous = Path(temporary) / 'previous'
        if output.exists():
            output.rename(previous)
        try:
            staging.rename(output)
        except OSError:
            if previous.exists():
                previous.rename(output)
            raise
    return len(projects)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    output = args.output or args.source / '_site'
    try:
        count = build(args.source, output)
    except (ContentError, OSError, KeyError) as error:
        print(f'Build failed: {error}', file=sys.stderr)
        return 1
    print(f'Built {count} projects → {output}')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
