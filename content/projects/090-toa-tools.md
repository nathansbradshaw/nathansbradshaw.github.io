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

Each press flips a tile and its neighbors. I wrote those effects as a matrix and used row reduction to work out which tiles need pressing. Because a second press cancels the first, the arithmetic only needs two values: on and off.

The Phaser app lets you set up a board and see the solution change as you edit it. The README follows the calculation from the initial matrix to the equations in the code.

![The TOA lights puzzle board with its illuminated tiles](/images/toa-tools.jpg){ width="420" height="420" loading="lazy" decoding="async" }
