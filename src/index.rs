//! The lyceum home page.
//!
//! A single index of every writeup, generated from the registry and
//! rendered into the dedicated index shell (`kernel/index-shell.html`).
//! Writeups are grouped into collections: an explicit `collection`
//! front-matter field when the author set one, otherwise the writeup's
//! source directory name as a fallback. A collection is an intellectual
//! family — it need not match where the files sit on disk (two projects
//! in different directories can share a collection, and often should).
//!
//! The index is a *catalogue*, not a reading surface: a wide scanning
//! surface laid out in aligned columns. `build` returns only the article
//! HTML — all presentation (the rail, the columns, the pinned theme)
//! lives in the index shell.
//!
//! Links are relative (`w/<slug>.html`) so the bundle is self-contained:
//! the same `index.html` works opened directly, served by `lyceum serve`,
//! or rsync'd to a host.

use crate::registry::Entry;
use crate::render::html_escape;
use std::path::Path;

/// Build the index article: a count line and one `<section>` per
/// collection. The shell's script builds the rail from these sections.
pub fn build(entries: &[Entry]) -> String {
    let groups = grouped(entries);

    let mut a = String::new();
    a.push_str(&format!(
        "  <p class=\"count\">{}</p>\n",
        summary(entries, &groups)
    ));

    if groups.is_empty() {
        a.push_str(
            "  <p class=\"empty\">No writeups yet. \
             Render one with <code>lyceum render</code>.</p>\n",
        );
        return a;
    }

    for (name, items) in &groups {
        a.push_str(&format!(
            "  <section class=\"collection\" id=\"col-{}\">\n",
            slug(name)
        ));
        a.push_str(&format!("    <h2>{}</h2>\n", html_escape(name)));
        for e in items {
            a.push_str(&entry_html(e));
        }
        a.push_str("  </section>\n");
    }
    a
}

/// The `created` date as a sortable key (a missing date sorts last).
fn ckey(e: &Entry) -> &str {
    e.created.as_deref().unwrap_or("")
}

/// The collection an entry belongs to: the explicit front-matter field,
/// or the source directory name as a fallback.
fn collection_of(e: &Entry) -> String {
    if let Some(c) = e.collection.as_deref() {
        if !c.is_empty() {
            return c.to_string();
        }
    }
    Path::new(&e.source)
        .parent()
        .and_then(|p| p.file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Uncategorised".to_string())
}

/// A collection name reduced to an HTML-id-safe slug, for the rail
/// anchor: runs of non-alphanumerics collapse to a single dash.
fn slug(name: &str) -> String {
    let mut s = String::new();
    let mut prev_dash = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            s.push(c.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            s.push('-');
            prev_dash = true;
        }
    }
    s.trim_matches('-').to_string()
}

/// Group entries by collection. Within a group: newest first. Groups
/// themselves: most-recently-active first, ties broken by name.
fn grouped(entries: &[Entry]) -> Vec<(String, Vec<&Entry>)> {
    let mut groups: Vec<(String, Vec<&Entry>)> = Vec::new();
    for e in entries {
        let key = collection_of(e);
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, items)) => items.push(e),
            None => groups.push((key, vec![e])),
        }
    }
    for (_, items) in &mut groups {
        items.sort_by(|a, b| ckey(b).cmp(ckey(a)));
    }
    groups.sort_by(|a, b| {
        let am = a.1.iter().map(|e| ckey(e)).max().unwrap_or("");
        let bm = b.1.iter().map(|e| ckey(e)).max().unwrap_or("");
        bm.cmp(am).then_with(|| a.0.cmp(&b.0))
    });
    groups
}

/// "N writeups" or "N writeups · M collections", singulars handled.
fn summary(entries: &[Entry], groups: &[(String, Vec<&Entry>)]) -> String {
    let n = entries.len();
    let noun = if n == 1 { "writeup" } else { "writeups" };
    if groups.len() <= 1 {
        format!("{n} {noun}")
    } else {
        format!("{n} {noun} \u{00b7} {} collections", groups.len())
    }
}

/// One writeup as a catalogue row: an `<a>` grid of
/// title | description | tags | date. Every column span is emitted even
/// when empty, so the columns stay aligned down the collection. `href`
/// is relative (`w/<slug>.html`) and `data-id` (the slug) is the stable
/// key the shell's "you were here" ribbon remembers in localStorage. A
/// `local_only` writeup carries a faint marker — it is in this local
/// catalogue but not on any synced copy.
fn entry_html(e: &Entry) -> String {
    let date = e.created.as_deref().unwrap_or("");
    let sub = e.subtitle.as_deref().unwrap_or("");
    let tags = e.tags.join(" \u{00b7} ");

    let title = if e.local_only {
        format!(
            "{}<span class=\"local-tag\">local</span>",
            html_escape(&e.title)
        )
    } else {
        html_escape(&e.title)
    };

    let mut s = String::new();
    s.push_str(&format!(
        "    <a class=\"row\" href=\"w/{}.html\" data-id=\"{}\">\n",
        html_escape(&e.slug),
        html_escape(&e.slug),
    ));
    s.push_str(&format!("      <span class=\"col-title\">{title}</span>\n"));
    s.push_str(&format!(
        "      <span class=\"col-sub\">{}</span>\n",
        html_escape(sub),
    ));
    s.push_str(&format!(
        "      <span class=\"col-tags\">{}</span>\n",
        html_escape(&tags),
    ));
    s.push_str(&format!(
        "      <span class=\"col-date\">{}</span>\n",
        html_escape(date),
    ));
    s.push_str("    </a>\n");
    s
}
