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
    assert!(
        out.status.success(),
        "positional form must keep working: {}",
        stderr(&out)
    );
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
    assert!(
        !out.status.success(),
        "conflicting output paths must be rejected"
    );
}

#[test]
fn pack_html_uses_output_flag() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let html = dir.path().join("doc.html");
    std::fs::write(
        &html,
        "<!DOCTYPE html><html><body><p>packed</p></body></html>",
    )
    .unwrap();
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

#[test]
fn patch_asset_requires_attribution_and_records_it() {
    // F15: an asset write is a document mutation — attribute it (--no-decision opts out).
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let asset = dir.path().join("logo.svg");
    std::fs::write(&asset, "<svg/>").unwrap();
    let before = std::fs::read(&parent).unwrap();

    // No attribution → teaching error, file untouched.
    let out = clan(&[
        "patch-asset",
        parent.to_str().unwrap(),
        "logo.svg",
        asset.to_str().unwrap(),
    ]);
    assert!(
        !out.status.success(),
        "asset write without attribution must fail"
    );
    assert!(
        stderr(&out).contains("--agent") && stderr(&out).contains("--no-decision"),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        before,
        std::fs::read(&parent).unwrap(),
        "rejected write must not touch the file"
    );

    // With attribution → succeeds and records the decision in the chain.
    let out = clan(&[
        "patch-asset",
        parent.to_str().unwrap(),
        "logo.svg",
        asset.to_str().unwrap(),
        "--agent",
        "designer",
        "--action",
        "added logo",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let chain = stdout(&clan(&["read", "chain", parent.to_str().unwrap()]));
    assert!(
        chain.contains("designer") && chain.contains("added logo"),
        "{chain}"
    );

    // --no-decision succeeds without recording.
    let asset2 = dir.path().join("icon.svg");
    std::fs::write(&asset2, "<svg/>").unwrap();
    let out = clan(&[
        "patch-asset",
        parent.to_str().unwrap(),
        "icon.svg",
        asset2.to_str().unwrap(),
        "--no-decision",
    ]);
    assert!(
        out.status.success(),
        "--no-decision must work: {}",
        stderr(&out)
    );
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

    let out = clan(&[
        "patch-html",
        parent.to_str().unwrap(),
        patch.to_str().unwrap(),
        "--no-decision",
    ]);
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
    assert!(
        out.status.success(),
        "command must still work: {}",
        stderr(&out)
    );
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
            "create",
            "--title",
            "Banner",
            "--brief",
            "b",
            "--output",
            parent.to_str().unwrap(),
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

    assert!(
        skipped_text.contains("guide body skipped"),
        "{skipped_text}"
    );
    assert!(
        skipped_text.contains("sha256:"),
        "skip note must carry the guide digest"
    );
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

// --- v1.1: fork / merge / render / teachable hints (spec §23, §24, §27) ---

fn clan_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_clan"));
    cmd.args(args).env("CLAN_NO_BANNER", "1");
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("failed to run clan binary")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// create --no-render → fork → namespace writes → merge. Returns the merged
/// path; `status` is contested (last-write), `findings` folds via append.
fn forked_merged(dir: &Path) -> PathBuf {
    let root = dir.join("root.clan");
    let out = clan(&[
        "create",
        "--title",
        "Fork",
        "--brief",
        "b",
        "--no-render",
        "--output",
        root.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));

    let branches = dir.join("branches");
    let out = clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "researcher,analyst",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "fork failed: {}", stderr(&out));

    let writes = [
        (
            "researcher",
            r#"{"findings": ["r1"], "status": "approved"}"#,
        ),
        (
            "analyst",
            r#"{"findings": ["a1"], "status": "needs-review"}"#,
        ),
    ];
    for (agent, json) in writes {
        let patch = dir.join(format!("{agent}.json"));
        std::fs::write(&patch, json).unwrap();
        let branch = branches.join(format!("{agent}.clan"));
        let out = clan(&[
            "patch-data",
            branch.to_str().unwrap(),
            patch.to_str().unwrap(),
            "--namespace",
        ]);
        assert!(
            out.status.success(),
            "namespace write failed: {}",
            stderr(&out)
        );
    }

    let merged = dir.join("merged.clan");
    let out = clan(&[
        "merge",
        branches.join("researcher.clan").to_str().unwrap(),
        branches.join("analyst.clan").to_str().unwrap(),
        "--output",
        merged.to_str().unwrap(),
        "--policy",
        "findings=append",
    ]);
    assert!(out.status.success(), "merge failed: {}", stderr(&out));
    merged
}

#[test]
fn fork_merge_folds_namespaces_and_reports_conflicts() {
    let dir = tempfile::tempdir().unwrap();
    let merged = forked_merged(dir.path());

    // append policy kept both findings; last-write picked analyst's status.
    let data = stdout(&clan(&["read", "data", merged.to_str().unwrap()]));
    assert!(data.contains("r1") && data.contains("a1"), "{data}");
    assert!(data.contains("needs-review"), "{data}");

    // The contested key is in the report with full provenance.
    let report = stdout(&clan(&["read", "report", merged.to_str().unwrap()]));
    assert!(report.contains("status"), "{report}");
    assert!(
        report.contains("researcher"),
        "loser provenance missing: {report}"
    );
    assert!(report.contains("unresolved: 1"), "{report}");

    // Merged file validates and is a real multi-parent merge.
    let out = clan(&["validate", merged.to_str().unwrap()]);
    assert!(out.status.success(), "{}", stderr(&out));
}

#[test]
fn forked_write_guard_is_a_teaching_error() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "b",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "a,b",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);

    let patch = dir.path().join("p.json");
    std::fs::write(&patch, r#"{"x": 1}"#).unwrap();
    let branch = branches.join("a.clan");
    let before = std::fs::read(&branch).unwrap();

    // Without --namespace: rejected, and the error names the fix.
    let out = clan(&[
        "patch-data",
        branch.to_str().unwrap(),
        patch.to_str().unwrap(),
    ]);
    assert!(
        !out.status.success(),
        "shared write on a forked file must fail"
    );
    let err = stderr(&out);
    assert!(
        err.contains("--namespace"),
        "error must teach the alternative: {err}"
    );
    assert!(
        err.contains("clan merge"),
        "error must teach the join step: {err}"
    );
    assert_eq!(
        before,
        std::fs::read(&branch).unwrap(),
        "file must be untouched"
    );
}

#[test]
fn adjudication_settles_contested_keys() {
    let dir = tempfile::tempdir().unwrap();
    let merged = forked_merged(dir.path());

    let adj = dir.path().join("adj.json");
    std::fs::write(&adj, r#"{"status": "approved"}"#).unwrap();
    // F15: settle WITH attribution — the adjudication is recorded in one step.
    let out = clan(&[
        "patch-data",
        merged.to_str().unwrap(),
        adj.to_str().unwrap(),
        "--agent",
        "synthesizer",
        "--action",
        "adjudicated status",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("recorded under your attribution"),
        "an attributed settle is self-recording: {}",
        stderr(&out)
    );

    let report = stdout(&clan(&["read", "report", merged.to_str().unwrap()]));
    assert!(report.contains("unresolved: 0"), "{report}");
    let data = stdout(&clan(&["read", "data", merged.to_str().unwrap()]));
    assert!(data.contains("approved"), "{data}");
    // The adjudication landed in the chain with exact fields_changed.
    let chain = stdout(&clan(&["read", "chain", merged.to_str().unwrap()]));
    assert!(
        chain.contains("synthesizer") && chain.contains("adjudicated status"),
        "{chain}"
    );
}

#[test]
fn no_render_then_render_materialises_view() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("agentic.clan");
    let out = clan(&[
        "create",
        "--title",
        "A2A",
        "--brief",
        "b",
        "--no-render",
        "--output",
        root.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));

    // No view yet; the file still validates (view is optional, spec §23).
    let human = clan(&["read", "human", root.to_str().unwrap()]);
    assert!(!human.status.success(), "agent-only file has no human view");
    assert!(clan(&["validate", root.to_str().unwrap()]).status.success());

    let out = clan(&["render", root.to_str().unwrap()]);
    assert!(out.status.success(), "render failed: {}", stderr(&out));
    let html = stdout(&clan(&["read", "human", root.to_str().unwrap()]));
    assert!(html.contains("A2A"), "{html}");
    assert!(clan(&["validate", root.to_str().unwrap()]).status.success());
}

#[test]
fn read_agent_on_forked_file_injects_branch_mode_block() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "b",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "alpha,beta",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);

    let ctx = stdout(&clan(&[
        "read",
        "agent",
        branches.join("alpha.clan").to_str().unwrap(),
    ]));
    assert!(ctx.contains("# Branch Mode"), "{ctx}");
    assert!(ctx.contains("agents/alpha/"), "{ctx}");

    // Sequential (unforked) files get exactly the v1.0 injection: no block.
    let plain = stdout(&clan(&["read", "agent", root.to_str().unwrap()]));
    assert!(!plain.contains("# Branch Mode"), "{plain}");
    assert!(!plain.contains("# Contested Keys"), "{plain}");
}

