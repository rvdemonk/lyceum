---
Document Context:
  Created: 2026-05-01
  Source: Project directory orientation for the kernel/ directory
  Status: REFERENCE
  Purpose: Brief orientation for what this directory is and how its parts relate
---

# Monodoc

A workshop for a single-file HTML document type designed for considered written work: knowledge bases, formalised essays and papers, pedagogical readings with linked sources, and published HTML content. The aim is a consistent, restrained visual system optimised for the reader's sustained attention.

The name is provisional. *Imprint* was a near-pick — typographic resonance, captures the act of pressing material into form — but didn't reach the wiki use case. *Monodoc* is the literal description we're using until something more apt emerges. It may also turn out that wiki-mode warrants a separate tool that nevertheless inherits this project's readability standards; that's an open question.

The skill that currently produces these documents lives at `~/.claude/skills/primer/` and will be renamed in lockstep once the project's name stabilises. This directory is where the design system is *developed*: the working demo, the principles distilled from iteration, the research notes. The skill is downstream of the work that happens here.

---

## Files

- `demo.html` — the working demo. Source of truth for what a monodoc currently looks like. Open in a browser to see the system in action.
- `theme.css` — colour tokens, hoisted from `demo.html` so themes can be added and tuned without touching the demo's typography or layout.
- `PRINCIPLES.md` — codified design principles. Each principle was articulated only after we'd seen its alternative fail. Add to this as new ones emerge.
- `RESEARCH.md` — evidence and findings the principles draw on, with source-confidence flags.
- `BACKLOG.md` — deferred refactors and ideas surfaced mid-session that aren't worth doing yet. Each entry must name the trigger that would make it worth revisiting; entries without a trigger should be deleted, not parked.
- `LYCEUM.md` — vision and architecture for the umbrella tool that will eventually absorb monodoc as its rendering kernel. Read this for the bigger frame the kernel work is feeding into.
- `README.md` — this file.

## Workflow

1. Iterate on `demo.html` to test new patterns, primitives, or interaction ideas.
2. When a pattern stabilises and we've seen *why* it's right, write it up in `PRINCIPLES.md`.
3. Eventually, codify the stable primitives into the skill so the skill produces documents that reflect the current best understanding.

The split keeps experimental work out of the skill. The skill should encode only what has earned its place.

## Use cases

- **Knowledge bases.** Durable, navigable references that grow over time.
- **Essays and papers.** Formalising conversations and research into considered written output.
- **Pedagogical readings.** Claude produces a reading with linked sources; we read and discuss.
- **Published HTML.** Content served from a web server — the file is self-contained, no asset pipeline.

Across all four, the constant is **readability optimisation**. That's the real product: a document form that makes sustained reading frictionless. Whether the content is literary, technical, or wiki-style, it inherits the same readability contract.
