use printer_site::{build, read_project, render_markdown};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct Fixture {
    _temporary: tempfile::TempDir,
    root: PathBuf,
    output: PathBuf,
}

fn copy(source: &Path, output: &Path) {
    fs::create_dir(output).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy(&entry.path(), &output.join(entry.file_name()));
        } else {
            fs::copy(entry.path(), output.join(entry.file_name())).unwrap();
        }
    }
}

impl Fixture {
    fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("source");
        fs::create_dir(&root).unwrap();
        for name in ["assets", "content", "templates"] {
            copy(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join(name),
                &root.join(name),
            );
        }
        for entry in fs::read_dir(root.join("content/projects")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|v| v == "md") {
                fs::remove_file(path).unwrap();
            }
        }
        let output = temporary.path().join("public");
        let fixture = Self {
            _temporary: temporary,
            root,
            output,
        };
        for name in [
            "010-first-project",
            "030-third-project",
            "050-removable-project",
        ] {
            fixture.project(name, "+++\ntitle=\"A project\"\ncategory=\"Software\"\n[[links]]\nlabel=\"Open\"\nurl=\"https://example.com/\"\n+++\nA description.");
        }
        fixture
    }
    fn project(&self, name: &str, content: &str) {
        fs::write(
            self.root.join(format!("content/projects/{name}.md")),
            content,
        )
        .unwrap();
    }
    fn html(&self) -> String {
        fs::read_to_string(self.output.join("index.html")).unwrap()
    }
    fn build(&self) -> usize {
        build(&self.root, &self.output).unwrap()
    }
}

#[test]
fn added_project_preserves_order_links_markdown_and_images() {
    let fixture = Fixture::new();
    assert_eq!(fixture.build(), 3);
    let template = fs::read_to_string(fixture.root.join("content/project-template.md")).unwrap();
    let body = template
        .replace("My new project", "A new circuit")
        .replace("Write a short description here.", "A **working** circuit.")
        + "\n\n![A circuit](/images/circuit.svg)";
    fixture.project("015-a-new-circuit", &body);
    fs::write(
        fixture.root.join("content/images/circuit.svg"),
        "<svg xmlns=\"http://www.w3.org/2000/svg\"/>",
    )
    .unwrap();
    assert_eq!(fixture.build(), 4);
    let html = fixture.html();
    assert!(
        html.find("id=\"first-project\"").unwrap() < html.find("id=\"a-new-circuit\"").unwrap()
    );
    assert!(
        html.find("id=\"a-new-circuit\"").unwrap() < html.find("id=\"third-project\"").unwrap()
    );
    for expected in [
        "A <strong>working</strong> circuit.",
        "src=\"/images/circuit.svg\"",
        "href=\"https://example.com/\"",
    ] {
        assert!(html.contains(expected), "{expected}");
    }
    assert!(fixture.output.join("images/circuit.svg").is_file());
    assert!(!html.contains("My new project"));
}

#[test]
fn bad_content_reports_filename_and_preserves_previous_build() {
    let fixture = Fixture::new();
    fixture.build();
    let before = fixture.html();
    for (content, expected) in [
        (
            "+++\ntitle = \"Missing end quote\n+++\nDescription",
            "070-broken-project.md: invalid TOML header",
        ),
        (
            "+++\ntitle=\"Broken link\"\ncategory=\"Software\"\n[[links]]\nlabel=\"Open\"\nurl=\"not-a-url\"\n+++\nDescription",
            "links[1].url",
        ),
    ] {
        fixture.project("070-broken-project", content);
        let error = format!("{:#}", build(&fixture.root, &fixture.output).unwrap_err());
        assert!(error.contains(expected), "{error}");
        assert_eq!(fixture.html(), before);
    }
}

#[test]
fn drafts_and_removed_files_are_not_published() {
    let fixture = Fixture::new();
    fixture.project("070-secret-draft", "+++\ndraft=true\n+++\nNot ready yet");
    assert_eq!(fixture.build(), 3);
    assert!(!fixture.html().contains("Not ready yet"));
    fs::remove_file(
        fixture
            .root
            .join("content/projects/050-removable-project.md"),
    )
    .unwrap();
    assert_eq!(fixture.build(), 2);
    assert!(!fixture.html().contains("id=\"removable-project\""));
}

