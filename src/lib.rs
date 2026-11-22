use anyhow::{Context, Result, bail};
use ego_tree::NodeRef;
use regex::{Captures, Regex};
use scraper::{Html, Node, Selector};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Component, Path, PathBuf},
};
use toml::Value;

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

fn text<'a>(data: &'a Value, key: &str, path: &Path) -> Result<&'a str> {
    data.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .with_context(|| format!("{}: '{key}' must be a non-empty string", path.display()))
}

fn url(value: &str, path: &Path, key: &str) -> Result<()> {
    let valid = url::Url::parse(value)
        .ok()
        .is_some_and(|u| matches!(u.scheme(), "http" | "https") && u.host_str().is_some());
    if !valid || value.chars().any(char::is_whitespace) {
        bail!(
            "{}: '{key}' must be a complete http:// or https:// URL",
            path.display()
        );
    }
    Ok(())
}

pub fn read_project(path: &Path) -> Result<(Value, String)> {
    let input = fs::read_to_string(path).with_context(|| path.display().to_string())?;
    let lines: Vec<_> = input.lines().collect();
    if lines.first().map(|s| s.trim()) != Some("+++") {
        bail!(
            "{}: start the file with +++ and a TOML header (see content/project-template.md)",
            path.display()
        );
    }
    let end = (1..lines.len())
        .find(|&i| lines[i].trim() == "+++")
        .with_context(|| {
            format!(
                "{}: missing closing +++ for the TOML header",
                path.display()
            )
        })?;
    let meta = toml::from_str::<Value>(&lines[1..end].join("\n"))
        .with_context(|| format!("{}: invalid TOML header", path.display()))?;
    Ok((meta, lines[end + 1..].join("\n").trim().to_string()))
}

fn markdown_options() -> comrak::Options<'static> {
    let mut options = comrak::Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.render.unsafe_ = true;
    options
}

fn replace_outside_code(
    body: &str,
    pattern: &Regex,
    options: &comrak::Options<'_>,
    mut replace: impl FnMut(&Captures<'_>) -> String,
) -> String {
    let arena = comrak::Arena::new();
    let root = comrak::parse_document(&arena, body, options);
    let mut offsets = vec![0];
    offsets.extend(body.match_indices('\n').map(|(i, _)| i + 1));
    let mut protected = Vec::new();
    for node in root.descendants() {
        let data = node.data.borrow();
        if matches!(
            data.value,
            comrak::nodes::NodeValue::CodeBlock(_) | comrak::nodes::NodeValue::Code(_)
        ) {
            let pos = data.sourcepos;
            let start = offsets
                .get(pos.start.line.saturating_sub(1))
                .copied()
                .unwrap_or(body.len());
            let end = offsets
                .get(pos.end.line.saturating_sub(1))
                .copied()
                .unwrap_or(body.len());
            // Protect the full lines of block code, and the precise inline-code span.
            if matches!(data.value, comrak::nodes::NodeValue::CodeBlock(_)) {
                protected.push(start..offsets.get(pos.end.line).copied().unwrap_or(body.len()));
            } else {
                protected
                    .push((start + pos.start.column.saturating_sub(1))..(end + pos.end.column));
            }
        }
    }
    pattern
        .replace_all(body, |c: &Captures<'_>| {
            let matched = c.get(0).unwrap();
            if protected
                .iter()
                .any(|r| matched.start() < r.end && matched.end() > r.start)
            {
                matched.as_str().to_string()
            } else {
                replace(c)
            }
        })
        .into_owned()
}

