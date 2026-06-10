// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! End-to-end CLI tests, run against the built `clan` binary.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn clan(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_clan"))
        .args(args)
        // Keep unrelated tests hermetic: never write the first-run marker
        // into the developer's real config dir (see banner tests below).
        .env("CLAN_NO_BANNER", "1")
        .output()
        .expect("failed to run clan binary")
}

/// Run the binary with banner control left to the caller.
fn clan_banner(args: &[&str], config_dir: &Path, no_banner: bool) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clan"));
    cmd.args(args)
        .env_remove("CLAN_NO_BANNER")
        .env("CLAN_CONFIG_DIR", config_dir);
    if no_banner {
        cmd.env("CLAN_NO_BANNER", "1");
    }
    cmd.output().expect("failed to run clan binary")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn create_parent(dir: &Path) -> PathBuf {
    let parent = dir.join("parent.clan");
    let out = clan(&[
        "create",
        "--title",
        "CLI Test",
        "--brief",
        "test brief",
        "--output",
        parent.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "create failed: {}", stderr(&out));
    parent
}

// --- #22: clan create accepts --output like the other write commands ---

#[test]
fn create_with_output_flag() {
    let dir = tempfile::tempdir().unwrap();
    let path = create_parent(dir.path());
    assert!(path.exists());

    let info = clan(&["info", path.to_str().unwrap()]);
    assert!(info.status.success());
    assert!(String::from_utf8_lossy(&info.stdout).contains("CLI Test"));
}

#[test]
fn create_with_positional_output_still_works_but_warns() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.clan");
    let out = clan(&[
        "create",
        "--title",
        "Legacy",
        "--brief",
        "b",
        path.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "positional form must keep working: {}", stderr(&out));
    assert!(path.exists());
    assert!(
        stderr(&out).contains("deprecated"),
        "positional form should warn: {}",
        stderr(&out)
    );
}

#[test]
fn create_without_output_fails() {
    let out = clan(&["create", "--title", "T", "--brief", "b"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("--output"), "{}", stderr(&out));
}

#[test]
fn create_rejects_both_output_forms() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a.clan");
    let b = dir.path().join("b.clan");
    let out = clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "b",
        "--output",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "conflicting output paths must be rejected");
}

#[test]
fn pack_html_uses_output_flag() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let html = dir.path().join("doc.html");
    std::fs::write(&html, "<!DOCTYPE html><html><body><p>packed</p></body></html>").unwrap();
    let next = dir.path().join("next.clan");

    let out = clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        html.to_str().unwrap(),
        "--output",
        next.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "pack-html failed: {}", stderr(&out));
    assert!(next.exists());
}

// --- #21: patch-html must fail loudly when the selector matches nothing ---

#[test]
fn patch_html_nonmatching_selector_exits_nonzero() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let before = std::fs::read(&parent).unwrap();

    let patch = dir.path().join("patch.html");
    std::fs::write(
        &patch,
        "---\nmode: patch-html\npatch_selector: \"#does-not-exist\"\npatch_action: \"append\"\n---\n<div>new</div>",
    )
    .unwrap();

    let out = clan(&["patch-html", parent.to_str().unwrap(), patch.to_str().unwrap()]);
    assert!(
        !out.status.success(),
        "zero-match selector must exit non-zero (stderr: {})",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("matched no elements"),
        "stderr should explain the failure: {}",
        stderr(&out)
    );
    assert_eq!(
        before,
        std::fs::read(&parent).unwrap(),
        "file must be untouched after a failed patch"
    );
}

// --- #31: ASCII art on first installation ---

const BANNER_TAGLINE: &str = "Context and Live Agent Notation";

#[test]
fn banner_shows_on_first_run_only() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("cfg");

    let first = clan_banner(&["agent-help"], &cfg, false);
    assert!(first.status.success());
    assert!(
        stderr(&first).contains(BANNER_TAGLINE),
        "first run must greet on stderr: {}",
        stderr(&first)
    );
    assert!(
        stderr(&first).contains("_____"),
        "banner should contain the ASCII art: {}",
        stderr(&first)
    );

    // Marker recorded, carrying the version.
    let marker = cfg.join("welcomed");
    assert!(marker.exists());
    assert_eq!(
        std::fs::read_to_string(&marker).unwrap(),
        env!("CARGO_PKG_VERSION")
    );

    let second = clan_banner(&["agent-help"], &cfg, false);
    assert!(second.status.success());
    assert!(
        !stderr(&second).contains(BANNER_TAGLINE),
        "second run must be silent: {}",
        stderr(&second)
    );
}

