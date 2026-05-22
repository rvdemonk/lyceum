//! `lyceum sync` — ship the library to a host.
//!
//! The host serves the bundle as plain static files; it runs no lyceum
//! process. Sync builds a *filtered* bundle — every writeup except the
//! `local_only` ones — into a staging directory, then rsyncs it.
//!
//! The rsync destination is deployment configuration, not code, so it
//! lives in `~/lyceum/.sync-target` rather than being compiled in.

use crate::{bundle, registry};
use std::fs;
use std::process::Command;

/// Build the filtered bundle and rsync it to the host.
pub fn sync(target: Option<String>, dry_run: bool) -> Result<String, String> {
    let target = match target {
        Some(t) => t,
        None => read_target()?,
    };

    let (staging, synced, withheld) = bundle::build_sync_staging()?;

    let mut args: Vec<String> = vec!["-az".into(), "--delete".into()];
    if dry_run {
        args.push("--dry-run".into());
        args.push("--itemize-changes".into());
    }
    // Trailing slash: copy the directory's *contents* into the target.
    args.push(format!("{}/", staging.display()));
    args.push(target.clone());

    let status = Command::new("rsync")
        .args(&args)
        .status()
        .map_err(|e| format!("running rsync: {e}"))?;
    if !status.success() {
        return Err(format!("rsync exited unsuccessfully ({status})"));
    }

    let held = if withheld > 0 {
        format!(" ({withheld} held back as local_only)")
    } else {
        String::new()
    };
    let verb = if dry_run { "would sync" } else { "synced" };
    Ok(format!("{verb} {synced} writeups \u{2192} {target}{held}"))
}

/// The rsync target, from `~/lyceum/.sync-target` — a one-line file
/// holding e.g. `root@host:/var/www/lyceum/`.
fn read_target() -> Result<String, String> {
    let path = registry::home_dir().join(".sync-target");
    match fs::read_to_string(&path) {
        Ok(s) if !s.trim().is_empty() => Ok(s.trim().to_string()),
        _ => Err(format!(
            "no sync target. Write the rsync destination to {} \
             (e.g. root@host:/var/www/lyceum/), or pass --target.",
            path.display()
        )),
    }
}
