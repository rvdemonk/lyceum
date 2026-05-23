//! Read access to the library.
//!
//! The renderer writes; this module reads. The three verbs here —
//! `ls`, `read`, `grep` — exist so anything (or anyone) with a shell
//! prompt can ask the library *what is in it* without crawling project
//! directories. The slug is the indirection: the caller names a
//! writeup; the registry resolves it to a source path.
//!
//! Default reads return the source `.md`, not the rendered HTML. The
//! HTML carries chrome — instrument scaffolding, primitive
//! class-wrappers, footnote machinery — that is for human eyes; for
//! programmatic re-reading the Markdown is denser and cleaner. `read`
//! takes `--rendered` for the rare cases where the rendered form is
//! actually what's wanted (debugging the renderer, auditing a
//! primitive's output).

use crate::registry::{self, Entry};
use std::fs;
use std::path::Path;

// ---------------------------------------------------------------------------
// ls
// ---------------------------------------------------------------------------

pub struct LsOptions {
    pub collection: Option<String>,
    pub tag: Option<String>,
    pub long: bool,
    pub json: bool,
}

pub fn ls(opts: LsOptions) -> Result<String, String> {
    let entries = filtered(opts.collection.as_deref(), opts.tag.as_deref())?;

    if opts.json {
        return serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("serialising entries: {e}"));
    }

    if entries.is_empty() {
        return Ok(String::from("(no writeups match)"));
    }

    // Sort: collection (None last), then title.
    let mut sorted = entries;
    sorted.sort_by(|a, b| match (&a.collection, &b.collection) {
        (Some(x), Some(y)) => x.cmp(y).then_with(|| a.title.cmp(&b.title)),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.title.cmp(&b.title),
    });

    let mut out = String::new();
    if opts.long {
        for (i, e) in sorted.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            out.push_str(&format_long(e));
        }
    } else {
        let slug_w = sorted.iter().map(|e| e.slug.len()).max().unwrap_or(0);
        let coll_w = sorted
            .iter()
            .map(|e| e.collection.as_deref().unwrap_or("-").len())
            .max()
            .unwrap_or(0);
        for e in &sorted {
            let coll = e.collection.as_deref().unwrap_or("-");
            let tags = if e.tags.is_empty() {
                String::new()
            } else {
                format!("  [{}]", e.tags.join(", "))
            };
            out.push_str(&format!(
                "{:<slug_w$}  {:<coll_w$}  {}{}\n",
                e.slug,
                coll,
                e.title,
                tags,
                slug_w = slug_w,
                coll_w = coll_w,
            ));
        }
        // Trim trailing newline so callers control spacing.
        if out.ends_with('\n') {
            out.pop();
        }
    }
    Ok(out)
}

fn format_long(e: &Entry) -> String {
    let mut s = String::new();
    s.push_str(&format!("slug:       {}\n", e.slug));
    s.push_str(&format!("title:      {}\n", e.title));
    if let Some(sub) = &e.subtitle {
        s.push_str(&format!("subtitle:   {sub}\n"));
    }
    if let Some(c) = &e.collection {
        s.push_str(&format!("collection: {c}\n"));
    }
    if !e.tags.is_empty() {
        s.push_str(&format!("tags:       [{}]\n", e.tags.join(", ")));
    }
    if let Some(c) = &e.created {
        s.push_str(&format!("created:    {c}\n"));
    }
    if let Some(u) = &e.updated {
        s.push_str(&format!("updated:    {u}\n"));
    }
    if e.local_only {
        s.push_str("local_only: true\n");
    }
    s.push_str(&format!("source:     {}\n", e.source));
    s
}

// ---------------------------------------------------------------------------
// read
// ---------------------------------------------------------------------------

pub fn read(slug: &str, rendered: bool) -> Result<String, String> {
    let entry = resolve_slug(slug)?;
    if rendered {
        let path = entry.page_in(&registry::home_dir());
        fs::read_to_string(&path)
            .map_err(|e| format!("reading {}: {e}", path.display()))
    } else {
        let path = Path::new(&entry.source);
        fs::read_to_string(path).map_err(|e| {
            format!(
                "reading {}: {e}\n(the source has moved or been deleted; \
                 try --rendered to read the bundled HTML instead, or re-render \
                 the writeup from its new location)",
                path.display()
            )
        })
    }
}

// ---------------------------------------------------------------------------
// grep
// ---------------------------------------------------------------------------

pub struct GrepOptions {
    pub collection: Option<String>,
    pub tag: Option<String>,
}

pub fn grep(pattern: &str, opts: GrepOptions) -> Result<String, String> {
    if pattern.is_empty() {
        return Err("empty pattern".into());
    }
    let needle = pattern.to_lowercase();
    let entries = filtered(opts.collection.as_deref(), opts.tag.as_deref())?;

    let mut out = String::new();
    let mut hits = 0usize;
    for e in &entries {
        let path = Path::new(&e.source);
        let body = match fs::read_to_string(path) {
            Ok(b) => b,
            Err(err) => {
                eprintln!(
                    "lyceum: skipping {} ({}): {err}",
                    e.slug,
                    path.display()
                );
                continue;
            }
        };
        for (i, line) in body.lines().enumerate() {
            if line.to_lowercase().contains(&needle) {
                hits += 1;
                out.push_str(&format!("{}:{}: {}\n", e.slug, i + 1, line));
            }
        }
    }
    if hits == 0 {
        return Ok(format!("(no matches for {pattern:?})"));
    }
    if out.ends_with('\n') {
        out.pop();
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// shared
// ---------------------------------------------------------------------------

fn filtered(collection: Option<&str>, tag: Option<&str>) -> Result<Vec<Entry>, String> {
    let entries = registry::load()?;
    Ok(entries
        .into_iter()
        .filter(|e| match collection {
            Some(c) => e.collection.as_deref() == Some(c),
            None => true,
        })
        .filter(|e| match tag {
            Some(t) => e.tags.iter().any(|x| x == t),
            None => true,
        })
        .collect())
}

/// Resolve a slug to an entry. On miss, suggest the closest registered
/// slugs by simple substring containment — enough to catch typos
/// without bringing in a Levenshtein dependency.
fn resolve_slug(slug: &str) -> Result<Entry, String> {
    let entries = registry::load()?;
    if let Some(e) = entries.iter().find(|e| e.slug == slug) {
        return Ok(e.clone());
    }
    let needle = slug.to_lowercase();
    let mut suggestions: Vec<&str> = entries
        .iter()
        .filter(|e| {
            e.slug.to_lowercase().contains(&needle)
                || needle.contains(&e.slug.to_lowercase())
        })
        .map(|e| e.slug.as_str())
        .collect();
    suggestions.sort();
    suggestions.dedup();
    if suggestions.is_empty() {
        Err(format!(
            "no writeup with slug {slug:?} (try `lyceum ls` to see what's registered)"
        ))
    } else {
        Err(format!(
            "no writeup with slug {slug:?}. did you mean: {}",
            suggestions.join(", ")
        ))
    }
}
