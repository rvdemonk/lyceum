//! `lyceum serve` — a local development server.
//!
//! Serves the `~/lyceum/` bundle over http with live-reload. This is a
//! *local* tool only: a host serving the library needs no lyceum process
//! at all — the bundle is static files behind a plain web server. Serve
//! and host are decoupled; the bundle is the shared artifact.
//!
//! File-watching is mtime polling rather than a `notify` dependency: the
//! watch set is small (the kernel shells plus each writeup source) and
//! polling keeps the dependency surface minimal.

use crate::{bundle, registry};
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
use tiny_http::{Header, Request, Response, Server, StatusCode};

type Body = Response<Cursor<Vec<u8>>>;

/// Serve the local bundle with live-reload. Blocks until interrupted.
pub fn serve(host: &str, port: u16) -> Result<(), String> {
    let built = bundle::build_local()?;
    println!("lyceum: built {built} writeups");

    let generation = Arc::new(AtomicU64::new(1));
    {
        let generation = Arc::clone(&generation);
        thread::spawn(move || watch(generation));
    }

    let addr = format!("{host}:{port}");
    let server = Server::http(&addr).map_err(|e| format!("binding {addr}: {e}"))?;
    println!("lyceum serve \u{2192} http://{addr}  (Ctrl-C to stop)");

    for request in server.incoming_requests() {
        let _ = handle(request, &generation);
    }
    Ok(())
}

/// Serve one request out of the bundle directory.
fn handle(request: Request, generation: &AtomicU64) -> std::io::Result<()> {
    let raw = request.url().to_string();
    let path = raw.split(['?', '#']).next().unwrap_or("/");

    // Live-reload poll endpoint: the current build generation.
    if path == "/.live" {
        let body = generation.load(Ordering::Relaxed).to_string();
        return request.respond(text(body, "text/plain"));
    }

    let rel = if path == "/" {
        "index.html".to_string()
    } else {
        path.trim_start_matches('/').to_string()
    };
    if rel.contains("..") {
        return request.respond(text("forbidden".into(), "text/plain").with_status_code(StatusCode(403)));
    }

    let file = registry::home_dir().join(&rel);
    match std::fs::read(&file) {
        Ok(bytes) => {
            let ctype = content_type(&rel);
            if ctype.starts_with("text/html") {
                let html = inject_live(&String::from_utf8_lossy(&bytes));
                request.respond(text(html, ctype))
            } else {
                request.respond(
                    Response::from_data(bytes).with_header(header("Content-Type", ctype)),
                )
            }
        }
        Err(_) => {
            request.respond(text("not found".into(), "text/plain").with_status_code(StatusCode(404)))
        }
    }
}

/// A `text/`-`ctype` response.
fn text(body: String, ctype: &str) -> Body {
    Response::from_string(body).with_header(header("Content-Type", ctype))
}

fn header(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("static header is valid")
}

fn content_type(rel: &str) -> &'static str {
    match rel.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript",
        Some("svg") => "image/svg+xml",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

/// Inject the live-reload poller before `</body>`. Done at serve time
/// only — the script never lands in the bundle on disk, so the synced
/// copy stays clean.
fn inject_live(html: &str) -> String {
    const SCRIPT: &str = "<script>(function(){var g=null;\
setInterval(function(){fetch('/.live').then(function(r){return r.text();})\
.then(function(t){if(g!==null&&t!==g){location.reload();}g=t;})\
.catch(function(){});},1000);})();</script>";
    match html.rfind("</body>") {
        Some(i) => format!("{}{}{}", &html[..i], SCRIPT, &html[i..]),
        None => format!("{html}{SCRIPT}"),
    }
}

/// Watch sources and kernel files; rebuild on change and bump the
/// generation so connected pages reload.
fn watch(generation: Arc<AtomicU64>) {
    let mut snapshot = scan();
    loop {
        thread::sleep(Duration::from_millis(700));
        let current = scan();
        if current != snapshot {
            snapshot = current;
            match bundle::build_local() {
                Ok(n) => {
                    generation.fetch_add(1, Ordering::Relaxed);
                    println!("lyceum: rebuilt {n} writeups");
                }
                Err(e) => eprintln!("lyceum: rebuild failed: {e}"),
            }
        }
    }
}

/// `(path, mtime)` for every file a rebuild depends on: the kernel
/// shells and theme, plus each registered writeup source.
///
/// The registry file itself is deliberately *not* watched — a rebuild
/// rewrites it, which would loop forever. The source list is re-read
/// from the registry every pass instead, so a writeup rendered in
/// another terminal is picked up on the next scan.
fn scan() -> Vec<(PathBuf, Option<SystemTime>)> {
    let kernel = bundle::default_shell();
    let mut files: Vec<PathBuf> = vec![bundle::default_shell(), bundle::default_index_shell()];
    if let Some(dir) = kernel.parent() {
        files.push(dir.join("theme.css"));
    }
    if let Ok(entries) = registry::load() {
        for e in entries {
            files.push(PathBuf::from(e.source));
        }
    }
    files.sort();
    files.dedup();
    files
        .into_iter()
        .map(|p| {
            let mtime = std::fs::metadata(&p).and_then(|m| m.modified()).ok();
            (p, mtime)
        })
        .collect()
}
