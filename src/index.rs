//! The lyceum home page.
//!
//! A single index of every writeup, generated from the registry. Writeups
//! are grouped into collections: an explicit `collection` front-matter
//! field when the author set one, otherwise the writeup's source directory
//! name as a fallback. A collection is an intellectual family — it need not
//! match where the files sit on disk (two projects in different directories
//! can share a collection, and often should).
//!
//! The index is a *catalogue*, not a reading surface. It deliberately
//! breaks monodoc's 55% prose measure and lays each writeup out as a wide
//! row — title, description, tags, date in aligned columns — for a
//! higher-bandwidth scan.
//!
//! Open `~/lyceum/index.html` as a browser tab; re-render any writeup (or
//! run `lyceum index`) to refresh it.

use crate::registry::Entry;
use crate::render::html_escape;
use std::path::Path;

/// Build the index page: returns (article-html, index-specific CSS).
pub fn build(entries: &[Entry]) -> (String, String) {
    let groups = grouped(entries);

    let mut article = String::new();
    article.push_str("  <h1>Lyceum</h1>\n");
    article.push_str(&format!(
        "  <p class=\"subtitle\">{}</p>\n\n",
        summary(entries, &groups)
    ));

    if groups.is_empty() {
        article
            .push_str("  <p>No writeups yet. Render one with <code>lyceum render</code>.</p>\n");
    } else {
        // Collection headings are real <h2>s so the margin table-of-contents
        // lists the collections for free.
        article.push_str("  <div class=\"lyceum-index\">\n");
        for (name, items) in &groups {
            article.push_str(&format!("    <h2>{}</h2>\n", html_escape(name)));
            for e in items {
                article.push_str(&entry_html(e));
            }
        }
        article.push_str("  </div>\n");
    }

    (article, INDEX_CSS.to_string())
}

/// The `created` date as a sortable key (a missing date sorts last).
fn ckey(e: &Entry) -> &str {
    e.created.as_deref().unwrap_or("")
}

/// The collection an entry belongs to: the explicit front-matter field, or
/// the source directory name as a fallback.
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

/// One writeup as a table row: an <a> grid of title | description | tags |
/// date. Every column span is emitted even when empty, so the columns stay
/// aligned down the collection.
fn entry_html(e: &Entry) -> String {
    let date = e.created.as_deref().unwrap_or("");
    let sub = e.subtitle.as_deref().unwrap_or("");
    let tags = e.tags.join(" \u{00b7} ");

    let mut s = String::new();
    s.push_str(&format!(
        "    <a class=\"lyceum-row\" href=\"{}\">\n",
        html_escape(&file_url(&e.output)),
    ));
    s.push_str(&format!(
        "      <span class=\"lyceum-col-title\">{}</span>\n",
        html_escape(&e.title),
    ));
    s.push_str(&format!(
        "      <span class=\"lyceum-col-sub\">{}</span>\n",
        html_escape(sub),
    ));
    s.push_str(&format!(
        "      <span class=\"lyceum-col-tags\">{}</span>\n",
        html_escape(&tags),
    ));
    s.push_str(&format!(
        "      <span class=\"lyceum-col-date\">{}</span>\n",
        html_escape(date),
    ));
    s.push_str("    </a>\n");
    s
}

/// Turn an absolute filesystem path into a `file://` URL. Spaces are the
/// one character common enough in paths to be worth encoding.
fn file_url(path: &str) -> String {
    let encoded = path.replace(' ', "%20");
    if encoded.starts_with('/') {
        format!("file://{encoded}")
    } else {
        encoded
    }
}

/// Index-specific styling, injected into the generated page only — it does
/// not belong in the monodoc shell, which renders writeups, not catalogues.
const INDEX_CSS: &str = "
    /* lyceum index — generated home page.
       A catalogue, not a reading surface: it breaks the 55% prose
       measure and lays each writeup out as a wide row for scanning.
       No italics anywhere — on a non-prose surface a slant is just
       decoration; italics are reserved for emphasis within prose. */

    .lyceum-index { width: auto; }

    /* The page subtitle (\"N writeups\") is a count, not prose — upright. */
    p.subtitle { font-style: normal; }

    /* Collection headings reuse monodoc's <h2> so the margin
       table-of-contents lists the collections for free. The section
       rule sits ABOVE the heading: a rule beneath it would bind the
       heading visually to the section that precedes it. */
    .lyceum-index h2 {
      width: auto;
      font-style: normal;
      margin-top: 2.6rem;
      margin-bottom: 0.55rem;
      padding-top: 0.6rem;
      border-top: 1px solid var(--text-faint);
    }
    .lyceum-index h2:first-child { margin-top: 1.6rem; }

    /* Each writeup is a row: title | description | tags | date.
       Identical column tracks on every row line the columns up down
       the page. Every field is the same size — hierarchy is carried
       by tone alone, never by size or slant. */
    .lyceum-row {
      display: grid;
      grid-template-columns: minmax(0, 17rem) minmax(0, 1fr) minmax(0, 14rem) 5.5rem;
      gap: 1.5rem;
      align-items: baseline;
      padding: 0.4rem 0;
      font-size: 0.95rem;
      text-decoration: none;
      color: inherit;
    }

    .lyceum-col-title { color: var(--text); transition: color 0.15s ease; }
    .lyceum-col-sub   { color: var(--text-muted); transition: color 0.15s ease; }
    .lyceum-col-tags,
    .lyceum-col-date {
      font-family: var(--sans);
      color: var(--text-faint);
      transition: color 0.15s ease;
    }
    .lyceum-col-date {
      text-align: right;
      white-space: nowrap;
    }

    /* Hover lifts the whole row one tone-rung — a coordinated change
       across all four columns, no background or border (monodoc
       Principle 3: state lives in tone, not geometry). */
    .lyceum-row:hover .lyceum-col-title { color: var(--text-head); }
    .lyceum-row:hover .lyceum-col-sub  { color: var(--text); }
    .lyceum-row:hover .lyceum-col-tags,
    .lyceum-row:hover .lyceum-col-date { color: var(--text-muted); }

    /* Phone: no room for columns — stack the row. */
    @media (max-width: 800px) {
      .lyceum-row {
        grid-template-columns: 1fr;
        gap: 0.1rem;
        padding: 0.55rem 0;
      }
      .lyceum-col-date { text-align: left; }
    }
";
