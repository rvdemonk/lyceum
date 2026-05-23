//! lyceum — a personal, LLM-maintained wiki.
//!
//! v0: the writeup renderer and a self-contained library bundle. A
//! writeup (vanilla Markdown + YAML front-matter + HTML escape hatches)
//! renders into `~/lyceum/` — a bundle that can be opened directly,
//! served locally with live-reload, or synced to a host as static
//! files. The rest of the wiki layer is not built yet; see LYCEUM.md at
//! the repo root.
//!
//! `main.rs` is a thin dispatcher — the build pipeline lives in
//! `bundle`, the dev server in `serve`, the host upload in `sync`.

mod bundle;
mod frontmatter;
mod index;
mod library;
mod registry;
mod render;
mod serve;
mod sync;

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(name = "lyceum", version, about = "Personal LLM-maintained wiki (v0)")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render a writeup (.md) into the library bundle and refresh the index.
    Render {
        /// Input markdown file.
        input: PathBuf,
        /// monodoc writeup shell to render into (default: kernel/demo.html).
        #[arg(long)]
        shell: Option<PathBuf>,
    },
    /// Regenerate the home-page index from the registry.
    Index,
    /// Re-render every registered writeup into a clean bundle.
    Build,
    /// Serve the library locally over http, with live-reload.
    Serve {
        /// Port to listen on.
        #[arg(long, default_value_t = 4321)]
        port: u16,
        /// Address to bind. Use 0.0.0.0 to reach it from another device.
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },
    /// Sync the bundle (excluding local_only writeups) to the host.
    Sync {
        /// rsync destination; overrides ~/lyceum/.sync-target.
        #[arg(long)]
        target: Option<String>,
        /// Show what would transfer without changing the host.
        #[arg(long)]
        dry_run: bool,
    },
    /// List registered writeups.
    Ls {
        /// Filter to one collection.
        #[arg(long)]
        collection: Option<String>,
        /// Filter to writeups carrying this tag.
        #[arg(long)]
        tag: Option<String>,
        /// Verbose: one stanza per writeup with all metadata.
        #[arg(long)]
        long: bool,
        /// Emit the filtered slice as JSON.
        #[arg(long)]
        json: bool,
    },
    /// Print a writeup's source Markdown (or rendered HTML with --rendered).
    Read {
        /// Writeup slug, as shown by `lyceum ls`.
        slug: String,
        /// Read the rendered HTML from the bundle instead of the source MD.
        #[arg(long)]
        rendered: bool,
    },
    /// Case-insensitive substring search across registered source files.
    Grep {
        /// Pattern to search for.
        pattern: String,
        /// Limit search to one collection.
        #[arg(long)]
        collection: Option<String>,
        /// Limit search to writeups carrying this tag.
        #[arg(long)]
        tag: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Render { input, shell } => {
            let shell = shell.unwrap_or_else(bundle::default_shell);
            bundle::render_single(&input, &shell, &bundle::default_index_shell())
                .map(|p| format!("rendered {} \u{2192} {}", input.display(), p.display()))
        }
        Command::Index => {
            bundle::reindex().map(|p| format!("index \u{2192} {}", p.display()))
        }
        Command::Build => bundle::build_local().map(|n| {
            format!(
                "built {n} writeups \u{2192} {}",
                registry::home_dir().display()
            )
        }),
        Command::Serve { port, host } => serve::serve(&host, port).map(|()| String::new()),
        Command::Sync { target, dry_run } => sync::sync(target, dry_run),
        Command::Ls {
            collection,
            tag,
            long,
            json,
        } => library::ls(library::LsOptions {
            collection,
            tag,
            long,
            json,
        }),
        Command::Read { slug, rendered } => library::read(&slug, rendered),
        Command::Grep {
            pattern,
            collection,
            tag,
        } => library::grep(&pattern, library::GrepOptions { collection, tag }),
    };

    match result {
        Ok(msg) => {
            if !msg.is_empty() {
                println!("{msg}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("lyceum: error: {e}");
            ExitCode::FAILURE
        }
    }
}