#[test]
fn read_agent_on_conflicted_merge_injects_contested_keys() {
    let dir = tempfile::tempdir().unwrap();
    let merged = forked_merged(dir.path());

    let ctx = stdout(&clan(&["read", "agent", merged.to_str().unwrap()]));
    assert!(ctx.contains("# Contested Keys"), "{ctx}");
    assert!(ctx.contains("status"), "{ctx}");

    // After adjudication the block disappears — injection is state-gated.
    let adj = dir.path().join("adj.json");
    std::fs::write(&adj, r#"{"status": "approved"}"#).unwrap();
    clan(&[
        "patch-data",
        merged.to_str().unwrap(),
        adj.to_str().unwrap(),
        "--agent",
        "synthesizer",
        "--action",
        "adjudicated status",
    ]);
    let ctx = stdout(&clan(&["read", "agent", merged.to_str().unwrap()]));
    assert!(!ctx.contains("# Contested Keys"), "{ctx}");
}

#[test]
fn hints_are_emitted_and_suppressible() {
    let dir = tempfile::tempdir().unwrap();
    let mk = |name: &str, extra_args: &[&str], envs: &[(&str, &str)]| {
        let path = dir.path().join(name);
        let mut args = vec![
            "create",
            "--title",
            "H",
            "--brief",
            "b",
            "--output",
            path.to_str().unwrap(),
        ];
        args.extend_from_slice(extra_args);
        clan_env(&args, envs)
    };

    let with_hints = mk("a.clan", &[], &[]);
    assert!(
        stderr(&with_hints).contains("next:"),
        "{}",
        stderr(&with_hints)
    );
    // Hints never pollute stdout.
    assert!(!stdout(&with_hints).contains("next:"));

    let quiet = mk("b.clan", &["--quiet"], &[]);
    assert!(!stderr(&quiet).contains("next:"), "{}", stderr(&quiet));

    let env_off = mk("c.clan", &[], &[("CLAN_NO_HINTS", "1")]);
    assert!(!stderr(&env_off).contains("next:"), "{}", stderr(&env_off));
}

#[test]
fn hints_are_precondition_gated() {
    let dir = tempfile::tempdir().unwrap();

    // A rendered (default) file: no agent-only render hint, no merge talk.
    let rendered = dir.path().join("r.clan");
    let out = clan(&[
        "create",
        "--title",
        "R",
        "--brief",
        "b",
        "--output",
        rendered.to_str().unwrap(),
    ]);
    let err = stderr(&out);
    assert!(!err.contains("agent-only"), "{err}");
    assert!(
        !err.contains("merge"),
        "unforked file must never hear about merging: {err}"
    );

    // An agent-only file mentions render; still no merge talk.
    let bare = dir.path().join("n.clan");
    let out = clan(&[
        "create",
        "--title",
        "N",
        "--brief",
        "b",
        "--no-render",
        "--output",
        bare.to_str().unwrap(),
    ]);
    let err = stderr(&out);
    assert!(err.contains("clan render"), "{err}");
    assert!(!err.contains("merge"), "{err}");
}

// --- v1.1 finding fixes (F3, F7, F8, F9, F12) ---

#[test]
fn patch_data_tolerates_utf8_bom() {
    // PowerShell 5.1 Out-File writes a UTF-8 BOM by default (F3).
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let patch = dir.path().join("bom.json");
    let mut bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
    bytes.extend_from_slice(br#"{"vendor": "Acme"}"#);
    std::fs::write(&patch, bytes).unwrap();

    let out = clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        patch.to_str().unwrap(),
        "--no-decision",
    ]);
    assert!(
        out.status.success(),
        "BOM JSON must parse: {}",
        stderr(&out)
    );
    let data = stdout(&clan(&["read", "data", parent.to_str().unwrap()]));
    assert!(data.contains("Acme"), "{data}");
}

