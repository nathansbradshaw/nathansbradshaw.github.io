+++
title = "Markdown styles"
description = "A test print of every supported Markdown layout. Scroll through the paper or use Next sample to inspect each one."
heading = "TYPE & LAYOUT SPECIMENS"
back_label = "Back to projects"
next_label = "Next sample ↓"
next_description = "Scroll paper to the next Markdown sample"
end_label = "END OF TEST PRINT"
+++

# Headings & printed type

## Heading two

### Heading three

#### Heading four

##### Heading five

###### Heading six

Plain ink, **extra impression**, *italic printing*, ***both together***, and ~~an old idea~~. An escaped \*asterisk\*, an en dash – and an ampersand & stay readable.

A paragraph can have a deliberate break.  
This line starts directly below it.

---

<!-- sample -->

## Links

An [inline link](https://github.com/nathansbradshaw), a [reference link][profile], a bare URL https://github.com/nathansbradshaw and an email <hello@example.com>. The blue stamps above and below the samples use the same style as project actions.

[profile]: https://github.com/nathansbradshaw "Nathan on GitHub"

<!-- sample -->

## Lists & checklists

- A circuit board
- A keyboard
    - Switches
    - A controller
        - USB connection
- A finished enclosure

1. Draw the circuit.
2. Test it on the bench.
    1. Check power.
    2. Check the signal.
3. Write it up.

- [x] Sketch the idea
- [x] Build a prototype
- [ ] Write up the next revision

Checklists are printed status marks, edited in the Markdown file.

<!-- sample -->

## Quotes

> Keep a notebook next to the soldering iron.
>
> - Measure first.
> - Write down what changed.
>
> > A note inside a note.

<!-- sample -->

## Code & keys

Inline `digitalWrite(LED_PIN, HIGH)` and <kbd>Ctrl</kbd> + <kbd>S</kbd>.

```cpp
const int LED_PIN = 13;
void setup() {
    pinMode(LED_PIN, OUTPUT);
    digitalWrite(LED_PIN, HIGH);
}
// A deliberately long line checks that code scrolls inside the paper rather than widening the entire page.
```

1. A code block inside a list:

    ```text
    USB -> controller -> switches
    ```

    Indented follow-up text.

<!-- sample -->

## Tables

| Part | Purpose | Quantity |
| :--- | :---: | ---: |
| Controller | Reads the switches | 1 |
| Diode | Keeps each key independent | 34 |
| Connector | USB power and data | 1 |

On a narrow screen, wide tables scroll within the paper.

<!-- sample -->

## Images

![A simple circuit diagram with a supply, resistor and LED](/images/markdown-circuit.svg "A circuit diagram")

*An image caption can be an italic paragraph.*

<!-- sample -->

## Definitions & abbreviations

PCB
:   A board that connects electronic components.

USB
:   A connection for power and data.

The PCB connects to USB.

*[PCB]: Printed circuit board
*[USB]: Universal Serial Bus

<!-- sample -->

## Footnotes

A project can include a small aside[^bench] without interrupting its description. Another reference to the same note[^bench].



[^bench]: Footnotes keep their own links even when multiple project files use the same name.

<!-- sample -->

## Optional details

<details markdown="1">
<summary>Read the bench notes</summary>

These notes expand on demand. **Markdown** also works inside them.

- Tested the prototype.
- Wrote down the next change.

</details>
