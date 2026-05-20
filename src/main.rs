//! lyceum — a personal, LLM-maintained wiki.
//!
//! v0 implements the renderer and a generated home-page index. A writeup
//! (vanilla Markdown + YAML front-matter + HTML escape hatches) is rendered
//! to a self-contained monodoc HTML page; every render is recorded in a
//! central registry from which `~/lyceum/index.html` is regenerated. The
//! rest of the wiki layer is not built yet — see LYCEUM.md at the repo root.

mod frontmatter;
mod index;
mod registry;
mod render;

use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "lyceum", version, about = "Personal LLM-maintained wiki (v0)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render a writeup (.md) to a self-contained monodoc HTML page, and
    /// refresh the home-page index.
    Render {
        /// Input markdown file.
        input: PathBuf,
        /// Output HTML file. Defaults to <input>.html beside the input.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// monodoc HTML shell to render into.
        /// Defaults to the bundled kernel/demo.html.
        #[arg(long)]
        shell: Option<PathBuf>,
    },
    /// Regenerate the home-page index (~/lyceum/index.html) from the
    /// registry, without rendering anything.
    Index {
        /// monodoc HTML shell to render the index into.
        #[arg(long)]
        shell: Option<PathBuf>,
    },
}

fn default_shell() -> PathBuf {
    // The monodoc kernel ships inside this repo at `kernel/`. Resolve the
    // shell relative to the crate so it is found regardless of the working
    // directory — no absolute path hardcoded to one machine's layout.
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/kernel/demo.html"))
}

/// The home page has its own shell — it is an instrument, not a writeup,
/// so it does not render into the monodoc document shell.
fn default_index_shell() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/kernel/index-shell.html"))
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Render { input, output, shell } => {
            let shell = shell.unwrap_or_else(default_shell);
            let output = output.unwrap_or_else(|| input.with_extension("html"));
            run_render(&input, &output, &shell)
                .map(|()| format!("rendered {} -> {}", input.display(), output.display()))
        }
        Command::Index { shell } => {
            let shell = shell.unwrap_or_else(default_index_shell);
            run_index(&shell).map(|p| format!("index -> {p}"))
        }
    };
    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("lyceum: error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Read the monodoc shell and the theme.css that sits beside it.
fn load_shell(shell: &Path) -> Result<(String, Option<String>), String> {
    let shell_html = fs::read_to_string(shell)
        .map_err(|e| format!("reading shell {}: {e}", shell.display()))?;
    let theme_css = shell
        .parent()
        .map(|d| d.join("theme.css"))
        .and_then(|p| fs::read_to_string(p).ok());
    Ok((shell_html, theme_css))
}

fn run_render(input: &Path, output: &Path, shell: &Path) -> Result<(), String> {
    let md = fs::read_to_string(input)
        .map_err(|e| format!("reading {}: {e}", input.display()))?;
    let (shell_html, theme_css) = load_shell(shell)?;

    let doc = render::render_document(&md)?;
    let html = assemble(
        &shell_html,
        theme_css.as_deref(),
        &doc.title,
        &doc.article_html,
        None,
    )?;
    fs::write(output, html).map_err(|e| format!("writing {}: {e}", output.display()))?;

    // Record the render and refresh the home page. Best-effort: a registry
    // problem must not fail an otherwise-successful render. The index is
    // regenerated into its own shell, independent of the writeup's.
    if let Err(e) = register(input, output, &doc) {
        eprintln!("lyceum: warning: registry not updated: {e}");
    } else if let Err(e) = regenerate_index(&default_index_shell()) {
        eprintln!("lyceum: warning: index not refreshed: {e}");
    }
    Ok(())
}

fn register(input: &Path, output: &Path, doc: &render::Document) -> Result<(), String> {
    let entry = registry::Entry {
        title: doc.title.clone(),
        subtitle: doc.subtitle.clone(),
        tags: doc.tags.clone(),
        collection: doc.collection.clone(),
        created: doc.created.clone(),
        updated: doc.updated.clone(),
        source: abs(input)?,
        output: abs(output)?,
    };
    registry::upsert(entry)
}

fn run_index(index_shell: &Path) -> Result<String, String> {
    regenerate_index(index_shell)?;
    Ok(registry::index_path().display().to_string())
}

/// Rebuild `~/lyceum/index.html` from the registry, into the index shell.
fn regenerate_index(index_shell: &Path) -> Result<(), String> {
    let (shell_html, theme_css) = load_shell(index_shell)?;

    let mut entries = registry::load()?;

    // Self-heal: drop entries whose source .md has been deleted, so the
    // home page never accumulates dead links.
    let before = entries.len();
    entries.retain(|e| Path::new(&e.source).exists());
    if entries.len() != before {
        registry::save(&entries)?;
    }

    let article = index::build(&entries);
    let html = assemble(&shell_html, theme_css.as_deref(), "Lyceum", &article, None)?;

    let dir = registry::home_dir();
    fs::create_dir_all(&dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    let path = registry::index_path();
    fs::write(&path, html).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// Canonical absolute path, as a string.
fn abs(p: &Path) -> Result<String, String> {
    fs::canonicalize(p)
        .map(|c| c.to_string_lossy().into_owned())
        .map_err(|e| format!("resolving {}: {e}", p.display()))
}

/// Splice a rendered article + title into the monodoc shell, inlining
/// theme.css and optionally an extra stylesheet, producing one
/// self-contained file.
fn assemble(
    shell: &str,
    theme_css: Option<&str>,
    title: &str,
    article_html: &str,
    extra_css: Option<&str>,
) -> Result<String, String> {
    let mut out = shell.to_string();

    // Inline theme.css in place of its <link>.
    if let Some(css) = theme_css {
        let link = "<link rel=\"stylesheet\" href=\"./theme.css\">";
        if out.contains(link) {
            out = out.replace(link, &format!("<style>\n{css}\n  </style>"));
        }
    }

    // Swap the document <title>.
    if let (Some(s), Some(e)) = (out.find("<title>"), out.find("</title>")) {
        if s < e {
            let t = format!("<title>{}</title>", render::html_escape(title));
            out.replace_range(s..e + "</title>".len(), &t);
        }
    }

    // Inject any extra stylesheet just before </head>.
    if let Some(css) = extra_css {
        if let Some(pos) = out.find("</head>") {
            out.insert_str(pos, &format!("  <style>{css}  </style>\n"));
        }
    }

    // Replace the <article> body.
    let astart = out.find("<article>").ok_or("shell has no <article> element")?;
    let aend = out.find("</article>").ok_or("shell has no </article> element")?;
    if aend < astart {
        return Err("shell <article> tags are out of order".into());
    }
    let article = format!("<article>\n{article_html}  </article>");
    out.replace_range(astart..aend + "</article>".len(), &article);

    Ok(out)
}