#[test]
fn read_decisions_aliases_read_chain() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let chain = clan(&["read", "chain", parent.to_str().unwrap()]);
    let decisions = clan(&["read", "decisions", parent.to_str().unwrap()]);
    assert!(
        decisions.status.success(),
        "alias must work: {}",
        stderr(&decisions)
    );
    assert_eq!(stdout(&chain), stdout(&decisions));
}

#[test]
fn create_seeds_schema_and_requirements() {
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("s.json");
    std::fs::write(
        &schema,
        r#"{"type":"object","required":["verdict"],"properties":{"verdict":{"type":"string"}}}"#,
    )
    .unwrap();
    let reqs = dir.path().join("r.yaml");
    std::fs::write(&reqs, "requires:\n  tools:\n    - name: web_search\n").unwrap();
    let path = dir.path().join("seeded.clan");

    let out = clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "b",
        "--schema",
        schema.to_str().unwrap(),
        "--requirements",
        reqs.to_str().unwrap(),
        "--output",
        path.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));

    // Schema seeded: a non-conforming pack is rejected by validation.
    let ctx = stdout(&clan(&["read", "agent", path.to_str().unwrap()]));
    assert!(
        ctx.contains("verdict"),
        "seeded schema fields must appear in context: {ctx}"
    );
    assert!(
        ctx.contains("# Capability Requirements"),
        "requirements block must inject: {ctx}"
    );
    assert!(ctx.contains("web_search"), "{ctx}");

    // A bad seed schema is rejected up front.
    let bad = dir.path().join("bad.clan");
    let badschema = dir.path().join("bad.json");
    std::fs::write(&badschema, "{not json").unwrap();
    let out = clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "b",
        "--schema",
        badschema.to_str().unwrap(),
        "--output",
        bad.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "invalid seed schema must fail");
}

#[test]
fn fork_context_dir_overrides_branch_task() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "parent brief",
        "--output",
        root.to_str().unwrap(),
    ]);

    let ctxdir = dir.path().join("ctx");
    std::fs::create_dir_all(&ctxdir).unwrap();
    std::fs::write(ctxdir.join("alpha.md"), "ALPHA TASK ONLY").unwrap();

    let branches = dir.path().join("br");
    let out = clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "alpha,beta",
        "--output-dir",
        branches.to_str().unwrap(),
        "--context-dir",
        ctxdir.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "{}", stderr(&out));

    let alpha_ctx = stdout(&clan(&[
        "read",
        "agent",
        branches.join("alpha.clan").to_str().unwrap(),
    ]));
    assert!(
        alpha_ctx.contains("ALPHA TASK ONLY"),
        "override applied: {alpha_ctx}"
    );
    let beta_ctx = stdout(&clan(&[
        "read",
        "agent",
        branches.join("beta.clan").to_str().unwrap(),
    ]));
    assert!(beta_ctx.contains("parent brief"), "beta keeps parent task");
    assert!(beta_ctx.contains("# Branch Mode"), "branch banner present");
}

