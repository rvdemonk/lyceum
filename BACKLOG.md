---
Document Context:
  Created: 2026-05-03
  Source: Mid-session captures from monodoc demo iteration
  Status: REFERENCE
  Purpose: Hold deferred refactors and ideas surfaced during work, so the
           current task isn't derailed by premature side-quests.
---

# Backlog

Items live here when they're real but not yet worth the cost of doing.
Each entry should answer: *what's the trigger that makes this worth
revisiting?* Items without a trigger should be deleted, not parked.

---

## Refactor `#cfg-area` into separate trigger and panel

**Captured**: 2026-05-03

### What

`#cfg-area` currently does two jobs glued together: it's both the
desktop corner trigger (containing the `.cfg-handle` Aa glyph that
activates on hover) **and** the parent container of `#cfg-panel` (so
the panel inherits its `position: fixed` placement and is shown via
`#cfg-area:hover #cfg-panel`).

After adding the mobile topbar, we now have **two triggers** (corner
hover on desktop, topbar tap on mobile) but only **one panel**. The
mobile trigger reaches across the DOM via a global `html.cfg-open`
class to control a panel it doesn't structurally own. Works, but the
trigger and panel stop being intuitively co-located in the source.

The cleaner shape:

```
#cfg-handle-corner     ← desktop trigger only (just the Aa glyph)
#cfg-handle-topbar     ← mobile trigger only (lives in topbar)
#cfg-panel             ← standalone, positioned independently,
                          opened by either trigger via html.cfg-open
```

Single open/close mechanism (the global class), symmetric structure,
new triggers can be added without re-wiring.

### Trigger to revisit

A *second* toggleable chrome element appears — TOC drawer for mobile,
quick-switcher in the topbar, "compose" panel, anything that needs
the same trigger-controls-panel pattern. At that point the ad hoc
`html.cfg-open` mechanism wants to be generalised into a convention,
and `#cfg-area`'s dual responsibility wants to be split.

### Why not now

One panel and two triggers is ~30 lines of CSS and a 4-line JS
handler. Refactoring preemptively would invent a convention from a
sample size of one, which has gone badly before (cf. PRINCIPLES on
deriving from theory). Wait for the second instance.

---

## Mermaid diagrams do not recolour with the theme

**Captured**: 2026-05-20

### What

Diagrams render with a fixed dark mermaid theme. Switching the document
between warm dark, cool dark, and parchment recolours every surface
except the diagrams, which stay dark. This breaks Principle 1 (colour
crosses to both surfaces) for the diagram surface specifically — a dark
diagram on the parchment theme will look like a foreign object.

### Trigger to revisit

A published writeup containing a mermaid diagram is read in a
non-default theme — most sharply, any diagram-bearing document viewed
on parchment.

### Why not now

mermaid does not re-theme in place. The fix is to re-run `mermaid.run()`
on every theme change with per-theme `themeVariables`, having stored
each diagram's source so it can be re-rendered — the same re-render
dance the retired primer skill documented. Worth doing once diagrams
are common, not before.

---

## Decide the fate of ~/claude-resources/primer-template/

**Captured**: 2026-05-20

### What

The retired `primer` skill referenced `~/claude-resources/primer-template/`
(template HTML, palette presets, mermaid gotchas). The new `writeup`
skill does not. The mermaid wisdom from its README has now been ported
into the monodoc shell and lyceum's renderer, so the directory is fully
orphaned.

### Trigger to revisit

A cleanup pass of `~/claude-resources/`, or a moment when lyceum's
renderer needs figure/diagram templates worth mining from it.

### Why not now

Dormant cost is zero. Recommended disposition: keep as an archive for
one more cycle in case a template idea is worth lifting, then delete.
Not worth a decision today.

---

## Renderer v0 — known limitations

**Captured**: 2026-05-20

### What

Sharp edges in lyceum's first renderer, each harmless until a real
writeup hits it:

- **Multi-paragraph footnotes** have their `<p>` tags stripped, so the
  paragraphs jam together inside the `<aside>`. Single-paragraph
  footnotes — the norm — are fine.
- **A footnote referenced outside a paragraph** (inside a list item or
  heading) has its aside flushed at the end of the document rather than
  after the host block.
- **No `<section>` wrapping** — content sits directly under `<article>`.
  Typography holds via adjacent-sibling selectors; the only loss is
  `section`-scoped CSS hooks.

### Trigger to revisit

Any one of these is hit by a genuine writeup and the output looks
wrong. Fix that specific one then.

### Why not now

Each is an edge case. The renderer handles the common writeup shape
correctly (verified against `examples/sample.md`). Solving all four
pre-emptively would be the theory-first move PRINCIPLES warns against.
