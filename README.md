# lyceum

A personal, LLM-maintained wiki. Markdown on disk is the source of truth;
rendered monodoc HTML is the reading surface; Claude is the primary author.

The vision, architecture, and naming are in `LYCEUM.md` (repo root).
The authoring conventions are in the `writeup` skill
(`~/.claude/skills/writeup/`).

## Status — v0

The **renderer**, the **home-page index**, and the **library bundle**
work — render, build, serve, and sync.

- `lyceum render` converts a *writeup* — vanilla Markdown + YAML
  front-matter + HTML escape hatches — into a self-contained monodoc
  HTML page.
- Every render is recorded in a central **registry**, and the **home
  page** is regenerated from it — one index of every writeup, wherever
  its source file lives.
- The output is a **self-contained bundle** at `~/lyceum/` that can be
  opened directly, served locally with live-reload, or synced to a host.

The rest of the wiki layer (`sources/`, ingest/query/lint, wiki-links)
is **not built yet**.

## Usage

```
lyceum render writeup.md          # render one writeup into the bundle, refresh the index
lyceum index                      # regenerate the home page from the registry
lyceum build                      # re-render every registered writeup into a clean bundle
lyceum serve                      # serve the bundle locally with live-reload
lyceum sync                       # rsync the bundle (minus local_only writeups) to a host
lyceum ls                         # list registered writeups
lyceum read <slug>                # print a writeup's source MD (or HTML with --rendered)
lyceum grep <pattern>             # case-insensitive substring search across sources
```

`render` accepts `--shell <path>` for a non-default monodoc shell.
`serve` accepts `--port` and `--host` (use `--host 0.0.0.0` to reach it
from a phone on the same network). `sync` accepts `--target` and
`--dry-run`. `ls` and `grep` both accept `--collection` and `--tag`
filters; `ls` also takes `--long` and `--json`.

## Reading the library

`ls`, `read`, and `grep` are the **read access** to the library — what
the renderer writes, these verbs read back. They exist so Claude (or
any shell) can ask the library *what is in it* without crawling the
project directories the source `.md` files live in: the slug is the
indirection, the registry resolves it to a path.

`read` returns the source Markdown by default, not the rendered HTML.
The MD is denser and free of the kernel's instrument scaffolding — the
right level for re-reading what's already in the library. `--rendered`
is reserved for the rare case where the HTML *is* what's wanted
(debugging the renderer, auditing how a primitive output).

```
lyceum ls --tag growth --long             # all growth writeups, full metadata
lyceum read the-disclosure-tax            # the source MD
lyceum grep "creator-founder"             # where has this phrase appeared before
lyceum grep endo --collection "Growth Strategy"
```

The renderer reads the monodoc HTML shell (the bundled `kernel/demo.html`
for writeups, `kernel/index-shell.html` for the home page), inlines
`theme.css`, swaps in the rendered article, and writes one self-contained
file — self-contained apart from CDN fonts and mermaid.

## The bundle

`~/lyceum/` is a **self-contained bundle**: `index.html` plus one
`w/<slug>.html` per writeup, linked relatively. The same directory works
three ways with no translation — opened as a `file://` page, served by
`lyceum serve`, or rsync'd to a host. The writeup *sources* (`.md`) stay
colocated with the projects they document; only the rendered HTML — a
derived artifact — is gathered into the bundle.

- **`lyceum serve`** is a *local* tool: an http server with live-reload,
  watching the kernel and every writeup source. A host serving the
  library needs no lyceum process — the bundle is just static files.
- **`lyceum sync`** builds a *filtered* bundle (every writeup except the
  `local_only` ones) into a staging directory and rsyncs it. The rsync
  destination lives in `~/lyceum/.sync-target` — deployment config, not
  code.
- A writeup with `local_only: true` in its front-matter stays in the
  local bundle but is never synced — it does not leave the machine.

## The home page

`~/lyceum/index.html` lists every writeup, **grouped into collections**,
each entry linking to its rendered page (`w/<slug>.html`). Open it as a
browser tab and leave it there; under `lyceum serve` it live-reloads on
every change.

The home page is an *instrument*, not a writeup — a standing view of the
library — so it has its own shell (`kernel/index-shell.html`): sans
throughout, pinned to one theme, with a peripheral left rail carrying the
lyceum mark and a live index of the collections. It also remembers the
last writeup opened from it and marks that row on return. The design
rationale is in `HOMEPAGE.md`.

A *collection* is a writeup's intellectual family. It comes from the
`collection` front-matter field; writeups sharing one are grouped
together on the home page. A collection is deliberately **not** tied to
disk layout — two writeups in different project directories can, and
often should, belong to the same collection. When `collection` is
absent, the home page falls back to grouping by source directory.

Writeup *sources* stay **colocated with the projects they document** —
the registry (`~/lyceum/registry.json`) is the thread that connects them
without moving them. The registry is a derived cache: every build
reconciles it against the source files — metadata is refreshed from
front-matter, entries whose source `.md` has vanished are dropped, and
slugs are assigned once and kept so links stay durable. Delete the
registry entirely and it rebuilds from the next render or build.

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

- **`kernel/`** — *monodoc*, the typographic system: the writeup shell
  (`demo.html`), the dedicated home-page shell (`index-shell.html`),
  theme tokens (`theme.css`), and the design docs that govern them
  (`PRINCIPLES.md`, `RESEARCH.md`). This is what a rendered writeup
  *looks* like.
- **The renderer** (`src/`) — the Rust that turns a writeup `.md` into a
  page built on that kernel.

| File | Role |
|------|------|
| `src/main.rs` | CLI — a thin dispatcher over the modules below |
| `src/bundle.rs` | Slugs, shell assembly, the build pipeline |
| `src/frontmatter.rs` | Small dependency-free front-matter parser |
| `src/render.rs` | Markdown -> article HTML; footnote + mermaid transforms |
| `src/registry.rs` | The writeup registry — load / upsert / save |
| `src/index.rs` | Home-page generation from the registry |
| `src/serve.rs` | Local dev server — http + live-reload + file-watch |
| `src/sync.rs` | Filtered staging build + rsync to a host |
| `src/library.rs` | Read access — `ls`, `read`, `grep` over the registry |

The renderer stays thin on purpose. The "vanilla MD + HTML escape
hatches" contract means most typographic primitives are just raw HTML the
author writes directly; the renderer only rewrites the two things plain
Markdown genuinely cannot express on its own.