#[test]
fn patch_data_no_decision_leaves_chain_empty() {
    // F1 + F15 at the CLI: an explicit --no-decision write adds NO chain entry
    // (and certainly no unknown-agent placeholder).
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let patch = dir.path().join("p.json");
    std::fs::write(&patch, r#"{"x": 1}"#).unwrap();
    let before = stdout(&clan(&["read", "chain", parent.to_str().unwrap()]));
    let out = clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        patch.to_str().unwrap(),
        "--no-decision",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let chain = stdout(&clan(&["read", "chain", parent.to_str().unwrap()]));
    assert!(
        !chain.contains("unknown-agent"),
        "F1: no placeholder decision: {chain}"
    );
    assert_eq!(before, chain, "F15: --no-decision must not touch the chain");
}

#[test]
fn patch_data_requires_attribution_by_default() {
    // F15: a shared patch-data without --agent/--action (and without
    // --no-decision) is a teaching error that names the flags.
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let before = std::fs::read(&parent).unwrap();
    let out = clan(&["patch-data", parent.to_str().unwrap(), r#"{"x":1}"#]);
    assert!(!out.status.success(), "missing attribution must fail");
    let err = stderr(&out);
    assert!(
        err.contains("--agent") && err.contains("--action"),
        "error must teach the flags: {err}"
    );
    assert!(
        err.contains("--no-decision"),
        "error must name the opt-out: {err}"
    );
    assert_eq!(
        before,
        std::fs::read(&parent).unwrap(),
        "rejected write must not touch the file"
    );

    // With attribution: succeeds and records the decision with exact fields_changed.
    let out = clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        r#"{"vendor":"Acme","total":5}"#,
        "--agent",
        "pricing",
        "--action",
        "set vendor and total",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let chain = stdout(&clan(&["read", "chain", parent.to_str().unwrap()]));
    assert!(
        chain.contains("pricing") && chain.contains("set vendor and total"),
        "{chain}"
    );
    // fields_changed lists exactly the patched keys, not the whole document.
    assert!(
        chain.contains("total") && chain.contains("vendor"),
        "{chain}"
    );
    assert!(
        !chain.contains("client_name"),
        "fields_changed must be only the patched keys: {chain}"
    );
}

#[test]
fn patch_data_inline_json_and_set() {
    // F13: inline JSON string (no temp file) and --set scalars.
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    // Inline JSON.
    let out = clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        r#"{"vendor":"Acme"}"#,
        "--no-decision",
    ]);
    assert!(
        out.status.success(),
        "inline JSON must work: {}",
        stderr(&out)
    );
    // --set scalars (typed: number stays a number, bare word a string).
    let out = clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        "--set",
        "seats=40",
        "--set",
        "tier=pro",
        "--no-decision",
    ]);
    assert!(out.status.success(), "--set must work: {}", stderr(&out));
    let data = stdout(&clan(&["read", "data", parent.to_str().unwrap()]));
    assert!(data.contains("Acme"), "{data}");
    assert!(
        data.contains("seats: 40"),
        "numeric --set must stay numeric: {data}"
    );
    assert!(data.contains("pro"), "{data}");

    // Neither a patch nor --set → teaching error.
    let out = clan(&["patch-data", parent.to_str().unwrap(), "--no-decision"]);
    assert!(!out.status.success(), "no patch source must fail");
    assert!(
        stderr(&out).contains("--set") || stderr(&out).contains("inline"),
        "{}",
        stderr(&out)
    );
}

#[test]
fn patch_data_append_concatenates_array() {
    // F14: --append <key> concatenates instead of RFC-7396 replace.
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    // Seed an array.
    clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        r#"{"tags":["a","b"]}"#,
        "--no-decision",
    ]);
    // Append (would replace without --append).
    let out = clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        r#"{"tags":["c"]}"#,
        "--append",
        "tags",
        "--no-decision",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let data = stdout(&clan(&["read", "data", parent.to_str().unwrap()]));
    assert!(
        data.contains('a') && data.contains('b') && data.contains('c'),
        "all three kept: {data}"
    );

    // Control: without --append the array is replaced.
    clan(&[
        "patch-data",
        parent.to_str().unwrap(),
        r#"{"tags":["only"]}"#,
        "--no-decision",
    ]);
    let data = stdout(&clan(&["read", "data", parent.to_str().unwrap()]));
    assert!(
        data.contains("only") && !data.contains("\"c\""),
        "replace is still the default: {data}"
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

    let out = clan(&[
        "patch-html",
        parent.to_str().unwrap(),
        patch.to_str().unwrap(),
        "--agent",
        "designer",
        "--action",
        "added a section",
    ]);
    assert!(out.status.success(), "patch-html failed: {}", stderr(&out));

    let read = clan(&["read", "human", parent.to_str().unwrap()]);
    assert!(String::from_utf8_lossy(&read.stdout).contains("id=\"added\""));
    // F15: the view change is attributed in the chain.
    let chain = stdout(&clan(&["read", "chain", parent.to_str().unwrap()]));
    assert!(
        chain.contains("designer") && chain.contains("added a section"),
        "{chain}"
    );
}

