# lyceum

An LLM-maintained personal wiki. Writeups — vanilla Markdown — render
through the *monodoc* typographic kernel into self-contained HTML pages
and accumulate into one navigable library. Claude is the primary author;
the rendered HTML is the reading surface. The aim is a knowledge base
that *compounds* — written once, kept, re-entered — not notes that
scatter and rot.

One repo, two halves: `kernel/` is monodoc, the typographic system; the
Rust crate at the root is the renderer. The renderer assembles writeups
into a self-contained bundle at `~/lyceum/` — one directory that opens
directly as `file://`, serves locally over http, or syncs to a host.
Orientation is in `README.md`; the vision and architecture in
`LYCEUM.md`; the home-page design in `HOMEPAGE.md`; deferred work in
`BACKLOG.md`. The kernel's design commitments live in
`kernel/PRINCIPLES.md`, its evidence base in `kernel/RESEARCH.md`.

## Anti-patterns

- **Don't break the renderer–kernel coupling silently.** `assemble()` in
  `src/bundle.rs` locates its splice points by plain string match —
  `<article>`, `</article>`, `<title>`, and the `theme.css` `<link>` —
  in *both* shells it renders into: `kernel/demo.html` (writeups) and
  `kernel/index-shell.html` (the home page). Alter, remove, or duplicate
  any of those markers — even inside an HTML comment — and rendering
  breaks with no error raised. Editing a shell means checking what the
  renderer greps for.

- **Don't watch the registry from the serve watcher.** `lyceum serve`
  rebuilds on file changes; a rebuild rewrites `~/lyceum/registry.json`;
  watching that file would loop forever. `serve.rs scan()` deliberately
  omits the registry — the watcher re-reads the *source list* from it
  each pass, so writeups added elsewhere still enter the watch set
  without the file itself being in it. Making the watcher "exhaustive"
  breaks this invariant silently.

- **Don't grow lyceum a hosting backend.** The library is static — a
  host serves the bundle as plain files behind a gate; no `lyceum`
  process ever runs there. Auth, sessions, or APIs *inside lyceum* would
  saddle a Markdown tool with a web server and security model. The gate
  is nginx config in front of the bundle, generic infrastructure
  reusable for any private subdomain. The host stays static; lyceum
  stays a renderer.

- **Don't add a principle without having seen the alternative fail.**
  Principles articulated from theory misfire in ways no one notices until
  much later. Try the wrong thing first; codify only after the scar
  tissue. `kernel/PRINCIPLES.md` is a living document — exceptions are
  legitimate, recorded openly, not assumed away.

- **Don't cite a source without verification.** One attribution in
  `kernel/RESEARCH.md` is already flagged unverified ("Eric Lawson / 66
  books"). Mark uncertain findings explicitly rather than asserting them.

- **Don't regenerate the `writeup` skill piecemeal.** The skill at
  `~/.claude/skills/writeup/` is downstream of the renderer and the
  kernel's conventions, and lives outside this repo. Update it
  deliberately when those stabilise — not ad hoc as they shift.