// Preserve the image attributes and abbreviation syntax used by existing content.
fn prepare_markdown(
    body: &str,
    options: &comrak::Options<'_>,
) -> (String, BTreeMap<String, String>) {
    let definition = Regex::new(r"(?m)^\*\[([^\]]+)\]:\s*(.+)\r?$").unwrap();
    let mut abbreviations = BTreeMap::new();
    let body = replace_outside_code(body, &definition, options, |c: &Captures<'_>| {
        abbreviations.insert(c[1].to_string(), c[2].trim().to_string());
        String::new()
    });
    let image = Regex::new(r"(!\[[^\]\n]*\]\([^\n]+?\))\{([^}\n]*)\}").unwrap();
    let attr =
        Regex::new(r#"([\w-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s]+))|([.#])([\w-]+)"#).unwrap();
    let body = replace_outside_code(&body, &image, options, |c: &Captures<'_>| {
        let rendered = comrak::markdown_to_html(&c[1], options);
        let mut attributes = BTreeMap::new();
        for a in attr.captures_iter(&c[2]) {
            if let Some(name) = a.get(1) {
                let value = a
                    .get(2)
                    .or_else(|| a.get(3))
                    .or_else(|| a.get(4))
                    .unwrap()
                    .as_str();
                attributes.insert(name.as_str().to_string(), value.to_string());
            } else if &a[5] == "#" {
                attributes.insert("id".into(), a[6].to_string());
            } else {
                let classes = attributes
                    .entry("class".to_string())
                    .or_insert_with(String::new);
                if !classes.is_empty() {
                    classes.push(' ');
                }
                classes.push_str(&a[6]);
            }
        }
        let attrs: String = attributes
            .iter()
            .map(|(k, v)| format!(" {k}=\"{}\"", escape(v)))
            .collect();
        rendered
            .trim()
            .strip_prefix("<p>")
            .unwrap_or(rendered.trim())
            .strip_suffix("</p>")
            .unwrap_or(rendered.trim())
            .replace("<img ", &format!("<img{attrs} "))
    });
    (body, abbreviations)
}

fn footnote_id(id: &str) -> String {
    if let Some(rest) = id.strip_prefix("fnref-") {
        format!("fnref:{rest}")
    } else if let Some(rest) = id.strip_prefix("fn-") {
        format!("fn:{rest}")
    } else {
        id.to_string()
    }
}

struct MarkdownRenderer<'a> {
    namespace: &'a str,
    ids: HashSet<String>,
    abbreviations: BTreeMap<String, String>,
}

impl MarkdownRenderer<'_> {
    fn render(&self, node: NodeRef<'_, Node>, literal: bool, raw: bool, out: &mut String) {
        match node.value() {
            Node::Text(value) => {
                if raw {
                    out.push_str(value);
                } else if literal || self.abbreviations.is_empty() {
                    out.push_str(&escape(value));
                } else {
                    let pattern = format!(
                        r"\b(?:{})\b",
                        self.abbreviations
                            .keys()
                            .map(|k| regex::escape(k))
                            .collect::<Vec<_>>()
                            .join("|")
                    );
                    let regex = Regex::new(&pattern).unwrap();
                    let mut start = 0;
                    for m in regex.find_iter(value) {
                        out.push_str(&escape(&value[start..m.start()]));
                        out.push_str(&format!(
                            "<abbr title=\"{}\">{}</abbr>",
                            escape(&self.abbreviations[m.as_str()]),
                            escape(m.as_str())
                        ));
                        start = m.end();
                    }
                    out.push_str(&escape(&value[start..]));
                }
            }
            Node::Comment(value) => out.push_str(&format!("<!--{}-->", value.comment)),
            Node::Element(element) => {
                let tag = element.name();
                let printed_photo = tag == "img"
                    && !element
                        .attr("class")
                        .unwrap_or("")
                        .split_whitespace()
                        .any(|class| class == "full-color");
                if tag == "html" {
                    for child in node.children() {
                        self.render(child, literal, raw, out);
                    }
                    return;
                }
                if tag == "table" {
                    out.push_str("<div class=\"table-scroll\" role=\"region\" aria-label=\"Scrollable table\" tabindex=\"0\">");
                }
                let mut attrs: BTreeMap<String, String> = element
                    .attrs()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                attrs.remove("markdown");
                if matches!(tag, "th" | "td") {
                    if let Some(alignment) = attrs.remove("align") {
                        attrs.insert("style".into(), format!("text-align: {alignment};"));
                    }
                }
                if let Some(id) = attrs.get_mut("id") {
                    *id = format!("{}--{}", self.namespace, footnote_id(id));
                }
                if let Some(href) = attrs.get_mut("href") {
                    if let Some(id) = href.strip_prefix('#') {
                        if self.ids.contains(id) {
                            *href = format!("#{}--{}", self.namespace, footnote_id(id));
                        }
                    }
                }
                if tag == "a"
                    && attrs
                        .get("href")
                        .and_then(|v| url::Url::parse(v).ok())
                        .is_some_and(|u| matches!(u.scheme(), "https" | "http"))
                {
                    attrs.insert("target".into(), "_blank".into());
                    attrs.insert("rel".into(), "noopener".into());
                }
                if tag == "section" && attrs.contains_key("data-footnotes") {
                    attrs.insert("class".into(), "footnote".into());
                }
                if printed_photo {
                    out.push_str("<span class=\"print-image\">");
                }
                out.push_str(&format!("<{tag}"));
                for (key, value) in attrs {
                    out.push_str(&format!(" {key}=\"{}\"", escape(&value)));
                }
                out.push('>');
                if tag == "a" && element.attr("data-footnote-backref").is_some() {
                    out.push_str("[back]");
                } else {
                    for child in node.children() {
                        self.render(
                            child,
                            literal || matches!(tag, "pre" | "code" | "abbr" | "script" | "style"),
                            matches!(tag, "script" | "style"),
                            out,
                        );
                    }
                }
                if !matches!(
                    tag,
                    "area"
                        | "base"
                        | "br"
                        | "col"
                        | "embed"
                        | "hr"
                        | "img"
                        | "input"
                        | "link"
                        | "meta"
                        | "param"
                        | "source"
                        | "track"
                        | "wbr"
                ) {
                    out.push_str(&format!("</{tag}>"));
                }
                if tag == "table" {
                    out.push_str("</div>");
                }
                if printed_photo {
                    out.push_str("</span>");
                }
            }
            _ => {
                for child in node.children() {
                    self.render(child, literal, raw, out);
                }
            }
        }
    }
}

