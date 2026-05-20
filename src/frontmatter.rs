//! Front-matter parsing.
//!
//! Deliberately not a full YAML parser. Lyceum writeups use a small, fixed
//! set of scalar and flat-list fields; a hand-rolled line parser covers them
//! with zero dependencies and no surprising YAML corner cases. If the
//! front-matter ever genuinely outgrows this, reach for a real YAML crate
//! then — not before.

/// The subset of front-matter fields a writeup actually uses.
#[derive(Debug, Default)]
pub struct FrontMatter {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub tags: Vec<String>,
    pub collection: Option<String>,
    pub epigraph: Option<String>,
    pub epigraph_source: Option<String>,
}

/// Split a document into (front-matter, body).
///
/// Front-matter is the block between a leading `---` line and the next `---`
/// line. With no such block, front-matter is empty and the whole input is
/// treated as body.
pub fn split(input: &str) -> (FrontMatter, &str) {
    let rest = match input
        .strip_prefix("---\n")
        .or_else(|| input.strip_prefix("---\r\n"))
    {
        Some(r) => r,
        None => return (FrontMatter::default(), input),
    };

    match find_close(rest) {
        Some((yaml, body)) => (parse(yaml), body),
        None => (FrontMatter::default(), input),
    }
}

/// Find the closing `---` fence, returning (yaml-block, body-after-fence).
fn find_close(rest: &str) -> Option<(&str, &str)> {
    let mut idx = 0;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some((&rest[..idx], &rest[idx + line.len()..]));
        }
        idx += line.len();
    }
    None
}

fn parse(yaml: &str) -> FrontMatter {
    let mut fm = FrontMatter::default();
    for line in yaml.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, val)) = line.split_once(':') else {
            continue;
        };
        let val = unquote(val.trim());
        match key.trim() {
            "title" => fm.title = nonempty(val),
            "subtitle" => fm.subtitle = nonempty(val),
            "created" => fm.created = nonempty(val),
            "updated" => fm.updated = nonempty(val),
            "epigraph" => fm.epigraph = nonempty(val),
            "epigraph_source" => fm.epigraph_source = nonempty(val),
            "collection" => fm.collection = nonempty(val),
            "tags" => fm.tags = parse_list(val),
            _ => {}
        }
    }
    fm
}

fn nonempty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

/// Strip a single matched pair of surrounding quotes.
fn unquote(s: &str) -> &str {
    let bytes = s.as_bytes();
    if bytes.len() >= 2 {
        let (first, last) = (bytes[0], bytes[bytes.len() - 1]);
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Parse a flat inline list: `[a, b, c]` or a bare comma-separated string.
fn parse_list(val: &str) -> Vec<String> {
    let inner = val
        .strip_prefix('[')
        .and_then(|v| v.strip_suffix(']'))
        .unwrap_or(val);
    inner
        .split(',')
        .map(|p| unquote(p.trim()).to_string())
        .filter(|p| !p.is_empty())
        .collect()
}
