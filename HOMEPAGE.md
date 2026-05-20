---
Document Context:
  Created: 2026-05-20
  Source: Design discussion while iterating the lyceum home page (index.html)
  Status: LIVING — first build landed 2026-05-20
  Purpose: Fix what the lyceum home page is *for* before its dedicated chrome is built, so the build proceeds from purpose rather than drift.
---

# The Lyceum Home Page

The home page (`~/lyceum/index.html`) is at a turning point. It began as a writeup rendered through the monodoc shell, and is becoming something else. This doc fixes *what it is for* before that something-else is built. WIP — directions, not commitments.

---

## Built so far

*Updated 2026-05-20 — a first pass landed the same day this doc was written.* The home page now renders through a **dedicated shell** (`kernel/index-shell.html`), no longer the monodoc writeup shell:

- **The mark** — a lambda (`Λ`): the *L* of Lyceum and a colonnade's pediment, crisp at favicon size. An inline `data:` SVG favicon, and the mark at the rail head.
- **A peripheral left rail** — the lyceum mark and wordmark stand present (the identity the dropped body-`<h1>` used to carry); a live index of the collections reveals on hover. Built from the catalogue's sections by script and scroll-spied. Styled as a library index — *not* the writeup TOC restyled, which is optimised for a single document.
- **Pinned theme** — cool dark, so the home page carries no config panel. The catalogue room is lit a touch cooler than the warm-dark reading rooms.
- **Sans throughout, no italics, no body `<h1>`** — instrument typography; identity lives in chrome.
- **A re-entry affordance** — the last writeup opened from the home page keeps a bookmark ribbon in its margin until another is opened (`localStorage`, so it survives across visits).

Still directions, not yet built: staged search, a cross-collection recency strip, `lyceum serve`.

---

## What this page does

*A thing is what it does.* The home page is the **standing view of the library** — the surface through which the whole accumulated corpus is seen, re-entered, and navigated. It is not a document. It is an instrument.

Four functions:

1. **Survey** — it makes the whole corpus visible at once; answers *what is in here.* You cannot hold fifty writeups in your head; this page is that, externalised.
2. **Re-entry** — it is the surface you return to after time away, to reorient. (Built, in part, the very week Lewis returned from two weeks abroad and needed exactly this.) Recency must be legible.
3. **Routing** — the jump-off point to any single writeup.
4. **Mirror** — it reflects the shape and growth of the thinking back at its owner. Collections, counts, accumulation: seeing the library *grow* is part of the point — the compounding artifact made visible.

## What follows from that

- **It is an instrument, not a document.** By monodoc Principle 1, instruments speak *sans*, hold a fixed scale, and may carry *present* (not peripheral) chrome. → Set the index in sans, not serif. → A visible nav is appropriate here in a way it never is inside a writeup.
- **It is not a writeup.** → "Lyceum" does not belong in the body as a pseudo-`<h1>`. Identity belongs in chrome.
- **It is always open.** → It must stay glanceable and stable, and scale from three entries to three hundred without redesign.
- **It is a re-entry point.** → Recency deserves a first-class place, not merely a sort order.
- **It is the catalogue of a private library — not an app dashboard.** The guardrail. "Nav bar, search bar, icon" is also the vocabulary of every SaaS dashboard; lyceum's home page must stay closer to *the contents page of a great book, or the card catalogue of an old library* — restrained, typographic, warm. Reach for library metaphors, not app ones.

## Directions (not yet committed)

- **Homepage chrome.** Replace the vestigial writeup-`<h1>` with real furniture. Likely a **left rail**: it absorbs the current collection table-of-contents, carries the Lyceum mark at its head, and leaves room for search. A rail is continuous with the TOC dots already in the left margin — an evolution, not a rupture — and frees the body for the full-width index.
- **The mark.** A small icon nodding to Aristotle's Lyceum. Candidates: a Greek column (iconic at favicon size; the Lyceum was a colonnaded peripatetic school), a scroll (pre-codex; echoes the scroll/codex/screen thread in the demo writeup), or a refined letterform. The favicon must survive 16px — favour the simplest. Generate options in pixery.
- **Sans throughout.** Drop serif from the index entirely — it is instrument typography.
- **Search.** A tag/title filter, staged: (a) a cheap client-side filter over already-rendered rows — viable soon, no database; (b) regex / semantic search over a SQLite metadata index with vectors, when library size warrants it. That size is the trigger LYCEUM.md reserves for adopting SQLite.
- **A recency affordance.** Flowing from the re-entry function: a marker or strip for the most recently added or updated writeups, across all collections.

## Pinning & serving

- **Favicon** — a self-contained page can carry one as an inline `data:` SVG in `<head>`; no server needed. Worth doing soon, so a pinned tab has identity.
- **Serving.** `file://` works but gives no hot reload, and `file://` links are blocked from an `http://`-served page — so any server must also serve the writeups, routing the registry's scattered paths. The proper answer is not vite but a built-in **`lyceum serve`** (already in the LYCEUM.md CLI sketch): serve `~/lyceum/`, route writeup pages from the registry, watch and live-reload. A real but bounded future build. For now: `file://` + manual refresh.

## Open questions

Resolved in the first build (2026-05-20):

- ~~Left rail vs top bar.~~ → A peripheral left rail, revealed on hover — but styled as a catalogue index, not the writeup TOC restyled.
- ~~Keep the monodoc shell, or get its own?~~ → Its own (`kernel/index-shell.html`). Chrome diverged too far to override; a separate shell is cleaner than fighting the writeup shell, and safer given the renderer's string-match splice points.
- ~~Theme control, or pin one theme?~~ → Pinned to cool dark. No config panel on the home page.

Still open:

- Whether the *relationships* between collections (not just a flat list of groups) ever become visible. Deferred — tags are the latent graph; don't draw it until the library is large enough to need it.