pub fn render_markdown(body: &str, namespace: &str) -> String {
    let options = markdown_options();
    let (body, abbreviations) = prepare_markdown(body, &options);
    let html = Html::parse_fragment(&comrak::markdown_to_html(&body, &options));
    let ids = html
        .select(&Selector::parse("[id]").unwrap())
        .filter_map(|e| e.value().attr("id").map(str::to_string))
        .collect();
    let renderer = MarkdownRenderer {
        namespace,
        ids,
        abbreviations,
    };
    let mut output = String::new();
    renderer.render(html.tree.root(), false, false, &mut output);
    output
}

fn stamp_link(label: &str, href: &str, external: bool) -> String {
    let attrs = if external {
        " target=\"_blank\" rel=\"noopener\""
    } else {
        ""
    };
    format!(
        "<a class=\"stamp-link\" href=\"{href}\"{attrs}>{label} <span aria-hidden=\"true\">-&gt;</span></a>"
    )
}

fn project_html(path: &Path, meta: &Value, body: &str, number: usize) -> Result<(String, String)> {
    let title = text(meta, "title", path)?;
    let category = text(meta, "category", path)?;
    if body.is_empty() {
        bail!(
            "{}: add a Markdown description below the closing +++",
            path.display()
        );
    }
    let links = meta
        .get("links")
        .and_then(Value::as_array)
        .filter(|v| !v.is_empty())
        .with_context(|| {
            format!(
                "{}: add at least one [[links]] section with a label and URL",
                path.display()
            )
        })?;
    let mut actions = String::new();
    for (i, link) in links.iter().enumerate() {
        let label = text(link, "label", path)?;
        let href = text(link, "url", path)?;
        url(href, path, &format!("links[{}].url", i + 1))?;
        actions.push_str(&stamp_link(&escape(label), &escape(href), true));
    }
    let stem = path
        .file_stem()
        .and_then(|v| v.to_str())
        .context("Invalid project filename")?;
    let slug = Regex::new(r"^\d+[-_]")
        .unwrap()
        .replace(stem, "")
        .to_string();
    if !Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
        .unwrap()
        .is_match(&slug)
    {
        bail!(
            "{}: use a lowercase filename such as 070-my-project.md",
            path.display()
        );
    }
    let html = format!(
        "<article class=\"paper-entry\" id=\"{slug}\">\n<p class=\"entry-label\">{number:03} / {}</p>\n<h2>{}</h2>\n<div class=\"markdown\">{}</div>\n<div class=\"entry-links\">{actions}</div>\n</article>",
        escape(&category.to_uppercase()),
        escape(title),
        render_markdown(body, &slug)
    );
    Ok((slug, html))
}

fn substitute(template: &str, values: &BTreeMap<String, String>) -> Result<String> {
    let pattern =
        Regex::new(r"\$\$|\$\{([a-zA-Z_][a-zA-Z0-9_]*)\}|\$([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    let mut error = None;
    let result = pattern.replace_all(template, |c: &Captures<'_>| {
        if &c[0] == "$$" {
            return "$".to_string();
        }
        let key = c.get(1).or_else(|| c.get(2)).unwrap().as_str();
        values.get(key).cloned().unwrap_or_else(|| {
            error = Some(key.to_string());
            String::new()
        })
    });
    if let Some(key) = error {
        bail!("Template refers to unknown setting: {key}");
    }
    Ok(result.into_owned())
}