#[test]
fn patch_html_requires_attribution_by_default() {
    // F15: a view change without --agent/--action and without a frontmatter
    // decision is a teaching error.
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let patch = dir.path().join("p.html");
    std::fs::write(&patch,
        "---\nmode: patch-html\npatch_selector: \"section\"\npatch_action: \"append\"\n---\n<div id=\"x\">y</div>").unwrap();
    let before = std::fs::read(&parent).unwrap();

    let out = clan(&[
        "patch-html",
        parent.to_str().unwrap(),
        patch.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "missing attribution must fail");
    let err = stderr(&out);
    assert!(
        err.contains("--agent") && err.contains("--no-decision"),
        "error must teach the options: {err}"
    );
    assert_eq!(
        before,
        std::fs::read(&parent).unwrap(),
        "rejected patch must not touch the file"
    );

    // A frontmatter decision satisfies the requirement without flags.
    let patch2 = dir.path().join("p2.html");
    std::fs::write(&patch2,
        "---\nmode: patch-html\npatch_selector: \"section\"\npatch_action: \"append\"\ndecision:\n  agent: designer\n  action: added via frontmatter\n---\n<div id=\"z\">y</div>").unwrap();
    let out = clan(&[
        "patch-html",
        parent.to_str().unwrap(),
        patch2.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "frontmatter decision must satisfy attribution: {}",
        stderr(&out)
    );
    let chain = stdout(&clan(&["read", "chain", parent.to_str().unwrap()]));
    assert!(chain.contains("added via frontmatter"), "{chain}");
}

// --- Multi-agent edge-case CLI tests ---

/// Three agents fork from same parent, each write unique keys, merge → 0 conflicts.
#[test]
fn three_agents_fork_merge_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "3-Agent",
        "--brief",
        "b",
        "--no-render",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    let out = clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "alpha,beta,gamma",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "fork 3 agents: {}", stderr(&out));

    for (agent, key, val) in [
        ("alpha", "a_key", "a_val"),
        ("beta", "b_key", "b_val"),
        ("gamma", "c_key", "c_val"),
    ] {
        let patch = dir.path().join(format!("{agent}.json"));
        std::fs::write(&patch, format!(r#"{{"{key}": "{val}"}}"#)).unwrap();
        let branch = branches.join(format!("{agent}.clan"));
        let out = clan(&[
            "patch-data",
            branch.to_str().unwrap(),
            patch.to_str().unwrap(),
            "--namespace",
        ]);
        assert!(out.status.success(), "{agent} write: {}", stderr(&out));
    }

    let merged = dir.path().join("merged.clan");
    let out = clan(&[
        "merge",
        branches.join("alpha.clan").to_str().unwrap(),
        branches.join("beta.clan").to_str().unwrap(),
        branches.join("gamma.clan").to_str().unwrap(),
        "--output",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "3-agent merge: {}", stderr(&out));

    let report = stdout(&clan(&["read", "report", merged.to_str().unwrap()]));
    assert!(
        report.contains("unresolved: 0"),
        "no conflicts expected: {report}"
    );

    let data = stdout(&clan(&["read", "data", merged.to_str().unwrap()]));
    assert!(
        data.contains("a_val") && data.contains("b_val") && data.contains("c_val"),
        "{data}"
    );

    assert!(clan(&["validate", merged.to_str().unwrap()])
        .status
        .success());
}

/// Fork 3 branches, merge only 2: the third is left unmerged.
#[test]
fn partial_merge_two_of_three_branches() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "Partial",
        "--brief",
        "b",
        "--no-render",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "r1,r2,r3",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);

    for (agent, val) in [("r1", "rv1"), ("r2", "rv2")] {
        let patch = dir.path().join(format!("{agent}.json"));
        std::fs::write(&patch, format!(r#"{{"{agent}_key": "{val}"}}"#)).unwrap();
        clan(&[
            "patch-data",
            branches.join(format!("{agent}.clan")).to_str().unwrap(),
            patch.to_str().unwrap(),
            "--namespace",
        ]);
    }

    // Merge only r1 + r2, intentionally omitting r3.
    let merged = dir.path().join("partial.clan");
    let out = clan(&[
        "merge",
        branches.join("r1.clan").to_str().unwrap(),
        branches.join("r2.clan").to_str().unwrap(),
        "--output",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "partial merge: {}", stderr(&out));
    let data = stdout(&clan(&["read", "data", merged.to_str().unwrap()]));
    assert!(data.contains("rv1") && data.contains("rv2"), "{data}");
    assert!(!data.contains("r3_key"), "r3 not merged: {data}");
}

/// All agents write the same value for the same key: no conflict produced.
#[test]
fn all_agents_agree_produces_no_conflict() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "Agree",
        "--brief",
        "b",
        "--no-render",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "a,b,c",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);

    for agent in ["a", "b", "c"] {
        let patch = dir.path().join(format!("{agent}.json"));
        std::fs::write(&patch, r#"{"verdict": "approved"}"#).unwrap();
        clan(&[
            "patch-data",
            branches.join(format!("{agent}.clan")).to_str().unwrap(),
            patch.to_str().unwrap(),
            "--namespace",
        ]);
    }

    let merged = dir.path().join("agreed.clan");
    clan(&[
        "merge",
        branches.join("a.clan").to_str().unwrap(),
        branches.join("b.clan").to_str().unwrap(),
        branches.join("c.clan").to_str().unwrap(),
        "--output",
        merged.to_str().unwrap(),
    ]);

    let report = stdout(&clan(&["read", "report", merged.to_str().unwrap()]));
    assert!(
        report.contains("unresolved: 0"),
        "all-agree must yield 0 conflicts: {report}"
    );
}

/// Settle two contested keys one at a time; unresolved decrements correctly.
#[test]
fn multi_key_adjudication_decrements_incrementally() {
    let dir = tempfile::tempdir().unwrap();
    let merged = forked_merged(dir.path()); // findings=append, status=last-write → 1 conflict

    // Add a second contested key via a fresh fork/merge with two keys conflicting.
    // Instead, create a separate merge with 2 contested keys.
    let root = dir.path().join("root2.clan");
    clan(&[
        "create",
        "--title",
        "Multi",
        "--brief",
        "b",
        "--no-render",
        "--output",
        root.to_str().unwrap(),
    ]);
    let br = dir.path().join("br");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "p,q",
        "--output-dir",
        br.to_str().unwrap(),
    ]);

    let p_patch = dir.path().join("p.json");
    std::fs::write(&p_patch, r#"{"k1": "p1", "k2": "p2"}"#).unwrap();
    clan(&[
        "patch-data",
        br.join("p.clan").to_str().unwrap(),
        p_patch.to_str().unwrap(),
        "--namespace",
    ]);

    let q_patch = dir.path().join("q.json");
    std::fs::write(&q_patch, r#"{"k1": "q1", "k2": "q2"}"#).unwrap();
    clan(&[
        "patch-data",
        br.join("q.clan").to_str().unwrap(),
        q_patch.to_str().unwrap(),
        "--namespace",
    ]);

    let m = dir.path().join("m2.clan");
    clan(&[
        "merge",
        br.join("p.clan").to_str().unwrap(),
        br.join("q.clan").to_str().unwrap(),
        "--output",
        m.to_str().unwrap(),
    ]);

    let report = stdout(&clan(&["read", "report", m.to_str().unwrap()]));
    assert!(
        report.contains("unresolved: 2"),
        "expect 2 conflicts: {report}"
    );

    // Settle k1 (with attribution — F15).
    clan(&[
        "patch-data",
        m.to_str().unwrap(),
        r#"{"k1": "settled"}"#,
        "--agent",
        "synth",
        "--action",
        "adjudicated k1",
    ]);
    let report = stdout(&clan(&["read", "report", m.to_str().unwrap()]));
    assert!(
        report.contains("unresolved: 1"),
        "after 1st settle: {report}"
    );

    // Settle k2.
    clan(&[
        "patch-data",
        m.to_str().unwrap(),
        r#"{"k2": "done"}"#,
        "--agent",
        "synth",
        "--action",
        "adjudicated k2",
    ]);
    let report = stdout(&clan(&["read", "report", m.to_str().unwrap()]));
    assert!(report.contains("unresolved: 0"), "all settled: {report}");
}

/// pack-html on a forked branch file is rejected with a teaching error.
#[test]
fn pack_html_on_forked_branch_is_teaching_error() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "b",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "a,b",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);

    let html = dir.path().join("view.html");
    std::fs::write(&html, "<!DOCTYPE html><html><body><p>hi</p></body></html>").unwrap();
    let branch = branches.join("a.clan");
    let before = std::fs::read(&branch).unwrap();

    let out = clan(&[
        "pack-html",
        branch.to_str().unwrap(),
        html.to_str().unwrap(),
        "--output",
        dir.path().join("out.clan").to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "pack-html on branch must fail");
    let err = stderr(&out);
    assert!(
        err.contains("--namespace") || err.contains("forked") || err.contains("merge"),
        "error must teach the correct path: {err}"
    );
    assert_eq!(
        before,
        std::fs::read(&branch).unwrap(),
        "branch file must be untouched"
    );
}

/// F17: pack-html --agent/--action records an inline attribution entry.
#[test]
fn pack_html_attribution_records_decision() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let html = dir.path().join("view.html");
    std::fs::write(&html, "<!DOCTYPE html><html><body><p>v2</p></body></html>").unwrap();
    let next = dir.path().join("next.clan");

    let out = clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        html.to_str().unwrap(),
        "--output",
        next.to_str().unwrap(),
        "--agent",
        "design-bot",
        "--action",
        "refresh layout",
    ]);
    assert!(
        out.status.success(),
        "pack-html with attribution failed: {}",
        stderr(&out)
    );

    let chain = clan(&["read", "chain", next.to_str().unwrap()]);
    let chain_out = String::from_utf8_lossy(&chain.stdout).into_owned();
    assert!(
        chain_out.contains("design-bot"),
        "agent not in chain: {chain_out}"
    );
    assert!(
        chain_out.contains("refresh layout"),
        "action not in chain: {chain_out}"
    );
    assert!(
        chain_out.contains("human/index.html"),
        "fields_changed missing: {chain_out}"
    );
}

