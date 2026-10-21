"""Check the file-editing workflow, including failed-edit recovery."""
from pathlib import Path
import shutil
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from build import ContentError, build, render_markdown, read_project


class ContentWorkflowTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name) / 'source'
        source = Path(__file__).resolve().parents[1]
        self.root.mkdir()
        for name in ['assets', 'content', 'templates']:
            shutil.copytree(source / name, self.root / name)
        self.output = Path(self.temporary.name) / 'public'

    def html(self):
        return (self.output / 'index.html').read_text()

    def test_drop_in_markdown_file_adds_project_links_markdown_and_image(self):
        self.assertEqual(build(self.root, self.output), 6)
        template = (self.root / 'content/project-template.md').read_text()
        body = template.replace('My new project', 'A new circuit').replace('Write a short description here.', 'A **working** circuit.')+ '\n\n![A circuit](/images/circuit.svg)'
        (self.root / 'content/projects/015-a-new-circuit.md').write_text(body)
        (self.root / 'content/images/circuit.svg').write_text('<svg xmlns="http://www.w3.org/2000/svg"/>')
        self.assertEqual(build(self.root, self.output), 7)
        html = self.html()
        self.assertLess(html.index('id="k2-midi-viewer"'), html.index('id="a-new-circuit"'))
        self.assertLess(html.index('id="a-new-circuit"'), html.index('id="keyboard-keyboard"'))
        self.assertIn('A <strong>working</strong> circuit.', html)
        self.assertIn('src="/images/circuit.svg"', html)
        self.assertTrue((self.output / 'images/circuit.svg').is_file())
        self.assertIn('href="https://example.com/"', html)
        self.assertNotIn('My new project', html)

    def test_bad_content_reports_filename_and_preserves_previous_build(self):
        build(self.root, self.output)
        before = (self.output / 'index.html').read_bytes()
        path = self.root / 'content/projects/070-broken-project.md'
        path.write_text('+++\ntitle = "Missing end quote\n+++\nDescription')
        with self.assertRaisesRegex(ContentError, '070-broken-project.md: invalid TOML header'):
            build(self.root, self.output)
        self.assertEqual((self.output / 'index.html').read_bytes(), before)
        path.write_text('+++\ntitle="Broken link"\ncategory="Software"\n[[links]]\nlabel="Open"\nurl="not-a-url"\n+++\nDescription')
        with self.assertRaisesRegex(ContentError, r'070-broken-project.md.*links\[1\].url'):
            build(self.root, self.output)
        self.assertEqual((self.output / 'index.html').read_bytes(), before)

    def test_draft_and_removed_files_are_not_published(self):
        path = self.root / 'content/projects/070-secret-draft.md'
        path.write_text('+++\ndraft=true\n+++\nNot ready yet')
        self.assertEqual(build(self.root, self.output), 6)
        self.assertNotIn('Not ready yet', self.html())
        (self.root / 'content/projects/060-angular-for-zed.md').unlink()
        self.assertEqual(build(self.root, self.output), 5)
        self.assertNotIn('id="angular-for-zed"', self.html())

    def test_settings_and_titles_are_escaped_while_markdown_still_formats(self):
        path = self.root / 'content/projects/070-special-characters.md'
        path.write_text('+++\ntitle="A <switch> & a \\\"quote\\\""\ncategory="Hardware & software"\n[[links]]\nlabel="Build & source"\nurl="https://example.com/?a=1&b=2"\n+++\nA *small* project.')
        settings = self.root / 'content/site.toml'
        settings.write_text(settings.read_text().replace('Nathan Bradshaw', 'Nathan & friends'))
        build(self.root, self.output)
        html = self.html()
        self.assertIn('A &lt;switch&gt; &amp; a &quot;quote&quot;', html)
        self.assertIn('Nathan &amp; friends', html)
        self.assertIn('A <em>small</em> project.', html)
        self.assertIn('href="https://example.com/?a=1&amp;b=2"', html)

    def test_markdown_specimen_is_always_built_and_linked(self):
        _, fixture = read_project(self.root / 'content/style-guide.md')
        html = render_markdown(fixture, 'first')
        for tag in ['h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'em', 'strong', 'del', 'blockquote', 'pre', 'table', 'dl', 'abbr', 'details', 'img']:
            self.assertIn('<' + tag, html)
        self.assertIn('class="table-scroll"', html)
        self.assertIn('disabled', html)
        self.assertIn('first--fn:', html)
        second = render_markdown(fixture, 'second')
        self.assertNotIn('first--fn:', second)
        self.assertIn('href="#second--fn:', second)
        build(self.root, self.output)
        self.assertTrue((self.output / 'style-guide.html').exists())
        self.assertIn('href="/style-guide.html"', self.html())
        guide = (self.output / 'style-guide.html').read_text()
        self.assertIn('Next sample', guide)
        self.assertIn('id="sample-10"', guide)
        self.assertNotIn('Electronics, keyboards, music &amp; software.', guide)
        self.assertIn('href="#sample-9--fn:bench"', guide)
        self.assertIn('id="sample-9--fn:bench"', guide)
        self.assertRegex((self.output / '404.html').read_text(), r'href="/assets/printer.css\?v=[a-f0-9]{12}"')


if __name__ == '__main__':
    unittest.main()
