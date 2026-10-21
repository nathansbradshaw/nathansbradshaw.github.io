# Nathan Bradshaw

My GitHub landing page: https://nathansbradshaw.github.io/

Just HTML and CSS. No build step or dependencies.

## Edit

- `index.html` — intro and project links
- `assets/home.css` — layout and colors
- `assets/nb.svg` — favicon

To preview locally, run this from the repository root:

```sh
python3 -m http.server 8000
```

Then open http://localhost:8000.

## Publish

Push to `main`. The Pages workflow copies the landing page, its two assets, and
`public/404.html` into the deployment. GitHub Pages uses **GitHub Actions** as its
publishing source.

The old Dioxus seating-chart source is still in `src/`, along with its Cargo
files and assets. It's kept for reference and isn't part of the landing page.