/// F17: --agent without --action (or vice versa) is an error.
#[test]
fn pack_html_partial_attribution_is_error() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());
    let html = dir.path().join("view.html");
    std::fs::write(&html, "<!DOCTYPE html><html><body><p>x</p></body></html>").unwrap();
    let next = dir.path().join("next.clan");

    let out = clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        html.to_str().unwrap(),
        "--output",
        next.to_str().unwrap(),
        "--agent",
        "bot",
        // --action intentionally omitted
    ]);
    assert!(!out.status.success(), "partial attribution must fail");
    assert!(
        stderr(&out).contains("--action"),
        "error must name the missing flag: {}",
        stderr(&out)
    );
}

/// F18: pack-html with structured: data + unchanged view is blocked without --force.
#[test]
fn pack_html_data_only_write_is_blocked() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    // Seed a view so the parent has human/index.html.
    let seed_html = dir.path().join("seed.html");
    std::fs::write(
        &seed_html,
        "<!DOCTYPE html><html><body><p>seed</p></body></html>",
    )
    .unwrap();
    let seeded = dir.path().join("seeded.clan");
    clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        seed_html.to_str().unwrap(),
        "--output",
        seeded.to_str().unwrap(),
    ]);

    // Now try to write structured data without changing the view.
    let data_patch = dir.path().join("patch.html");
    std::fs::write(
        &data_patch,
        "---\nstructured:\n  price: 99\n---\n<!DOCTYPE html><html><body><p>seed</p></body></html>",
    )
    .unwrap();
    let next = dir.path().join("next.clan");
    let out = clan(&[
        "pack-html",
        seeded.to_str().unwrap(),
        data_patch.to_str().unwrap(),
        "--output",
        next.to_str().unwrap(),
    ]);
    assert!(!out.status.success(), "data-only write must be blocked");
    let err = stderr(&out);
    assert!(
        err.contains("patch-data") && err.contains("--force"),
        "error must teach patch-data and --force: {err}"
    );
}

