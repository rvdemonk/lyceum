# lyceum

A personal, LLM-maintained wiki. Markdown on disk is the source of truth;
rendered monodoc HTML is the reading surface; Claude is the primary author.

The vision, architecture, and naming are in `LYCEUM.md` (repo root).
The authoring conventions are in the `writeup` skill
(`~/.claude/skills/writeup/`).

## Status — v0

Two things work: the **renderer** and the **home-page index**.

- `lyceum render` converts a *writeup* — vanilla Markdown + YAML
  front-matter + HTML escape hatches — into a self-contained monodoc
  HTML page.
- Every render is recorded in a central **registry**, and the **home
  page** (`~/lyceum/index.html`) is regenerated from it — one index of
  every writeup, wherever its source file lives.

The rest of the wiki layer (`sources/`, ingest/query/lint, wiki-links)
is **not built yet**.

## Usage

```
lyceum render examples/sample.md         # render + refresh the home page
lyceum render writeup.md -o out.html     # explicit output path
lyceum render writeup.md --shell <path>  # non-default monodoc shell
lyceum index                             # rebuild the home page only
```

The renderer reads the monodoc HTML shell (default the bundled
`kernel/demo.html`), inlines `theme.css`, swaps in the rendered article,
and writes a single file. Output is self-contained apart from CDN fonts
and mermaid.

## The home page

`~/lyceum/index.html` lists every writeup, **grouped into collections**,
each entry linking to its rendered HTML. Open it as a browser tab and
leave it there; re-render any writeup (or run `lyceum index`) and refresh
the tab to see the update.

A *collection* is a writeup's intellectual family. It comes from the
`collection` front-matter field; writeups sharing one are grouped
together on the home page. A collection is deliberately **not** tied to
disk layout — two writeups in different project directories can, and
often should, belong to the same collection. When `collection` is
absent, the home page falls back to grouping by source directory.

Writeups stay **colocated with the projects they document** — the
registry (`~/lyceum/registry.json`) is the thread that connects them
without moving them. The registry is a derived cache: delete it and it
rebuilds as writeups are re-rendered. Entries whose source `.md` has been
deleted are dropped from the home page automatically.

## What the renderer handles

- **Front-matter**: `title`, `subtitle`, `created`, `updated`,
  `collection`, `tags`, `epigraph`, `epigraph_source`.
- **Standard Markdown**: headings, prose, lists, code, blockquotes,
  tables, images, smart punctuation.
- **Footnotes** (`[^name]`) — mapped to monodoc's sidenote primitive: an
  inline `<span class="sn">` marker plus an `<aside class="sidenote">`
  placed right after the host paragraph.
- **Fenced ```mermaid blocks** — emitted as `<div class="mermaid">`,
  HTML-escaped so mermaid.js reads the source intact.
- **Raw HTML passthrough** — custom figures, `<span class="newthought">`,
  `<p class="pull-quote">`, hand-written `<aside class="sidenote">`, and
  anything else the writeup author drops in.

## Design

The repository holds two halves:

- **`kernel/`** — *monodoc*, the typographic system: the HTML shell
  (`demo.html`), theme tokens (`theme.css`), and the design docs that
  govern them (`PRINCIPLES.md`, `RESEARCH.md`). This is what a rendered
  writeup *looks* like.
- **The renderer** (`src/`) — the Rust that turns a writeup `.md` into a
  page built on that kernel.

| File | Role |
|------|------|
| `src/main.rs` | CLI, shell assembly, render/index orchestration |
| `src/frontmatter.rs` | Small dependency-free front-matter parser |
| `src/render.rs` | Markdown -> article HTML; footnote + mermaid transforms |
| `src/registry.rs` | The writeup registry — load / upsert / save |
| `src/index.rs` | Home-page generation from the registry |

The renderer stays thin on purpose. The "vanilla MD + HTML escape
hatches" contract means most typographic primitives are just raw HTML the
author writes directly; the renderer only rewrites the two things plain
Markdown genuinely cannot express on its own.