fn copy_tree(source: &Path, output: &Path) -> Result<()> {
    fs::create_dir(output)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let dest = output.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_tree(&entry.path(), &dest)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

fn resolved(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return Ok(fs::canonicalize(path)?);
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for part in absolute.components() {
        match part {
            Component::ParentDir => {
                normalized.pop();
            }
            Component::CurDir => (),
            _ => normalized.push(part.as_os_str()),
        }
    }
    if let Some(parent) = normalized.parent().filter(|p| *p != normalized) {
        return Ok(
            resolved(parent)?.join(normalized.file_name().context("Missing output filename")?)
        );
    }
    Ok(normalized)
}

pub fn build(source: &Path, output: &Path) -> Result<usize> {
    let source = fs::canonicalize(source)?;
    let output = resolved(output)?;
    if source.starts_with(&output)
        || ["content", "assets", "templates", "src", "tests", ".git"]
            .iter()
            .any(|name| output.starts_with(source.join(name)))
    {
        bail!("Choose a separate output directory, normally _site.");
    }
    if output.exists() && (!output.is_dir() || !output.join(".site-build").is_file()) {
        bail!(
            "{}: refusing to replace a directory not created by this builder",
            output.display()
        );
    }
    let settings_path = source.join("content/site.toml");
    let settings = toml::from_str::<Value>(&fs::read_to_string(&settings_path)?)
        .with_context(|| format!("{}: invalid TOML", settings_path.display()))?;
    let mut values = BTreeMap::new();
    for key in [
        "name",
        "title",
        "description",
        "url",
        "website_url",
        "website_label",
        "github_url",
        "github_label",
        "projects_heading",
        "end_label",
        "all_projects_label",
        "all_projects_url",
    ] {
        let value = text(&settings, key, &settings_path)?;
        if matches!(
            key,
            "url" | "website_url" | "github_url" | "all_projects_url"
        ) {
            url(value, &settings_path, key)?;
        }
        values.insert(key.to_string(), escape(value));
    }
    let printer = settings
        .get("printer")
        .filter(|v| v.is_table())
        .with_context(|| {
            format!(
                "{}: add a [printer] section (see README.md)",
                settings_path.display()
            )
        })?;
    for key in [
        "label",
        "receipt_label",
        "next_label",
        "next_description",
        "ready",
        "printing",
        "end",
    ] {
        values.insert(
            format!("printer_{key}"),
            escape(text(printer, key, &settings_path)?),
        );
    }
    for (kind, filename) in [
        ("css", "printer.css"),
        ("js", "printer.js"),
        ("impact_photo", "printer-impact-compact.webp"),
        ("receipt_photo", "printer-receipt-compact.webp"),
        ("head_photo", "printer-impact-head.webp"),
    ] {
        let digest = format!(
            "{:x}",
            Sha256::digest(fs::read(source.join("assets").join(filename))?)
        );
        values.insert(format!("{kind}_version"), digest[..12].to_string());
    }
    values.insert(
        "intro_html".into(),
        render_markdown(
            &fs::read_to_string(source.join("content/intro.md"))?,
            "intro",
        ),
    );
    let guide_path = source.join("content/style-guide.md");
    let (guide_meta, guide_body) = read_project(&guide_path)?;
    let guide_title = escape(text(&guide_meta, "title", &guide_path)?);
    let back_label = escape(text(&guide_meta, "back_label", &guide_path)?);
    values.insert(
        "header_link_html".into(),
        format!(
            "{}\n{}",
            stamp_link(&values["website_label"], &values["website_url"], true),
            stamp_link(&values["github_label"], &values["github_url"], true)
        ),
    );
    values.insert(
        "footer_links_html".into(),
        format!(
            "{}\n{}",
            stamp_link(
                &values["all_projects_label"],
                &values["all_projects_url"],
                true
            ),
            stamp_link(&guide_title, "/style-guide.html", false)
        ),
    );
    let mut paths = fs::read_dir(source.join("content/projects"))?
        .map(|e| e.map(|e| e.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut projects = Vec::new();
    let mut slugs = HashSet::new();
    for path in paths.into_iter().filter(|p| {
        p.extension().is_some_and(|v| v == "md")
            && !p.file_name().unwrap().to_string_lossy().starts_with('_')
    }) {
        let (meta, body) = read_project(&path)?;
        if let Some(draft) = meta.get("draft") {
            if !draft.is_bool() {
                bail!(
                    "{}: 'draft' must be true or false, without quotation marks",
                    path.display()
                );
            }
            if draft.as_bool() == Some(true) {
                continue;
            }
        }
        let (slug, html) = project_html(&path, &meta, &body, projects.len() + 1)?;
        if !slugs.insert(slug.clone()) {
            bail!(
                "{}: duplicate project name in filenames: {slug}",
                path.display()
            );
        }
        projects.push(html);
    }
    if projects.is_empty() {
        bail!(
            "{}: add at least one non-draft .md project",
            source.join("content/projects").display()
        );
    }
    values.insert(
        "projects_html".into(),
        projects.join("\n<div class=\"fold-line\" aria-hidden=\"true\"></div>\n"),
    );
    let template = fs::read_to_string(source.join("templates/index.html"))?;
    let html = substitute(&template, &values)?;
    let mut guide = values.clone();
    guide.insert("title".into(), guide_title.clone());
    guide.insert("name".into(), guide_title.clone());
    guide.insert(
        "url".into(),
        format!("{}/style-guide.html", values["url"].trim_end_matches('/')),
    );
    guide.insert(
        "intro_html".into(),
        render_markdown(text(&guide_meta, "description", &guide_path)?, "intro"),
    );
    guide.insert("projects_html".into(), guide_body.split("<!-- sample -->").enumerate().map(|(i, body)| format!("<article class=\"paper-entry\" id=\"sample-{}\"><div class=\"markdown\">{}</div></article>", i + 1, render_markdown(body.trim(), &format!("sample-{}", i + 1)))).collect::<Vec<_>>().join("\n"));
    for (key, meta_key) in [
        ("description", "description"),
        ("projects_heading", "heading"),
        ("end_label", "end_label"),
        ("printer_next_label", "next_label"),
        ("printer_next_description", "next_description"),
    ] {
        guide.insert(
            key.into(),
            escape(text(&guide_meta, meta_key, &guide_path)?),
        );
    }
    guide.insert(
        "header_link_html".into(),
        stamp_link(&back_label, "/", false),
    );
    guide.insert(
        "footer_links_html".into(),
        stamp_link(&back_label, "/", false),
    );
    let guide_html = substitute(&template, &guide)?;
    let mut missing = values.clone();
    for (key, value) in [
        ("title", "Page not found"),
        ("name", "404 / Paper not found"),
        (
            "intro_html",
            "<p>This page is missing from the printout.</p>",
        ),
        ("projects_heading", ""),
        ("projects_html", ""),
    ] {
        missing.insert(key.into(), value.into());
    }
    missing.insert(
        "url".into(),
        format!("{}/404.html", values["url"].trim_end_matches('/')),
    );
    missing.insert(
        "header_link_html".into(),
        stamp_link(&back_label, "/", false),
    );
    missing.insert(
        "footer_links_html".into(),
        stamp_link(&guide_title, "/style-guide.html", false),
    );
    let missing_html = substitute(&template, &missing)?;
    let parent = output.parent().context("Output directory needs a parent")?;
    fs::create_dir_all(parent)?;
    let temporary = tempfile::Builder::new()
        .prefix(".printer-build-")
        .tempdir_in(parent)?;
    let staging = temporary.path().join("new");
    fs::create_dir(&staging)?;
    for (name, content) in [
        ("index.html", html.as_str()),
        ("style-guide.html", &guide_html),
        ("404.html", &missing_html),
        (".site-build", "printer-site\n"),
        (".nojekyll", ""),
    ] {
        fs::write(staging.join(name), content)?;
    }
    fs::create_dir(staging.join("assets"))?;
    for name in [
        "printer.css",
        "printer.js",
        "printer.svg",
        "printer-impact-compact.webp",
        "printer-receipt-compact.webp",
        "printer-impact-head.webp",
        "DotMatrix-Bold.woff2",
        "DotMatrix-BoldItalic.woff2",
        "DotMatrix-OFL.txt",
        "fine-dots.svg",
        "print-edge.svg",
    ] {
        fs::copy(
            source.join("assets").join(name),
            staging.join("assets").join(name),
        )?;
    }
    let images = source.join("content/images");
    if images.exists() {
        copy_tree(&images, &staging.join("images"))?;
    }
    let previous = temporary.path().join("previous");
    if output.exists() {
        fs::rename(&output, &previous)?;
    }
    if let Err(error) = fs::rename(&staging, &output) {
        if previous.exists() {
            fs::rename(&previous, &output).context("Could not restore previous build")?;
        }
        return Err(error.into());
    }
    Ok(projects.len())
}