/// F18: --force bypasses the guard and emits a tiny note.
#[test]
fn pack_html_data_only_write_with_force_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    let seed_html = dir.path().join("seed.html");
    std::fs::write(
        &seed_html,
        "<!DOCTYPE html><html><body><p>seed</p></body></html>",
    )
    .unwrap();
    let seeded = dir.path().join("seeded.clan");
    clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        seed_html.to_str().unwrap(),
        "--output",
        seeded.to_str().unwrap(),
    ]);

    let data_patch = dir.path().join("patch.html");
    std::fs::write(
        &data_patch,
        "---\nstructured:\n  price: 99\n---\n<!DOCTYPE html><html><body><p>seed</p></body></html>",
    )
    .unwrap();
    let next = dir.path().join("next.clan");
    let out = clan(&[
        "pack-html",
        seeded.to_str().unwrap(),
        data_patch.to_str().unwrap(),
        "--output",
        next.to_str().unwrap(),
        "--force",
    ]);
    assert!(
        out.status.success(),
        "--force must allow the write: {}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(
        err.contains("note:"),
        "must emit a tiny note even with --force: {err}"
    );
    assert!(next.exists());
}

/// F18: pack-html with structured: + genuinely changed view emits a hint but succeeds.
#[test]
fn pack_html_structured_with_changed_view_hints_but_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    let seed_html = dir.path().join("seed.html");
    std::fs::write(
        &seed_html,
        "<!DOCTYPE html><html><body><p>old</p></body></html>",
    )
    .unwrap();
    let seeded = dir.path().join("seeded.clan");
    clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        seed_html.to_str().unwrap(),
        "--output",
        seeded.to_str().unwrap(),
    ]);

    // View is genuinely different — should succeed with a hint on stderr.
    let combo = dir.path().join("combo.html");
    std::fs::write(
        &combo,
        "---\nstructured:\n  price: 99\n---\n<!DOCTYPE html><html><body><p>new</p></body></html>",
    )
    .unwrap();
    let next = dir.path().join("next.clan");
    let out = clan(&[
        "pack-html",
        seeded.to_str().unwrap(),
        combo.to_str().unwrap(),
        "--output",
        next.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "combined view+data write must succeed: {}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("patch-data"),
        "must emit the hint: {}",
        stderr(&out)
    );
    assert!(next.exists());
}

/// Run the binary with hints explicitly enabled (CLAN_NO_HINTS="").
fn clan_hints(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_clan"))
        .args(args)
        .env("CLAN_NO_BANNER", "1")
        .env("CLAN_NO_HINTS", "")
        .output()
        .expect("failed to run clan binary")
}

/// F2b: patching a key that is {{bound}} in the HTML view suppresses the stale hint.
#[test]
fn patch_data_bound_key_suppresses_stale_hint() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    // Seed a view that binds {{price}}.
    let html = dir.path().join("bound.html");
    std::fs::write(
        &html,
        "<!DOCTYPE html><html><body><p>Price: {{price}}</p></body></html>",
    )
    .unwrap();
    let seeded = dir.path().join("seeded.clan");
    clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        html.to_str().unwrap(),
        "--output",
        seeded.to_str().unwrap(),
    ]);

    // Patch only the bound key — view will auto-render, no stale hint.
    let out = clan_hints(&[
        "patch-data",
        seeded.to_str().unwrap(),
        r#"{"price":99}"#,
        "--agent",
        "test-bot",
        "--action",
        "update price",
    ]);
    assert!(out.status.success());
    assert!(
        !stderr(&out).contains("stale"),
        "stale hint must be suppressed for bound key: {}",
        stderr(&out)
    );
}

