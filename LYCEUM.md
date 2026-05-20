---
Document Context:
  Created: 2026-05-06
  Source: Multi-session iteration on monodoc; reframed after reading Karpathy's LLM Wiki gist
  Status: DRAFT PLAN
  Purpose: Capture the vision, name, and architectural decisions for lyceum — the LLM-driven personal wiki tool that absorbs monodoc as its rendering kernel
---

# Lyceum

A personal, LLM-maintained wiki built on monodoc's typographic kernel. Source of truth is markdown on the filesystem; consumption is rendered HTML in a browser; authorship is mostly Claude. The pattern is Karpathy's *LLM Wiki*; the visual system is monodoc's.

The name is a deliberate step away from *monodoc* (literally "one document"), which is wrong for an umbrella tool. *Lyceum* is the place Aristotle established for organised collective learning — a *site of study*, not a great-man label. The wiki is a place; *lyceum* names it.

---

## The synthesis

Two threads converge here:

- **Karpathy's *LLM Wiki*** ([gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f)) provides the *organisational discipline*: a markdown wiki maintained continuously by an LLM rather than re-derived per query, organised into raw sources / wiki / schema layers, with ingest / query / lint operations. The pattern only becomes viable now, with model quality where it is — humans abandon wikis under bookkeeping fatigue; LLMs don't get bored.
- **Monodoc** provides the *rendering aesthetic*: typographic primitives (sidenotes, semantic zoom, themes, the readability contract from `PRINCIPLES.md`) that turn each wiki page into a designed artifact rather than a Markdown dump. Karpathy uses Obsidian as the visual layer because it's good enough for utility consumption; lyceum holds a higher bar.

The two pieces are complementary, not competitive. Lyceum is *Karpathy's wiki pattern + monodoc's kernel + a custom MD pipeline + browser-native consumption*.

---

## Use cases

The cases lyceum is designed for, all *personal*:

- Reports, writeups, summaries — Claude-generated, accumulated
- Boutique investigations — multi-session research where the wiki compounds
- Market research, due diligence
- Literary essays, reading companions
- Personal development plans, study plans
- General knowledge accumulation — what Karpathy calls a "personal LLM-driven library"

The constant: *long-running, accumulating, organised by the LLM, consumed by the human*.

---

## Architecture

### Three layers (Karpathy)

- **Sources** — immutable raw material the LLM reads. Articles, PDFs, transcripts, data files. Lives in a `sources/` directory; lyceum reads, never modifies.
- **Wiki** — LLM-owned markdown files. Summaries, entity pages, concept pages, comparisons, indices. Lives in a `wiki/` directory; the LLM creates, updates, cross-references, and maintains.
- **Schema** — a configuration document (likely `LYCEUM.md` or `AGENTS.md` at the wiki root) that tells Claude how the wiki is structured, what conventions hold, what workflows to follow. Co-evolves with use.

### Source format: vanilla MD + HTML escape hatches

**Decision (provisional)**: lyceum entries are written in vanilla Markdown, with frontmatter for metadata and HTML escape hatches for primitives MD can't natively express. *Not* a custom MD dialect.

The argument against custom syntax: every project that's invented its own MD extensions (MDX, Obsidian's `:::callout`, AsciiDoc detours) pays in editor support that doesn't quite work, parsing edge cases, and lock-in. The output stops being plain MD in any meaningful sense.

The things lyceum needs to express, and how vanilla MD covers them:

