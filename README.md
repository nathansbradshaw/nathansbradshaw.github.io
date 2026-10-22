# Nathan Bradshaw

A dot-matrix printout of my projects. The page is built from plain text files.

## Change the content

- `content/site.toml`: my name, profile links, page headings, and printer labels.
- `content/intro.md`: the introduction below my name.
- `content/projects/*.md`: one Markdown file per project.
- `content/images/`: photos or diagrams to use in Markdown.

### Add a project

1. Copy `content/project-template.md` into `content/projects/`.
2. Name it something like `070-my-new-project.md`.
3. Edit its title, category, links, and description.

That's it. The build finds every `.md` file in that folder automatically.
There is no project list or manifest to update, and no HTML or JavaScript to edit.

Files appear in filename order. The number prefix controls the order; the visible
project numbers are generated automatically. Use lowercase, hyphenated names.
Rename a file's number prefix to move it. Add `draft = true` in the top TOML
section to hide a project; remove that line when it's ready.

A complete project file looks like this:

```markdown
+++
title = "My new project"
category = "Electronics"

[[links]]
label = "Open website"
url = "https://example.com/"

[[links]]
label = "View source"
url = "https://github.com/nathansbradshaw/my-project"
+++

A short description. **Bold**, *italics*, lists, and inline links work here.
```

Keep the settings between the two `+++` lines. Everything below is Markdown.
Remove or add `[[links]]` sections as needed. Links need a complete `https://`
or `http://` address.

For a photo, put `my-project.jpg` in `content/images/` and write:

```markdown
![My keyboard on the bench](/images/my-project.jpg)
```

The project images use the same Markdown syntax. Optional width and height
attributes reserve space while a photo loads:

```markdown
![My keyboard on the bench](/images/my-project.jpg){ width="1200" height="600" loading="lazy" decoding="async" }
```

Images print as halftone dots in the same ink and paper colors as the text,
with slight ink spread and faint ribbon lines. The SVG filter lives in the
page template; CSS selects a finer dot pattern on small screens. Original image files stay in full color. To bypass the effect, add
`class="full-color"` to its Markdown image attributes.

The Keyboard Keyboard photo comes from the presentation's project-owned photos;
the Seahorse photo comes from its project README. The Synthphone-E photo is
the existing cutout from the presentation, preserving the original photo pixels. The K2, program builder, and
TOA images are screenshots of the actual apps. The guitar wiring photo comes from
my original Rock Band conversion write-up. All images are stored in this
repository, so the page doesn't depend on another site's image hosting.

## Preview locally

Python 3.11 or newer is required. Install the build dependencies once:

```sh
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements.txt
```

Build and serve:

```sh
.venv/bin/python scripts/build.py
.venv/bin/python -m http.server 8000 --directory _site
```

Open http://localhost:8000. After editing content, rebuild and refresh the page.
The build names the file and field if content is invalid. A failed build leaves
the last good output alone.

## Markdown styles

Supported: headings 1–6, paragraphs and hard breaks, bold and italic, strikethrough,
inline/reference/automatic links and email links, images, nested ordered and
unordered lists, static task lists, nested quotes, horizontal rules, inline and
fenced code (including fences inside lists), tables with alignment, definitions,
abbreviations, footnotes, and optional HTML details. Code prints in monochrome.
Wide code and tables scroll inside the paper. Task marks reflect the file;
visitors cannot change them. Raw HTML is trusted author content, not sanitized.

Every build includes `style-guide.html`, linked at the bottom of the homepage.
Open http://localhost:8000/style-guide.html to inspect all the samples. Edit
`content/style-guide.md` to change the examples or page labels. Next sample
feeds the paper to the next group of examples.

The site uses Python Markdown with the listed extensions; it does not implement
GitHub-specific embeds, Mermaid, or mathematical notation.

## Publish

Pushes to `main` build the content and publish `_site/` through GitHub Pages. Adding a file on
GitHub and committing it will use that same workflow. Files elsewhere on your
computer aren't visible to the website until you put them in this repository.


## Design and development

The layout is in `templates/index.html`, styling in `assets/printer.css`, and
paper-feed behavior in `assets/printer.js`. Desktop uses tractor-feed paper and a
cream dot-matrix printer; screens up to 600px use plain receipt paper and a compact
thermal printer. Both keep printed content clipped at the paper exit. Plastic and
paper textures use lightweight SVG grain layered with CSS.
The builder is `scripts/build.py`.
Content is rendered into the HTML at build time; the page remains readable if
JavaScript is unavailable. JavaScript drives the printer and Next project.

The DotMatrix fonts are bundled locally. Their SIL Open Font License is in
`assets/DotMatrix-OFL.txt`.

Run the content workflow checks with:

```sh
.venv/bin/python -m unittest discover -s tests
```