#[test]
fn banner_never_pollutes_stdout() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("cfg");

    let first = clan_banner(&["agent-help"], &cfg, false);
    let second = clan_banner(&["agent-help"], &cfg, false);
    assert_eq!(
        first.stdout, second.stdout,
        "stdout must be byte-identical whether or not the banner fires"
    );
    assert!(!String::from_utf8_lossy(&first.stdout).contains(BANNER_TAGLINE));
}

#[test]
fn no_banner_env_suppresses_without_side_effects() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("cfg");

    let out = clan_banner(&["agent-help"], &cfg, true);
    assert!(out.status.success());
    assert!(!stderr(&out).contains(BANNER_TAGLINE));
    assert!(
        !cfg.join("welcomed").exists(),
        "CLAN_NO_BANNER must not write the marker"
    );

    // Opt back in afterwards: greeting still happens once.
    let later = clan_banner(&["agent-help"], &cfg, false);
    assert!(stderr(&later).contains(BANNER_TAGLINE));
}

#[test]
fn unwritable_config_dir_degrades_silently() {
    let dir = tempfile::tempdir().unwrap();
    // Point the config dir inside an existing FILE so create_dir_all fails.
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"file").unwrap();
    let cfg = blocker.join("cfg");

    let out = clan_banner(&["agent-help"], &cfg, false);
    assert!(out.status.success(), "command must still work: {}", stderr(&out));
    assert!(
        !stderr(&out).contains(BANNER_TAGLINE),
        "banner must not fire when the marker cannot be recorded (else it \
         would greet on every run): {}",
        stderr(&out)
    );
}

#[test]
fn banner_does_not_break_piped_workflows() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = dir.path().join("cfg");
    let parent = dir.path().join("p.clan");

    // First-ever invocation is a real command; banner goes to stderr, the
    // command's own output and exit code are unaffected.
    let create = clan_banner(
        &[
            "create", "--title", "Banner", "--brief", "b",
            "--output", parent.to_str().unwrap(),
        ],
        &cfg,
        false,
    );
    assert!(create.status.success(), "{}", stderr(&create));
    assert!(stderr(&create).contains(BANNER_TAGLINE));
    assert!(parent.exists());

    let read = clan_banner(&["read", "data", parent.to_str().unwrap()], &cfg, false);
    assert!(read.status.success());
    let data = String::from_utf8_lossy(&read.stdout);
    assert!(data.contains("$schema"), "stdout must be pure data: {data}");
    assert!(!data.contains(BANNER_TAGLINE));
}

// --- #24: clan read agent --skip-guide ---

#[test]
fn read_agent_skip_guide_omits_guide_body() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    let full = clan(&["read", "agent", parent.to_str().unwrap()]);
    assert!(full.status.success(), "{}", stderr(&full));
    let skipped = clan(&["read", "agent", parent.to_str().unwrap(), "--skip-guide"]);
    assert!(skipped.status.success(), "{}", stderr(&skipped));

    let full_text = String::from_utf8_lossy(&full.stdout).into_owned();
    let skipped_text = String::from_utf8_lossy(&skipped.stdout).into_owned();

    assert!(skipped_text.contains("guide body skipped"), "{skipped_text}");
    assert!(skipped_text.contains("sha256:"), "skip note must carry the guide digest");
    assert!(
        skipped_text.len() < full_text.len() / 2,
        "skipping the guide should cut the context substantially \
         (full: {}, skipped: {})",
        full_text.len(),
        skipped_text.len()
    );
    // The rest of the context is unchanged.
    let tail = |s: &str| s.split("# Your Task").nth(1).map(str::to_owned);
    assert_eq!(tail(&skipped_text), tail(&full_text));
}

#[test]
fn patch_html_matching_selector_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    let patch = dir.path().join("patch.html");
    std::fs::write(
        &patch,
        "---\nmode: patch-html\npatch_selector: \"section\"\npatch_action: \"append\"\n---\n<div id=\"added\">new</div>",
    )
    .unwrap();

    let out = clan(&["patch-html", parent.to_str().unwrap(), patch.to_str().unwrap()]);
    assert!(out.status.success(), "patch-html failed: {}", stderr(&out));

    let read = clan(&["read", "human", parent.to_str().unwrap()]);
    assert!(String::from_utf8_lossy(&read.stdout).contains("id=\"added\""));
}