/// F2b: patching an unbound key names it in the hint rather than emitting a generic stale message.
#[test]
fn patch_data_unbound_key_names_orphan_in_hint() {
    let dir = tempfile::tempdir().unwrap();
    let parent = create_parent(dir.path());

    // Seed a view that binds {{price}} but NOT {{discount}}.
    let html = dir.path().join("bound.html");
    std::fs::write(
        &html,
        "<!DOCTYPE html><html><body><p>Price: {{price}}</p></body></html>",
    )
    .unwrap();
    let seeded = dir.path().join("seeded.clan");
    clan(&[
        "pack-html",
        parent.to_str().unwrap(),
        html.to_str().unwrap(),
        "--output",
        seeded.to_str().unwrap(),
    ]);

    // Patch an unbound key — hint must name it, not give a generic stale message.
    let out = clan_hints(&[
        "patch-data",
        seeded.to_str().unwrap(),
        r#"{"discount":5}"#,
        "--agent",
        "test-bot",
        "--action",
        "add discount",
    ]);
    assert!(out.status.success());
    let err = stderr(&out);
    assert!(
        err.contains("discount"),
        "hint must name the unbound key: {err}"
    );
    assert!(
        !err.contains("stale — `clan render"),
        "must not emit generic stale hint: {err}"
    );
}

/// After merging, the resulting file can be forked again for a second parallel pass.
#[test]
fn fork_merge_then_fork_again_works() {
    let dir = tempfile::tempdir().unwrap();
    let merged = forked_merged(dir.path());

    // Settle the conflict first so merged is clean.
    let adj = dir.path().join("adj.json");
    std::fs::write(&adj, r#"{"status": "final"}"#).unwrap();
    clan(&[
        "patch-data",
        merged.to_str().unwrap(),
        adj.to_str().unwrap(),
        "--agent",
        "synth",
        "--action",
        "finalised status",
    ]);

    // Fork the merged result for a second parallel pass.
    let round2 = dir.path().join("round2");
    let out = clan(&[
        "fork",
        merged.to_str().unwrap(),
        "--agents",
        "writer,reviewer",
        "--output-dir",
        round2.to_str().unwrap(),
    ]);
    assert!(
        out.status.success(),
        "re-fork after merge must work: {}",
        stderr(&out)
    );
    assert!(round2.join("writer.clan").exists());
    assert!(round2.join("reviewer.clan").exists());

    // Both are valid forks.
    assert!(
        clan(&["validate", round2.join("writer.clan").to_str().unwrap()])
            .status
            .success()
    );
}

/// Merged file passes full validation.
#[test]
fn merged_file_validates_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let merged = forked_merged(dir.path());
    let out = clan(&["validate", merged.to_str().unwrap()]);
    assert!(
        out.status.success(),
        "merged file must validate: {}",
        stderr(&out)
    );
}

/// --skip-guide on a forked branch: branch mode block still injected despite guide being omitted.
#[test]
fn skip_guide_on_forked_file_still_injects_branch_block() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "T",
        "--brief",
        "b",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "a,b",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);

    let out = clan(&[
        "read",
        "agent",
        branches.join("a.clan").to_str().unwrap(),
        "--skip-guide",
    ]);
    assert!(out.status.success(), "{}", stderr(&out));
    let ctx = stdout(&out);
    assert!(
        ctx.contains("guide body skipped"),
        "guide must be skipped: {ctx}"
    );
    assert!(
        ctx.contains("# Branch Mode"),
        "branch block must still inject: {ctx}"
    );
    assert!(ctx.contains("agents/a/"), "{ctx}");
}

/// Five-agent fork/merge stress: all 5 branches write to the same key, last-write wins.
#[test]
fn five_agent_last_write_stress() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("root.clan");
    clan(&[
        "create",
        "--title",
        "5Agent",
        "--brief",
        "b",
        "--no-render",
        "--output",
        root.to_str().unwrap(),
    ]);
    let branches = dir.path().join("branches");
    clan(&[
        "fork",
        root.to_str().unwrap(),
        "--agents",
        "a0,a1,a2,a3,a4",
        "--output-dir",
        branches.to_str().unwrap(),
    ]);

    for i in 0..5usize {
        let patch = dir.path().join(format!("p{i}.json"));
        std::fs::write(&patch, format!(r#"{{"shared_key": "from_agent_{i}"}}"#)).unwrap();
        let b = branches.join(format!("a{i}.clan"));
        clan(&[
            "patch-data",
            b.to_str().unwrap(),
            patch.to_str().unwrap(),
            "--namespace",
        ]);
    }

    let merged = dir.path().join("merged5.clan");
    let out = clan(&[
        "merge",
        branches.join("a0.clan").to_str().unwrap(),
        branches.join("a1.clan").to_str().unwrap(),
        branches.join("a2.clan").to_str().unwrap(),
        branches.join("a3.clan").to_str().unwrap(),
        branches.join("a4.clan").to_str().unwrap(),
        "--output",
        merged.to_str().unwrap(),
    ]);
    assert!(out.status.success(), "5-agent merge: {}", stderr(&out));

    // last-write: the last listed agent (a4) wins.
    let data = stdout(&clan(&["read", "data", merged.to_str().unwrap()]));
    assert!(
        data.contains("from_agent_4"),
        "last-write winner must be a4: {data}"
    );

    let report = stdout(&clan(&["read", "report", merged.to_str().unwrap()]));
    assert!(
        report.contains("unresolved: 1"),
        "one conflict (shared_key): {report}"
    );
    assert!(clan(&["validate", merged.to_str().unwrap()])
        .status
        .success());
}
