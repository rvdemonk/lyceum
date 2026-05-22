//! The library bundle.
//!
//! `~/lyceum/` is a *self-contained bundle*: `index.html` plus one
//! `w/<slug>.html` per writeup, linked relatively. The same directory
//! therefore works three ways with no translation —
//!
//!   * opened directly as a `file://` page,
//!   * served over http by `lyceum serve`,
//!   * rsync'd to a host by `lyceum sync`.
//!
//! The writeup *sources* (`.md`) stay colocated with the projects they
//! document; the registry threads them. Only the rendered HTML — a
//! derived artifact — is gathered here.
//!
//! This module owns slug assignment, shell assembly, and the build
//! pipeline. `main.rs` is a thin dispatcher over it.

use crate::frontmatter;
use crate::index;
use crate::registry::{self, Entry};
use crate::render;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The bundled monodoc writeup shell.
pub fn default_shell() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/kernel/demo.html"))
}

/// The bundled home-page shell. The home page is an instrument, not a
/// writeup, so it has its own shell.
pub fn default_index_shell() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/kernel/index-shell.html"))
}

/// Reduce a title to a URL-safe slug: lower-case alphanumerics, runs of
/// anything else collapsed to a single dash.
pub fn slugify(s: &str) -> String {
    let mut out = String::new();
    let mut dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            dash = false;
        } else if !out.is_empty() && !dash {
            out.push('-');
            dash = true;
        }
    }
    let trimmed = out.trim_end_matches('-');
    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Reconcile the registry with the source files it indexes — the
/// registry is a derived cache, and this is what re-derives it:
///
///   * entries whose source `.md` has vanished are dropped;
///   * every surviving entry's metadata is refreshed from its current
///     front-matter (title, tags, `local_only`, …);
///   * every entry is given a unique slug, keeping any it already has —
///     slugs persist across title edits, so links stay durable.
///
/// The reconciled registry is saved and returned.
pub fn reconcile() -> Result<Vec<Entry>, String> {
    let mut entries = registry::load()?;
    entries.retain(|e| Path::new(&e.source).exists());

    // Refresh metadata from front-matter. The slug and source are not
    // touched: the slug is durable, the source is the entry's identity.
    for e in &mut entries {
        if let Ok(md) = fs::read_to_string(&e.source) {
            let (fm, _) = frontmatter::split(&md);
            e.title = fm.title.clone().unwrap_or_else(|| "Untitled".to_string());
            e.subtitle = fm.subtitle.clone();
            e.tags = fm.tags.clone();
            e.collection = fm.collection.clone();
            e.created = fm.created.clone();
            e.updated = fm.updated.clone();
            e.local_only = fm.local_only;
        }
    }

    let mut taken: HashSet<String> = HashSet::new();
    for e in &mut entries {
        if e.slug.is_empty() {
            e.slug = slugify(&e.title);
        }
        let base = e.slug.clone();
        let mut n = 2;
        while !taken.insert(e.slug.clone()) {
            e.slug = format!("{base}-{n}");
            n += 1;
        }
    }

    registry::save(&entries)?;
    Ok(entries)
}

/// Render one writeup `.md` into the library bundle, refresh the index,
/// and return the path of the page written. Incremental — it touches
/// only this writeup's page and the index, not the rest of the bundle.
pub fn render_single(
    input: &Path,
    writeup_shell: &Path,
    index_shell: &Path,
) -> Result<PathBuf, String> {
    let md = fs::read_to_string(input)
        .map_err(|e| format!("reading {}: {e}", input.display()))?;
    let doc = render::render_document(&md)?;
    let source = abs(input)?;

    // Preserve an existing slug for this source so re-rendering never
    // changes the page's URL.
    let prior = registry::load()?
        .into_iter()
        .find(|e| e.source == source)
        .map(|e| e.slug)
        .filter(|s| !s.is_empty())
        .unwrap_or_default();

    registry::upsert(Entry {
        title: doc.title.clone(),
        subtitle: doc.subtitle.clone(),
        tags: doc.tags.clone(),
        collection: doc.collection.clone(),
        created: doc.created.clone(),
        updated: doc.updated.clone(),
        slug: prior,
        local_only: doc.local_only,
        source: source.clone(),
    })?;

    let entries = reconcile()?;
    let entry = entries
        .iter()
        .find(|e| e.source == source)
        .ok_or("entry missing from registry after upsert")?;

    let home = registry::home_dir();
    let (wshell, wtheme) = load_shell(writeup_shell)?;
    fs::create_dir_all(home.join("w"))
        .map_err(|e| format!("creating bundle: {e}"))?;
    render_page(entry, &home, &wshell, wtheme.as_deref())?;
    write_index(&home, &entries, index_shell)?;

    Ok(entry.page_in(&home))
}

/// Re-render every registered writeup into a clean local bundle at
/// `~/lyceum/`. Returns the number of writeups built.
pub fn build_local() -> Result<usize, String> {
    let entries = reconcile()?;
    build_into(
        &registry::home_dir(),
        &entries,
        &default_shell(),
        &default_index_shell(),
    )?;
    Ok(entries.len())
}

/// Regenerate only `~/lyceum/index.html` from the registry.
pub fn reindex() -> Result<PathBuf, String> {
    let entries = reconcile()?;
    write_index(&registry::home_dir(), &entries, &default_index_shell())?;
    Ok(registry::index_path())
}

