//! macOS: when the app is run from a DMG (not from /Applications), install itself (and optionally configure CLI), then launch the installed app.

use std::path::PathBuf;
use std::process::Command;

const DEST_APP: &str = "/Applications/Port Sweeper.app";

/// Path to the .app bundle containing the current executable (Contents/MacOS/psweep -> .. -> ..).
fn app_bundle_path() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    // exe is .../Port Sweeper.app/Contents/MacOS/psweep
    let contents_macos = exe.parent()?;
    let contents = contents_macos.parent()?;
    let bundle = contents.parent()?;
    if bundle.extension()? == "app" {
        Some(bundle.to_path_buf())
    } else {
        None
    }
}

/// True if we're running from /Applications/Port Sweeper.app (already installed).
pub fn is_installed() -> bool {
    let bundle = match app_bundle_path() {
        Some(p) => p,
        None => return false,
    };
    let canonical_bundle = match std::fs::canonicalize(&bundle) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let canonical_apps = match std::fs::canonicalize("/Applications") {
        Ok(p) => p,
        Err(_) => return false,
    };
    canonical_bundle.starts_with(&canonical_apps)
}

/// Install this app to /Applications, clear quarantine, then open the installed app and exit.
/// CLI (psweep/port-sweeper in PATH) is commented out for this release — GUI only.
pub fn install_then_launch() -> anyhow::Result<()> {
    let bundle = app_bundle_path().ok_or_else(|| anyhow::anyhow!("not running from an app bundle"))?;
    let bundle_str = bundle.to_string_lossy();

    // Use /tmp so the privileged script (runs as root) can read these
    let path_file = PathBuf::from("/tmp/psweep_install_src");
    let script_file = PathBuf::from("/tmp/psweep_install_run.sh");

    std::fs::write(&path_file, bundle_str.as_bytes())?;

    let script = format!(
        r#"#!/bin/bash
set -e
SRC=$(cat "{}")
cp -R "$SRC" /Applications/
xattr -cr "{}"
# This release: GUI only — uncomment to add CLI to PATH
# mkdir -p /usr/local/bin
# ln -sf "{}/Contents/MacOS/psweep" /usr/local/bin/psweep
# ln -sf "{}/Contents/MacOS/psweep" /usr/local/bin/port-sweeper
"#,
        path_file.to_string_lossy(),
        DEST_APP,
        DEST_APP,
        DEST_APP,
    );
    std::fs::write(&script_file, script)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&script_file, std::fs::Permissions::from_mode(0o700))?;
    }

    let script_path = script_file.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"");
    let status = Command::new("osascript")
        .arg("-e")
        .arg(format!(
            "do shell script \"/bin/bash \\\"{}\\\"\" with administrator privileges",
            script_path
        ))
        .status()?;

    let _ = std::fs::remove_file(&path_file);
    let _ = std::fs::remove_file(&script_file);

    if !status.success() {
        anyhow::bail!("install was cancelled or failed");
    }

    Command::new("open").arg(DEST_APP).spawn()?;
    std::process::exit(0);
}
