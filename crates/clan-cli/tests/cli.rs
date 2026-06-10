// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! End-to-end CLI tests, run against the built `clan` binary.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn clan(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_clan"))
        .args(args)
        .output()
        .expect("failed to run clan binary")
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