#[test]
fn metadata_is_escaped_while_markdown_still_formats() {
    let fixture = Fixture::new();
    fixture.project("070-special-characters", "+++\ntitle='A <switch> & a \"quote\"'\ncategory=\"Hardware & software\"\n[[links]]\nlabel=\"Build & source\"\nurl=\"https://example.com/?a=1&b=2\"\n+++\nA *small* project.");
    let settings = fixture.root.join("content/site.toml");
    fs::write(
        &settings,
        fs::read_to_string(&settings)
            .unwrap()
            .replace("Nathan Bradshaw", "Nathan & friends"),
    )
    .unwrap();
    fixture.build();
    let html = fixture.html();
    for expected in [
        "A &lt;switch&gt; &amp; a &quot;quote&quot;",
        "Nathan &amp; friends",
        "A <em>small</em> project.",
        "href=\"https://example.com/?a=1&amp;b=2\"",
    ] {
        assert!(html.contains(expected), "{expected}");
    }
}

#[test]
fn markdown_specimen_and_namespaced_footnotes_are_preserved() {
    let fixture = Fixture::new();
    let (_, body) = read_project(&fixture.root.join("content/style-guide.md")).unwrap();
    let html = render_markdown(&body, "first");
    for tag in [
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "em",
        "strong",
        "del",
        "blockquote",
        "pre",
        "table",
        "dl",
        "abbr",
        "details",
        "img",
    ] {
        assert!(html.contains(&format!("<{tag}")), "Missing {tag}");
    }
    assert!(html.contains("class=\"table-scroll\""));
    assert!(html.contains("disabled"));
    assert!(html.contains("first--fn:"));
    let second = render_markdown(&body, "second");
    assert!(!second.contains("first--fn:"));
    assert!(second.contains("href=\"#second--fn:"));
    fixture.build();
    assert!(fixture.output.join("style-guide.html").is_file());
    assert!(fixture.html().contains("href=\"/style-guide.html\""));
    let guide = fs::read_to_string(fixture.output.join("style-guide.html")).unwrap();
    for expected in [
        "Next sample",
        "id=\"sample-10\"",
        "href=\"#sample-9--fn:bench\"",
        "id=\"sample-9--fn:bench\"",
    ] {
        assert!(guide.contains(expected), "{expected}");
    }
    assert!(!guide.contains("Electronics, keyboards, music &amp; software."));
    let missing = fs::read_to_string(fixture.output.join("404.html")).unwrap();
    assert!(
        regex::Regex::new(r#"href="/assets/printer.css\?v=[a-f0-9]{12}""#)
            .unwrap()
            .is_match(&missing)
    );
}

#[test]
fn image_attributes_and_details_markdown_survive_migration() {
    let html = render_markdown(
        "![Phone](/images/phone.png){ width=\"500\" height=\"349\" loading=\"lazy\" decoding=\"async\" .full-color }\n\n<details markdown=\"1\">\n<summary>Notes</summary>\n\n**Bold notes**\n\n</details>",
        "project",
    );
    for expected in [
        "width=\"500\"",
        "height=\"349\"",
        "loading=\"lazy\"",
        "decoding=\"async\"",
        "class=\"full-color\"",
        "<strong>Bold notes</strong>",
    ] {
        assert!(html.contains(expected), "{expected}\n{html}");
    }
}

#[test]
fn attributes_and_abbreviations_leave_code_examples_literal() {
    let input = "```markdown\n![Phone](/phone.png){ width=\"500\" }\n*[PCB]: Printed circuit board\n```\n\n`![Phone](/phone.png){ width=\"500\" }`\n\nPCB\n\n*[PCB]: Printed circuit board";
    let html = render_markdown(input, "example");
    assert!(
        html.contains("![Phone](/phone.png){ width=&quot;500&quot; }"),
        "{html}"
    );
    assert!(html.contains("*[PCB]: Printed circuit board"), "{html}");
    assert!(!html.contains("<img"), "{html}");
    assert!(
        html.contains("<abbr title=\"Printed circuit board\">PCB</abbr>"),
        "{html}"
    );
    let html = render_markdown("| A | B |\n| :- | -: |\n| a | b |", "table");
    assert!(html.contains("style=\"text-align: right;\""), "{html}");
}

#[test]
fn output_refuses_source_and_unowned_directories() {
    let fixture = Fixture::new();
    for output in [
        &fixture.root,
        &fixture.root.join("content/images"),
        &fixture.root.join("templates/new-output"),
    ] {
        assert!(build(&fixture.root, output).is_err());
    }
    fs::create_dir(&fixture.output).unwrap();
    fs::write(fixture.output.join("keep.txt"), "Unrelated work").unwrap();
    assert!(
        build(&fixture.root, &fixture.output)
            .unwrap_err()
            .to_string()
            .contains("refusing to replace")
    );
    assert_eq!(
        fs::read_to_string(fixture.output.join("keep.txt")).unwrap(),
        "Unrelated work"
    );
}