- **Footnotes / sidenotes** — standard MD `[^1]` syntax; the renderer maps them to monodoc's sidenote primitive.
- **Images with placement** — standard `![alt](src)`; for sidebar placement, wrap in `<figure class="sidenote">…</figure>`.
- **Custom HTML figures** — already HTML; inline as needed.
- **Pull quotes, epigraphs, semantic-zoom blocks** — small set of class-name conventions on HTML escapes.
- **Wiki links** — Obsidian-style `[[entity]]` syntax (a single concession, well-supported by parsers, doesn't break vanilla MD compatibility).
- **Frontmatter** — YAML for tags, dates, source refs, page type.

The line we hold: *vanilla MD + frontmatter + a small set of class-name conventions for HTML escapes*. Not a parallel syntax. If a primitive emerges that genuinely can't be expressed cleanly through this, justify the dialect extension at that point. Don't pre-invent.

### Storage: filesystem, no DB at first

Karpathy's piece runs without SQLite at moderate scale (~100 sources, hundreds of pages). His tools: an `index.md` (content-oriented catalogue of all pages, regenerated per ingest) and a `log.md` (chronological append-only, parseable with `grep`). At lyceum's likely scale, this is enough.

**Decision**: defer SQLite. Build with filesystem + index.md + log.md. Add a derived index DB only when grep-over-filesystem demonstrably fails to answer something. (Probably never, given Karpathy's reports.)

Counter-argument worth keeping live: an index DB unlocks structured queries (all entries tagged X, all entries linking to Y, all entries created in date range Z). If those queries become routine, the DB earns its keep. Until then, frontmatter + grep covers it.

### Consumption: browser-as-GUI

**Decision**: the rendered HTML *is* the interface. No Obsidian dependency.

This has consequences:
1. The typographic kernel does double duty — reading surface and primary interface. Every minute spent on monodoc pays off because it *is* the GUI.
2. **Publish-by-default** becomes free. Every wiki entry is a static HTML file deployable to the droplet. Personal use and public use are the same artifact, gated by access control (subdomain, basic auth, or just selectively-published subdirectories). Drafts stay local; finished material can ship to `lyceum.3rigby.xyz` (or similar) without translation.
3. No graph view by default — replaced with TOC + backlinks block on each page, plus the index page.

### Authorship: Claude-primary, vim as escape hatch

Most entries are Claude-generated. The browser is the consumption surface; vim is the editor of last resort when Claude needs human correction or you want to tweak something directly. The dev pattern from monodoc (vim → save → vite-style HMR in browser) carries over.

**Skipped**: a browser-native editor (CodeMirror/Monaco). Adds parallel state to manage and an in-browser tooling layer that competes with the simplicity of file-on-disk. If editing-in-browser becomes a felt need later, revisit.

### Operations (Karpathy)

- **Ingest** — Claude reads a new source, discusses takeaways, writes a summary page, updates the index, updates relevant entity/concept pages across the wiki, appends a `log.md` entry. A single source might touch 10–15 wiki pages.
- **Query** — Claude searches the wiki (not raw sources), reads relevant pages, synthesises an answer with citations. Good answers get filed back as new pages so explorations compound rather than disappearing into chat history.
- **Lint** — periodic health-check: contradictions between pages, stale claims newer sources have superseded, orphan pages, important concepts mentioned without their own page, missing cross-references, data gaps that warrant new sources.

---

## The kernel relationship

monodoc — the typographic kernel — and lyceum — the tool — live in **one repository**. The kernel sits at `kernel/`; the Rust renderer alongside it. (An earlier plan kept them as two separate repos; that was consolidated on 2026-05-20 — two repos was overhead for one solo, lockstep-developed project. The reasoning is recorded in `examples/consolidation.md`.)

The kernel/tool distinction is real and preserved — it is now a *directory* boundary, not a *repository* one:

```
~/tools/lyceum/         # the one repo (rvdemonk/lyceum)
  kernel/               # monodoc — demo.html, theme.css, PRINCIPLES, RESEARCH
  src/                  # the Rust renderer
  examples/             # example writeups
  LYCEUM.md  HOMEPAGE.md  BACKLOG.md  README.md
~/lyceum/               # generated home page + registry — the library, kept
                        # separate from the tooling so it backs up on its own
```

monodoc keeps its name as the name of the kernel; `kernel/` is simply where it sits.

---

## Pipeline

**Likely language**: Rust.

Per global `CLAUDE.md`: *"the 'fast development speed' of prototyping with weaker languages like python, js and tsx is becoming hard to defend; tsx and rust are the same number of tokens, and by skipping straight to a prototype in rust we are addressing the real obstacles faster."* The pipeline is a long-lived dependency for a personal library — Rust pays off here. Likely crates: `pulldown-cmark` (vanilla MD parsing with HTML passthrough), `gray_matter` (YAML frontmatter), `tera` or hand-rolled templating, `clap` for CLI.

**Sketch of the CLI surface** (sufficient for v0):

```
lyceum new "Title"            scaffold wiki/<slug>.md with frontmatter
lyceum ls [--tag X]           list entries
lyceum ingest <source>        ingest a raw source (dispatches to Claude)
lyceum search "query"         filesystem search; FTS only when grep fails
lyceum link a b               add a wiki-link from a to b
lyceum build                  render everything to out/
lyceum serve                  dev server with HMR
lyceum publish [<slug>]       deploy selected entries to the droplet
```

The CLI is Claude-facing primarily (per Lewis's note). Human use is occasional and via the browser.

---

## Replacing the primer skill

The skill at `~/.claude/skills/primer/` currently produces single-file HTML primers. Once lyceum is operational, this skill is replaced by a *lyceum skill* — Claude's operating manual for maintaining a lyceum installation. The new skill encodes:

- Directory conventions (`sources/`, `wiki/`, `index.md`, `log.md`)
- MD authoring conventions (frontmatter shape, link syntax, escape-hatch class names)
- Ingest / query / lint workflows
- Templates for common entry types (source summary, entity page, concept page, comparison)
- The schema document's role and how to update it

A standalone primer (single document, no wiki context) becomes a degenerate case the lyceum skill can still handle — same renderer, just no cross-references. The primer skill doesn't need to coexist.

**Phasing**: the new skill can ship before the lyceum tool is complete. Phase 1 — Claude writes entries in the lyceum format into a `wiki/` directory; user reads them as MD. Phase 2 — lyceum's renderer lands and the same entries render as primers. The skill matures alongside the tool.

---

## Open questions

- **Wiki content directory location** — `~/lyceum/` (clean, top-level) vs `~/Documents/lyceum/` (default macOS) vs `~/notes/lyceum/`. Probably `~/lyceum/`. Defer until the tool exists.
- **Schema document name** — `LYCEUM.md` (consistent with `CLAUDE.md`) or `AGENTS.md` (Codex/cross-LLM convention). Probably `LYCEUM.md` for our own clarity; can also drop a same-content `AGENTS.md` symlink for portability.
- **Frontmatter shape** — exact fields TBD with first ingests. Likely starts: `title`, `tags`, `created`, `updated`, `sources`, `links`. Iterate from use.
- **Theme defaults for published pages** — same warm-dark default as the demo, or auto light/dark by user preference? Both are solvable; defer.
- **Backlink generation** — at build time (renderer scans all entries for `[[wiki-links]]` and emits backlink blocks) or at ingest time (Claude maintains explicit backlink lists). Build-time is more durable; ingest-time is more visible. Probably build-time.
- **Search at scale** — whether `qmd` (Karpathy mentions it: BM25 + vector + LLM re-ranking, local) is worth integrating, or whether `ripgrep` over MD is sufficient. Probably `rg` until proven inadequate.

---

## What stays the same in monodoc

Active development of the typographic kernel continues in the `kernel/` directory, under the provisional name *monodoc*. `PRINCIPLES.md` remains the design source of truth; `RESEARCH.md` carries evidence; `BACKLOG.md` parks deferred work. The current iteration loop (vite + theme.css + demo.html) stays as the experimental surface.

When the kernel reaches a stable v0 — signalled by the PRINCIPLES list stabilising, themes sealed, mobile resolved, the on-demand notes pattern proven across realistic content lengths — the kernel becomes a dependency and lyceum-the-tool becomes its own project.