/// Build the filtered bundle that `lyceum sync` ships: every writeup
/// except the `local_only` ones, rendered into a staging directory.
/// Returns `(staging_dir, synced_count, withheld_count)`.
pub fn build_sync_staging() -> Result<(PathBuf, usize, usize), String> {
    let all = reconcile()?;
    let total = all.len();
    let synced: Vec<Entry> = all.into_iter().filter(|e| !e.local_only).collect();
    let withheld = total - synced.len();

    let dir = registry::home_dir().join(".sync");
    build_into(&dir, &synced, &default_shell(), &default_index_shell())?;
    Ok((dir, synced.len(), withheld))
}

// --- internals -------------------------------------------------------------

/// Render `entries` into a self-contained bundle at `dir`, then prune any
/// stale pages. `dir` need not be the canonical `~/lyceum/` — sync uses
/// this to build a filtered bundle in a staging directory.
fn build_into(
    dir: &Path,
    entries: &[Entry],
    writeup_shell: &Path,
    index_shell: &Path,
) -> Result<(), String> {
    let (wshell, wtheme) = load_shell(writeup_shell)?;
    let wdir = dir.join("w");
    fs::create_dir_all(&wdir).map_err(|e| format!("creating {}: {e}", wdir.display()))?;

    for e in entries {
        render_page(e, dir, &wshell, wtheme.as_deref())?;
    }
    write_index(dir, entries, index_shell)?;
    prune_pages(&wdir, entries);
    Ok(())
}

/// Render one entry's source into `dir/w/<slug>.html`.
fn render_page(
    e: &Entry,
    dir: &Path,
    writeup_shell: &str,
    theme: Option<&str>,
) -> Result<(), String> {
    let md = fs::read_to_string(&e.source)
        .map_err(|err| format!("reading {}: {err}", e.source))?;
    let doc = render::render_document(&md)?;
    let html = assemble(writeup_shell, theme, &doc.title, &doc.article_html, None)?;
    let page = e.page_in(dir);
    fs::write(&page, html).map_err(|err| format!("writing {}: {err}", page.display()))
}

/// Write `dir/index.html` from the given entries.
fn write_index(dir: &Path, entries: &[Entry], index_shell: &Path) -> Result<(), String> {
    let (shell, theme) = load_shell(index_shell)?;
    let article = index::build(entries);
    let html = assemble(&shell, theme.as_deref(), "Lyceum", &article, None)?;
    fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let path = dir.join("index.html");
    fs::write(&path, html).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// Delete `*.html` in `wdir` that no current entry claims — the residue
/// of deleted or re-slugged writeups.
fn prune_pages(wdir: &Path, entries: &[Entry]) {
    let keep: HashSet<String> = entries.iter().map(|e| format!("{}.html", e.slug)).collect();
    if let Ok(rd) = fs::read_dir(wdir) {
        for ent in rd.flatten() {
            let name = ent.file_name().to_string_lossy().into_owned();
            if name.ends_with(".html") && !keep.contains(&name) {
                let _ = fs::remove_file(ent.path());
            }
        }
    }
}

/// Read a monodoc shell and the `theme.css` that sits beside it.
fn load_shell(shell: &Path) -> Result<(String, Option<String>), String> {
    let shell_html = fs::read_to_string(shell)
        .map_err(|e| format!("reading shell {}: {e}", shell.display()))?;
    let theme_css = shell
        .parent()
        .map(|d| d.join("theme.css"))
        .and_then(|p| fs::read_to_string(p).ok());
    Ok((shell_html, theme_css))
}

/// Splice a rendered article + title into a shell, inlining `theme.css`
/// and optionally an extra stylesheet, producing one self-contained
/// file. The splice points are plain string matches — see the project
/// CLAUDE.md anti-pattern on the renderer–kernel coupling.
fn assemble(
    shell: &str,
    theme_css: Option<&str>,
    title: &str,
    article_html: &str,
    extra_css: Option<&str>,
) -> Result<String, String> {
    let mut out = shell.to_string();

    if let Some(css) = theme_css {
        let link = "<link rel=\"stylesheet\" href=\"./theme.css\">";
        if out.contains(link) {
            out = out.replace(link, &format!("<style>\n{css}\n  </style>"));
        }
    }

    if let (Some(s), Some(e)) = (out.find("<title>"), out.find("</title>")) {
        if s < e {
            let t = format!("<title>{}</title>", render::html_escape(title));
            out.replace_range(s..e + "</title>".len(), &t);
        }
    }

    if let Some(css) = extra_css {
        if let Some(pos) = out.find("</head>") {
            out.insert_str(pos, &format!("  <style>{css}  </style>\n"));
        }
    }

    let astart = out.find("<article>").ok_or("shell has no <article> element")?;
    let aend = out
        .find("</article>")
        .ok_or("shell has no </article> element")?;
    if aend < astart {
        return Err("shell <article> tags are out of order".into());
    }
    let article = format!("<article>\n{article_html}  </article>");
    out.replace_range(astart..aend + "</article>".len(), &article);

    Ok(out)
}

/// Canonical absolute path, as a string.
fn abs(p: &Path) -> Result<String, String> {
    fs::canonicalize(p)
        .map(|c| c.to_string_lossy().into_owned())
        .map_err(|e| format!("resolving {}: {e}", p.display()))
}
