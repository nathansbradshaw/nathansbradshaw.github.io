+++
title = "TOA Tools"
category = "Games + software"

[[links]]
label = "Open website"
url = "https://nathansbradshaw.github.io/toa_tools/"

[[links]]
label = "The math + source"
url = "https://github.com/nathansbradshaw/toa_tools#the-equation"
+++

A browser-based solver for the Kephri path's Lights Out puzzle in Old School RuneScape's Tombs of Amascut.

Each press flips a tile and its neighbors. I modeled those effects as a matrix and used row reduction to derive the solution. Pressing twice cancels out, so the arithmetic only needs on and off. The README shows how that calculation becomes the equations in the code.

![The TOA lights puzzle board with its illuminated tiles](/images/toa-tools.jpg){ width="420" height="420" loading="lazy" decoding="async" }
