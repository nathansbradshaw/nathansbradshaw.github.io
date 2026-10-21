# Nathan Bradshaw

My GitHub landing page: https://nathansbradshaw.github.io/

A Victorian calling card for my projects: burgundy damask, textured paper,
gilt borders, and engraved ornaments. Just HTML, CSS, and local assets.
No build step or JavaScript.

## Edit

- `index.html` — intro and project links
- `assets/home.css` — layout and colors
- `assets/nb.svg` — favicon
- `assets/ornament.svg`, `corner.svg`, `damask.svg`, `grain.svg` — editable ornaments and textures

The heading font is Cormorant Garamond, bundled locally under the SIL Open Font
License. Its license is in `assets/Cormorant-Garamond-OFL.txt`.

To preview locally, run this from the repository root:

```sh
python3 -m http.server 8000
```

Then open http://localhost:8000.

## Publish

Push to `main`. The Pages workflow copies the landing page, its assets, and
`public/404.html` into the deployment. GitHub Pages uses **GitHub Actions** as its
publishing source.

The old Dioxus seating-chart source is still in `src/`, along with its Cargo
files and assets. It's kept for reference and isn't part of the landing page.
