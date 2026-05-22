//! Markdown -> monodoc article HTML.
//!
//! The transform is deliberately small. Vanilla Markdown is parsed by
//! pulldown-cmark; only two things are rewritten:
//!
//!   1. Footnotes (`[^name]`) become monodoc's sidenote primitive — an
//!      empty `<span class="sn">` reference inline, and an
//!      `<aside class="sidenote">` placed as a sibling immediately after
//!      the paragraph that referenced it.
//!   2. Fenced ```mermaid blocks become `<div class="mermaid">` so the
//!      shell's mermaid.js renders them.
//!
//! Everything else — including raw HTML blocks such as custom figures,
//! `<span class="newthought">`, or hand-written `<aside class="sidenote">`
//! — passes through untouched. That is the whole point of the
//! "vanilla MD + HTML escape hatches" contract: the renderer stays thin.

use pulldown_cmark::html;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

use crate::frontmatter;

/// A rendered writeup: the page title, the front-matter metadata the
/// registry needs, and the inner HTML of `<article>`.
pub struct Document {
    pub title: String,
    pub subtitle: Option<String>,
    pub tags: Vec<String>,
    pub collection: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub local_only: bool,
    pub article_html: String,
}

/// Render a full writeup (front-matter + body) into a [`Document`].
pub fn render_document(input: &str) -> Result<Document, String> {
    let (fm, body) = frontmatter::split(input);
    let title = fm.title.clone().unwrap_or_else(|| "Untitled".to_string());

    let mut article = String::new();

    // Epigraph — optional, from front-matter.
    if let Some(ep) = &fm.epigraph {
        article.push_str("  <div class=\"epigraph\">\n");
        article.push_str(&format!("    <p>{}</p>\n", html_escape(ep)));
        if let Some(src) = &fm.epigraph_source {
            article.push_str(&format!("    <footer>{}</footer>\n", html_escape(src)));
        }
        article.push_str("  </div>\n\n");
    }

    // Title + optional subtitle.
    article.push_str(&format!("  <h1>{}</h1>\n", html_escape(&title)));
    if let Some(sub) = &fm.subtitle {
        article.push_str(&format!(
            "  <p class=\"subtitle\">{}</p>\n",
            html_escape(sub)
        ));
    }
    article.push('\n');

    // Body.
    article.push_str(&render_body(body));
    article.push('\n');

    Ok(Document {
        title,
        subtitle: fm.subtitle.clone(),
        tags: fm.tags.clone(),
        collection: fm.collection.clone(),
        created: fm.created.clone(),
        updated: fm.updated.clone(),
        local_only: fm.local_only,
        article_html: article,
    })
}

/// Render the Markdown body to HTML, applying the footnote and mermaid
/// transforms.
fn render_body(md: &str) -> String {
    let mut opts = Options::empty();
    opts.insert(Options::ENABLE_FOOTNOTES);
    opts.insert(Options::ENABLE_TABLES);
    opts.insert(Options::ENABLE_STRIKETHROUGH);
    opts.insert(Options::ENABLE_SMART_PUNCTUATION);

    let events: Vec<Event> = Parser::new_ext(md, opts).collect();
    let transformed = transform(events);

    let mut out = String::new();
    html::push_html(&mut out, transformed.into_iter());
    out
}

/// Apply the footnote -> sidenote and ```mermaid -> div rewrites.
fn transform(events: Vec<Event<'_>>) -> Vec<Event<'_>> {
    // --- Pass A: lift footnote definitions out of the stream ---------------
    // Definitions are usually written at the foot of the document; collecting
    // them first means Pass B has every definition available the moment it
    // meets a reference, wherever the two sit relative to each other.
    let mut defs: HashMap<String, String> = HashMap::new();
    let mut body: Vec<Event> = Vec::with_capacity(events.len());

    let mut i = 0;
    while i < events.len() {
        if let Event::Start(Tag::FootnoteDefinition(name)) = &events[i] {
            let name = name.to_string();
            i += 1;
            let mut inner: Vec<Event> = Vec::new();
            let mut depth = 1;
            while i < events.len() {
                match &events[i] {
                    Event::Start(Tag::FootnoteDefinition(_)) => depth += 1,
                    Event::End(TagEnd::FootnoteDefinition) => {
                        depth -= 1;
                        if depth == 0 {
                            i += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                inner.push(events[i].clone());
                i += 1;
            }
            // Strip the wrapping <p> so the note text sits directly inside
            // the <aside> (monodoc styles the aside, not a child paragraph).
            let stripped: Vec<Event> = inner
                .into_iter()
                .filter(|e| {
                    !matches!(
                        e,
                        Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph)
                    )
                })
                .collect();
            let mut buf = String::new();
            html::push_html(&mut buf, stripped.into_iter());
            defs.insert(name, buf.trim().to_string());
        } else {
            body.push(events[i].clone());
            i += 1;
        }
    }

    // --- Pass B: rewrite references and mermaid blocks ---------------------
    let mut out: Vec<Event> = Vec::with_capacity(body.len());
    let mut name_to_id: HashMap<String, usize> = HashMap::new();
    let mut counter = 0usize;
    let mut pending: Vec<String> = Vec::new(); // asides awaiting a paragraph end

    let mut in_mermaid = false;
    let mut mermaid_src = String::new();

    for ev in body {
        match ev {
            // Open of a ```mermaid fence: start capturing, emit nothing.
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref info)))
                if info.split_whitespace().next() == Some("mermaid") =>
            {
                in_mermaid = true;
                mermaid_src.clear();
            }
            // Code text inside a mermaid fence: accumulate.
            Event::Text(ref t) if in_mermaid => {
                mermaid_src.push_str(t);
            }
            // Close of a mermaid fence: emit the diagram div. The source is
            // HTML-escaped so the browser's parser leaves it intact for
            // mermaid.js to read back via textContent (this is also what
            // keeps `<br/>` in node labels from being eaten).
            Event::End(TagEnd::CodeBlock) if in_mermaid => {
                in_mermaid = false;
                out.push(Event::Html(
                    format!(
                        "<div class=\"mermaid\">\n{}\n</div>\n",
                        html_escape(mermaid_src.trim_end())
                    )
                    .into(),
                ));
            }
            // A footnote reference: emit the inline sidenote marker and
            // queue the matching aside to be flushed after this paragraph.
            Event::FootnoteReference(name) => {
                let name = name.to_string();
                let id = *name_to_id.entry(name.clone()).or_insert_with(|| {
                    counter += 1;
                    counter
                });
                out.push(Event::InlineHtml(
                    format!("<span class=\"sn\" id=\"sn-{id}\"></span>").into(),
                ));
                if let Some(def) = defs.get(&name) {
                    pending.push(format!(
                        "<aside class=\"sidenote\" data-ref=\"sn-{id}\">{def}</aside>"
                    ));
                }
            }
            // Paragraph end: flush any sidenotes referenced within it, so
            // each aside lands as a sibling directly after its paragraph.
            Event::End(TagEnd::Paragraph) => {
                out.push(Event::End(TagEnd::Paragraph));
                for aside in pending.drain(..) {
                    out.push(Event::Html(format!("\n{aside}\n").into()));
                }
            }
            other => out.push(other),
        }
    }

    // Footnote referenced outside any paragraph (rare): don't lose the note.
    for aside in pending.drain(..) {
        out.push(Event::Html(format!("\n{aside}\n").into()));
    }

    out
}

/// Escape the five characters that matter in HTML text/attribute content.
pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
