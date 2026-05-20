---
title: One Repository
subtitle: Why monodoc and lyceum became a single tree
created: 2026-05-20
collection: Tooling
tags: [lyceum, architecture, git]
---

<span class="newthought">Two repositories</span> is the kind of decision that feels principled and turns out to be overhead. monodoc — the typographic kernel — and lyceum — the tool built on it — were always going to be developed by one person, in lockstep, with the renderer reading the kernel's shell file directly from disk.[^coupling] A repository boundary buys independent versioning, independent release, separate access control. None of those applied here.

So the two became one. The kernel now lives at `kernel/` inside the lyceum repository; the renderer sits alongside it. The conceptual separation — kernel versus tool — survives intact, carried by a directory boundary rather than a repository one. The same distinction, drawn in the right place, at a fraction of the cost.

The consolidation paid an unexpected dividend. The renderer's shell path was an absolute hardcode into a sibling directory; it is now resolved relative to the crate itself. A wart removed by a move — the kind of thing that happens when two pieces of one program are finally allowed to sit in one tree.

This document has a second job. It is the first writeup rendered after the consolidation — a smoke test, proof that the pipeline still runs end to end:

- front-matter parsed, including the `collection` that files it under *Tooling*;
- prose rendered through the monodoc shell, now loaded from `kernel/`;
- a footnote lifted into the margin as a sidenote;
- the home-page index regenerated to include this very page.

If you are reading this as a designed page, all four held.

[^coupling]: A runtime file read of a hardcoded sibling path is not a versioned dependency. It is two halves of one program that happen to sit in different folders — which is the surest sign they belong in the same tree.
