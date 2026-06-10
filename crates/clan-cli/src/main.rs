// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! `clan` — command-line tool for CLAN files.
//!
//! Usage:
//!   clan create --title "My Doc" --brief "…" [--type invoice] --output <output.clan>
//!   clan validate <file.clan>
//!   clan read agent <file.clan>
//!   clan read human <file.clan>
//!   clan read data <file.clan>
//!   clan info <file.clan>
//!   clan edit <file.clan>
//!   clan patch-html --output <next.clan> <parent.clan> <patch.html>
//!   clan pack --output <next.clan> [--delta "…"] <parent.clan> <output.json>
//!   clan pack-html --output <next.clan> [--assets <dir>] [--delta "…"] <parent.clan> <output.html>
//!   clan export-static <file.clan> [--output static.json]
//!   clan agent-help

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clan_sdk::{
    assemble, create, export_static, fork, merge, pack, pack_html, patch_data,
    patch_data_namespaced, patch_decision, patch_state, patch_context, patch_asset, render,
    validate, AgentOutput, ClanFile, CreateOptions, InjectOptions, MergeOptions, MergePolicies,
    PackOptions, DecisionEntry, MERGE_REPORT_PATH,
};

#[derive(Parser)]
#[command(
    name = "clan",
    about = "CLAN — Context and Live Agent Notation. Read, write, validate, and pipeline .clan files.",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    /// Suppress `next:` teaching hints (spec §27). CLAN_NO_HINTS=1 does the same.
    #[arg(long, global = true)]
    quiet: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new .clan file from a title and brief.
    Create {
        /// Document title.
        #[arg(long)]
        title: String,
        /// Initial brief / task description for the first agent.
        #[arg(long)]
        brief: String,
        /// Optional document type tag (e.g. invoice, report).
        #[arg(long = "doc-type")]
        doc_type: Option<String>,
        /// Output path for the new .clan file.
        #[arg(long, value_name = "PATH")]
        output: Option<PathBuf>,
        /// Agent-only file: skip the human view (spec §23). Any later hop can
        /// materialise it with `clan render`.
        #[arg(long)]
        no_render: bool,
        /// Deprecated: positional output path. Use --output instead.
        #[arg(value_name = "OUTPUT", conflicts_with = "output", hide = true)]
        positional_output: Option<PathBuf>,
    },
    /// Fork a file into one branch per agent for parallel work (spec §24.1).
    /// Each agent writes only inside its own `agents/<id>/` namespace.
    Fork {
        /// Parent .clan file.
        parent: PathBuf,
        /// Comma-separated agent ids (e.g. researcher,analyst,critic).
        #[arg(long, value_delimiter = ',', required = true)]
        agents: Vec<String>,
        /// Directory to write `<agent>.clan` branch files into.
        #[arg(long, value_name = "DIR")]
        output_dir: PathBuf,
    },
    /// Join fork branches into one merged file (spec §24.3): a deterministic
    /// per-key fold — conflicts land in merge-report.yaml, not failures.
    Merge {
        /// Branch .clan files (2 or more), folded in argument order.
        #[arg(required = true, num_args = 2..)]
        branches: Vec<PathBuf>,
        /// Output path for the merged .clan file.
        #[arg(long)]
        output: PathBuf,
        /// Per-key policy overrides, e.g. --policy findings=append --policy score=max.
        /// Policies: last-write (default), append, max, min, agent-priority.
        #[arg(long = "policy", value_name = "KEY=POLICY")]
        policies: Vec<String>,
        /// Drop the agents/<id>/ namespaces instead of keeping them as provenance.
        #[arg(long)]
        prune_namespaces: bool,
        /// Human-readable description of the merge.
        #[arg(long)]
        delta: Option<String>,
    },
    /// Materialise the human view from the structured members (spec §23).
    Render {
        /// The .clan file to render in-place.
        file: PathBuf,
    },
    /// Validate a .clan file and print a report.
    Validate {
        /// Path to the .clan file.
        file: PathBuf,
        /// Exit with non-zero status if content checks also fail.
        #[arg(long)]
        strict: bool,
    },
    /// Read a section of a .clan file.
    Read {
        #[command(subcommand)]
        section: ReadSection,
    },
    /// Show manifest metadata.
    Info {
        file: PathBuf,
    },
    /// Pack a new .clan from a parent file and agent JSON output.
    Pack {
        /// Parent .clan file.
        parent: PathBuf,
        /// Agent output JSON file (or `-` for stdin).
        output_json: String,
        /// Output path for the new .clan file.
        #[arg(long)]
        output: PathBuf,
        /// Optional new JSON schema to override the parent's schema.
        #[arg(long)]
        schema: Option<PathBuf>,
        /// Human-readable description of what changed.
        #[arg(long)]
        delta: Option<String>,
    },
    /// Interactively edit a .clan file's data and UI in your default $EDITOR.
    Edit {
        /// The .clan file to edit in-place.
        file: PathBuf,
    },
    /// Apply an HTML patch in-place without creating intermediate files.
    /// The patch file should contain YAML frontmatter with `mode: patch-html`.
    PatchHtml {
        /// The .clan file to edit in-place.
        file: PathBuf,
        /// HTML patch file (or `-` for stdin).
        html_file: String,
        /// Human-readable description of what changed.
        #[arg(long)]
        delta: Option<String>,
    },
    /// Surgically patch `shared/data.yaml` inside a .clan file using JSON Merge Patch (RFC 7396).
    PatchData {
        /// The .clan file to edit in-place.
        file: PathBuf,
        /// JSON patch file (or `-` for stdin).
        json_file: String,
        /// On a forked branch file: write into your `agents/<id>/` namespace
        /// instead of the (locked) shared data (spec §24.1).
        #[arg(long)]
        namespace: bool,
    },
    /// Surgically patch `agent/output-schema.json` inside a .clan file.
    PatchSchema {
        /// The .clan file to edit in-place.
        file: PathBuf,
        /// JSON Schema file (or `-` for stdin).
        schema_file: String,
    },
    /// Append a new decision entry to `shared/decision-chain.yaml` inside a .clan file.
    PatchDecision {
        /// The .clan file to edit in-place.
        file: PathBuf,
        /// Name of the agent making the decision.
        #[arg(long)]
        agent: String,
        /// Short description of the action taken.
        #[arg(long)]
        action: String,
        /// Detailed rationale for the decision.
        #[arg(long)]
        rationale: String,
        /// Pin this decision to ensure it remains highly visible.
        #[arg(long)]
        pinned: bool,
    },
    /// Surgically patch `agent/state.yaml` inside a .clan file using JSON Merge Patch (RFC 7396).
    PatchState {
        /// The .clan file to edit in-place.
        file: PathBuf,
        /// JSON patch file (or `-` for stdin).
        json_file: String,
    },
    /// Overwrite or append to `agent/context.md` inside a .clan file.
    PatchContext {
        /// The .clan file to edit in-place.
        file: PathBuf,
        /// Markdown file containing the context (or `-` for stdin).
        markdown_file: String,
        /// Append rather than overwrite.
        #[arg(long)]
        append: bool,
    },
    /// Inject or replace an asset (e.g. image) in `human/assets/` inside a .clan file.
    PatchAsset {
        /// The .clan file to edit in-place.
        file: PathBuf,
        /// Internal path within `human/assets/` (e.g. `logo.png`).
        internal_path: String,
        /// Local file containing the asset.
        local_file: PathBuf,
    },

    /// Export a .clan as a single JSON blob for SDK-less agents.
    ExportStatic {
        file: PathBuf,
        /// Output JSON file (defaults to stdout).
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Pack a new .clan directly from an HTML file — no JSON encoding needed.
    /// The HTML file may contain an optional YAML frontmatter block (between --- markers)
    /// at the very top to supply structured data and a decision entry.
    PackHtml {
        /// Parent .clan file.
        parent: PathBuf,
        /// HTML file to pack (or `-` for stdin).
        html_file: String,
        /// Output path for the new .clan file.
        #[arg(long)]
        output: PathBuf,
        /// Optional directory containing binary/text assets to mount.
        #[arg(long)]
        assets: Option<PathBuf>,
        /// Optional new JSON schema to override the parent's schema.
        #[arg(long)]
        schema: Option<PathBuf>,
        /// Human-readable description of what changed.
        #[arg(long)]
        delta: Option<String>,
    },
    /// Print a compact agent-oriented quick reference (< 200 tokens).
    /// Use this instead of --help when operating as an AI agent.
    AgentHelp,
}

#[derive(Subcommand)]
enum ReadSection {
    /// Print the assembled agent context (TOON-encoded).
    Agent {
        file: PathBuf,
        #[arg(long)]
        no_patches: bool,
        /// Skip the agent guide body (for agents that already read it);
        /// prints a one-line note with the guide digest instead.
        #[arg(long)]
        skip_guide: bool,
    },
    /// Print human/index.html.
    Human { file: PathBuf },
    /// Print shared/data.yaml.
    Data { file: PathBuf },
    /// Print agent/decision-chain.yaml.
    Chain { file: PathBuf },
    /// Print merge-report.yaml (contested keys from the last merge), TOON-encoded.
    Report { file: PathBuf },
}

/// ASCII banner shown once, on the first ever run of the CLI (#31).
const WELCOME_BANNER: &str = r#"
   _____ _        _    _   _
  / ____| |      / \  | \ | |
 | |    | |     / _ \ |  \| |
 | |____| |___ / ___ \| |\  |
  \_____|_____/_/   \_\_| \_|
"#;

/// Resolve the directory holding the first-run marker:
/// $CLAN_CONFIG_DIR, then $XDG_CONFIG_HOME/clan, then ~/.config/clan.
fn banner_config_dir() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("CLAN_CONFIG_DIR") {
        return Some(PathBuf::from(dir));
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("clan"));
    }
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(|home| PathBuf::from(home).join(".config").join("clan"))
}

/// Print the welcome banner on the very first run, then never again.
/// stderr only — stdout belongs to command output and is often piped.
/// CLAN_NO_BANNER=1 disables it entirely (no marker is written either);
/// an unwritable config dir silently skips the banner.
fn maybe_show_welcome_banner() {
    if std::env::var_os("CLAN_NO_BANNER").is_some() {
        return;
    }
    let Some(dir) = banner_config_dir() else { return };
    let marker = dir.join("welcomed");
    if marker.exists() {
        return;
    }
    let _ = std::fs::create_dir_all(&dir);
    // Only greet if the marker sticks, so a read-only config dir doesn't
    // produce the banner on every run.
    if std::fs::write(&marker, env!("CARGO_PKG_VERSION")).is_err() {
        return;
    }
    eprintln!(
        "{WELCOME_BANNER}\n  CLAN v{} — Context and Live Agent Notation\n  Documents that carry their own context. Run `clan agent-help` to get started.\n",
        env!("CARGO_PKG_VERSION")
    );
}

fn main() -> Result<()> {
    maybe_show_welcome_banner();
    let cli = Cli::parse();
    let hints = Hints::new(cli.quiet);
    match cli.command {
        Commands::Create {
            title,
            brief,
            doc_type,
            output,
            no_render,
            positional_output,
        } => {
            let output = match (output, positional_output) {
                (Some(path), _) => path,
                (None, Some(path)) => {
                    eprintln!(
                        "warning: the positional output path is deprecated; use --output <PATH>"
                    );
                    path
                }
                (None, None) => anyhow::bail!("missing output path: pass --output <PATH>"),
            };
            cmd_create(title, brief, doc_type, output, no_render, &hints)
        }
        Commands::Fork { parent, agents, output_dir } => cmd_fork(parent, agents, output_dir, &hints),
        Commands::Merge { branches, output, policies, prune_namespaces, delta } => {
            cmd_merge(branches, output, policies, prune_namespaces, delta, &hints)
        }
        Commands::Render { file } => cmd_render(file, &hints),
        Commands::Validate { file, strict } => cmd_validate(file, strict, &hints),
        Commands::Read { section } => cmd_read(section),
        Commands::Info { file } => cmd_info(file),
        Commands::Pack {
            parent,
            output_json,
            output,
            schema,
            delta,
        } => cmd_pack(parent, output_json, output, schema, delta, &hints),
        Commands::Edit { file } => cmd_edit(file),
        Commands::PatchHtml { file, html_file, delta } => cmd_patch_html(file, html_file, delta),
        Commands::PatchData { file, json_file, namespace } => cmd_patch_data(file, json_file, namespace, &hints),
        Commands::PatchSchema { file, schema_file } => cmd_patch_schema(file, schema_file),
        Commands::PatchDecision { file, agent, action, rationale, pinned } => cmd_patch_decision(file, agent, action, rationale, pinned, &hints),
        Commands::PatchState { file, json_file } => cmd_patch_state(file, json_file),
        Commands::PatchContext { file, markdown_file, append } => cmd_patch_context(file, markdown_file, append),
        Commands::PatchAsset { file, internal_path, local_file } => cmd_patch_asset(file, internal_path, local_file),
        Commands::ExportStatic { file, output } => cmd_export_static(file, output),
        Commands::PackHtml { parent, html_file, output, assets, schema, delta } => {
            cmd_pack_html(parent, html_file, output, assets, schema, delta, &hints)
        }
        Commands::AgentHelp => { cmd_agent_help(); Ok(()) }
    }
}

/// Teaching hints (spec §27): deterministic, bounded (≤3 lines), and
/// precondition-gated — a hint never mentions a capability whose
/// preconditions are absent from the file's state. Emitted on the diagnostic
/// stream (stderr) so stdout stays pure data; suppressed by --quiet or
/// CLAN_NO_HINTS=1.
struct Hints {
    enabled: bool,
}

impl Hints {
    fn new(quiet: bool) -> Self {
        Self {
            enabled: !quiet && std::env::var_os("CLAN_NO_HINTS").is_none(),
        }
    }

    fn emit<S: AsRef<str>>(&self, lines: &[S]) {
        if !self.enabled {
            return;
        }
        for line in lines.iter().take(3) {
            eprintln!("next: {}", line.as_ref());
        }
    }
}

/// Hints derived purely from a written file's state (spec §27.1): each line's
/// precondition is the manifest/member state that makes it actionable.
fn file_state_hints(clan: &ClanFile, path: &std::path::Path) -> Vec<String> {
    let mut hints = Vec::new();
    let m = clan.manifest();
    let p = path.display();

    if let Some(fork) = &m.fork {
        hints.push(format!(
            "you are branch agent '{}'; write only inside {} (patch-data --namespace, patch-decision auto-routes)",
            fork.agent_id, fork.namespace
        ));
    }
    if clan.has_entry(MERGE_REPORT_PATH) {
        if let Ok(report) = clan.read_entry(MERGE_REPORT_PATH) {
            if let Ok(report) = clan_sdk::MergeReport::from_yaml(&report) {
                if report.unresolved > 0 {
                    hints.push(format!(
                        "{} contested key(s) in merge-report.yaml — `clan read report {p}`, then adjudicate with patch-data + patch-decision",
                        report.unresolved
                    ));
                }
            }
        }
    }
    if let Some(view) = &m.view {
        if !view.present && view.renderable {
            hints.push(format!(
                "file is agent-only; `clan render {p}` materialises the human view when needed"
            ));
        } else if view.present && view.stale {
            hints.push(format!("human view is stale — `clan render {p}` to refresh"));
        }
    }
    hints
}

fn cmd_create(
    title: String,
    brief: String,
    doc_type: Option<String>,
    output: PathBuf,
    no_render: bool,
    hints: &Hints,
) -> Result<()> {
    let bytes = create(CreateOptions {
        title: title.clone(),
        brief,
        document_type: doc_type,
        no_render,
    })
    .context("failed to create .clan file")?;
    std::fs::write(&output, &bytes)
        .with_context(|| format!("could not write {}", output.display()))?;
    let clan = ClanFile::from_bytes(bytes.clone())?;
    eprintln!(
        "created {} ({} bytes)  id={}",
        output.display(),
        bytes.len(),
        clan.manifest().id
    );
    let mut lines = vec![format!("clan read agent {}", output.display())];
    lines.extend(file_state_hints(&clan, &output));
    hints.emit(&lines);
    Ok(())
}

fn cmd_fork(parent_path: PathBuf, agents: Vec<String>, output_dir: PathBuf, hints: &Hints) -> Result<()> {
    let parent = open(&parent_path)?;
    let branches = fork(&parent, &agents).context("fork failed")?;
    std::fs::create_dir_all(&output_dir)
        .with_context(|| format!("could not create {}", output_dir.display()))?;
    for (agent_id, bytes) in &branches {
        let path = output_dir.join(format!("{agent_id}.clan"));
        std::fs::write(&path, bytes)
            .with_context(|| format!("could not write {}", path.display()))?;
        eprintln!("forked {} for agent '{agent_id}'", path.display());
    }
    hints.emit(&[
        format!(
            "each agent writes only inside its own agents/<id>/ namespace ({} branches, conflicts impossible by construction)",
            branches.len()
        ),
        format!(
            "when all branches are done: clan merge {}\\*.clan --output <merged.clan>",
            output_dir.display()
        ),
    ]);
    Ok(())
}

fn cmd_merge(
    branch_paths: Vec<PathBuf>,
    output: PathBuf,
    policy_args: Vec<String>,
    prune_namespaces: bool,
    delta: Option<String>,
    hints: &Hints,
) -> Result<()> {
    let mut branches = Vec::with_capacity(branch_paths.len());
    for path in &branch_paths {
        branches.push(open(path)?);
    }

    let policies = if policy_args.is_empty() {
        None
    } else {
        let mut keys = std::collections::BTreeMap::new();
        for arg in &policy_args {
            let (key, policy) = arg
                .split_once('=')
                .with_context(|| format!("invalid --policy {arg:?}: expected KEY=POLICY"))?;
            keys.insert(key.to_string(), policy.to_string());
        }
        Some(MergePolicies { default: None, keys })
    };

    let outcome = merge(
        &branches,
        MergeOptions { policies, prune_namespaces, delta },
    )
    .context("merge failed")?;
    std::fs::write(&output, &outcome.bytes)
        .with_context(|| format!("could not write {}", output.display()))?;
    eprintln!(
        "merged {} branches into {} ({} contested key(s))",
        branches.len(),
        output.display(),
        outcome.report.unresolved
    );
    if outcome.report.unresolved > 0 {
        hints.emit(&[
            format!(
                "{} contested key(s) recorded in merge-report.yaml — `clan read report {}`",
                outcome.report.unresolved,
                output.display()
            ),
            "adjudicate each: clan patch-data <file> <json> (settles the key), then clan patch-decision --agent <you> --action \"adjudicated <key>\" --rationale \"...\"".to_string(),
        ]);
    } else {
        hints.emit(&[format!(
            "clean merge — clan read agent {} to continue",
            output.display()
        )]);
    }
    Ok(())
}

fn cmd_render(file: PathBuf, hints: &Hints) -> Result<()> {
    let clan = open(&file)?;
    let bytes = render(&clan).context("render failed")?;
    std::fs::write(&file, &bytes)?;
    eprintln!("materialised human view in {}", file.display());
    hints.emit(&[format!(
        "open in the viewer, or `clan read human {}` to print the HTML",
        file.display()
    )]);
    Ok(())
}

fn cmd_validate(file: PathBuf, strict: bool, hints: &Hints) -> Result<()> {
    let clan = ClanFile::open(&file)
        .with_context(|| format!("could not open {}", file.display()))?;
    let report = validate(&clan);
    println!("{}", report.display());
    if !report.is_valid() {
        std::process::exit(1);
    }
    if strict && !report.is_content_valid() {
        std::process::exit(2);
    }
    hints.emit(&file_state_hints(&clan, &file));
    Ok(())
}

fn cmd_read(section: ReadSection) -> Result<()> {
    match section {
        ReadSection::Agent { file, no_patches, skip_guide } => {
            let clan = open(&file)?;
            let ctx = assemble(
                &clan,
                &InjectOptions {
                    include_patches: !no_patches,
                    skip_guide,
                },
            )
            .context("failed to assemble agent context")?;
            print!("{}", ctx.text);
        }
        ReadSection::Human { file } => {
            let clan = open(&file)?;
            print!("{}", clan.read_entry_string("human/index.html")?);
        }
        ReadSection::Data { file } => {
            let clan = open(&file)?;
            print!("{}", clan.read_entry_string("shared/data.yaml")?);
        }
        ReadSection::Chain { file } => {
            let clan = open(&file)?;
            print!("{}", clan.read_entry_string("agent/decision-chain.yaml")?);
        }
        ReadSection::Report { file } => {
            let clan = open(&file)?;
            if !clan.has_entry(MERGE_REPORT_PATH) {
                anyhow::bail!(
                    "no merge report: {} was not produced by `clan merge` (or its report was pruned)",
                    file.display()
                );
            }
            let yaml = clan.read_entry(MERGE_REPORT_PATH)?;
            print!("{}", clan_sdk::yaml_to_toon(&yaml)?);
        }
    }
    Ok(())
}

fn cmd_info(file: PathBuf) -> Result<()> {
    let clan = open(&file)?;
    let m = clan.manifest();
    println!("title:    {}", m.title);
    println!("id:       {}", m.id);
    println!("version:  {}.{}", m.clan_version, m.clan_version_minor);
    println!("created:  {}", m.created_at);
    println!("updated:  {}", m.updated_at);
    if let Some(doc_type) = &m.document_type {
        println!("type:     {doc_type}");
    }
    if let Some(lin) = &m.lineage {
        println!("parent:   {}", lin.parent_id);
        println!("delta:    {}", lin.delta);
    } else {
        println!("parent:   (root — no lineage)");
    }
    println!("sha256:   {}", clan.sha256());
    println!("files:    {}", m.files.len());
    Ok(())
}

fn cmd_pack(
    parent_path: PathBuf,
    output_json: String,
    output: PathBuf,
    schema_path: Option<PathBuf>,
    delta: Option<String>,
    hints: &Hints,
) -> Result<()> {
    let parent = open(&parent_path)?;

    let json = if output_json == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&output_json)
            .with_context(|| format!("could not read {output_json}"))?
    };

    let agent_output = AgentOutput::from_json(&json).context("failed to parse agent output")?;
    let schema_override = if let Some(sp) = schema_path {
        Some(std::fs::read_to_string(&sp).with_context(|| format!("could not read schema {}", sp.display()))?)
    } else { None };

    let opts = PackOptions {
        delta,
        output_path: Some(parent_path.display().to_string()),
        schema_override,
        ..Default::default()
    };
    let bytes = pack(&parent, agent_output, opts, None).context("pack failed")?;
    std::fs::write(&output, &bytes)
        .with_context(|| format!("could not write {}", output.display()))?;
    eprintln!(
        "packed {} ({} bytes)",
        output.display(),
        bytes.len()
    );
    hints.emit(&file_state_hints(&ClanFile::from_bytes(bytes)?, &output));
    Ok(())
}

fn cmd_export_static(file: PathBuf, output: Option<PathBuf>) -> Result<()> {
    let clan = open(&file)?;
    let json = export_static(&clan).context("export failed")?;
    let pretty = serde_json::to_string_pretty(&json)?;
    match output {
        Some(path) => {
            std::fs::write(&path, &pretty)
                .with_context(|| format!("could not write {}", path.display()))?;
            eprintln!("wrote {}", path.display());
        }
        None => println!("{pretty}"),
    }
    Ok(())
}

fn cmd_pack_html(
    parent_path: PathBuf,
    html_file: String,
    output: PathBuf,
    assets_dir: Option<PathBuf>,
    schema_path: Option<PathBuf>,
    delta: Option<String>,
    hints: &Hints,
) -> Result<()> {
    let parent = open(&parent_path)?;

    let raw_html = if html_file == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&html_file)
            .with_context(|| format!("could not read {html_file}"))?
    };

    let mut assets_map = std::collections::HashMap::new();
    if let Some(dir) = assets_dir {
        if dir.is_dir() {
            for entry in std::fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let bytes = std::fs::read(&path)?;
                        assets_map.insert(name.to_string(), bytes);
                    }
                }
            }
        }
    }

    let schema_override = if let Some(sp) = schema_path {
        Some(std::fs::read_to_string(&sp).with_context(|| format!("could not read schema {}", sp.display()))?)
    } else { None };

    let bytes = pack_html(&parent, &raw_html, Some(assets_map), schema_override, delta, None).context("pack-html failed")?;
    std::fs::write(&output, &bytes)
        .with_context(|| format!("could not write {}", output.display()))?;
    eprintln!("packed {} ({} bytes)", output.display(), bytes.len());
    hints.emit(&file_state_hints(&ClanFile::from_bytes(bytes)?, &output));
    Ok(())
}

fn cmd_patch_html(file: PathBuf, html_file: String, delta: Option<String>) -> Result<()> {
    let clan = open(&file)?;
    let raw_html = if html_file == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&html_file).with_context(|| format!("could not read {html_file}"))?
    };
    
    let bytes = pack_html(&clan, &raw_html, None, None, delta, None)?;
    std::fs::write(&file, &bytes)?;
    eprintln!("Patched {} in-place", file.display());
    Ok(())
}

fn cmd_edit(file: PathBuf) -> Result<()> {
    let clan = open(&file)?;
    let temp_dir = tempfile::tempdir()?;
    let html_path = temp_dir.path().join("index.html");
    
    let existing_html = clan.read_entry_string("human/index.html").unwrap_or_default();
    let existing_data = clan.read_entry_string("shared/data.yaml").unwrap_or_default();
    
    let mut content = String::new();
    if !existing_data.is_empty() {
        content.push_str("---\nstructured:\n");
        for line in existing_data.lines() {
            content.push_str(&format!("  {}\n", line));
        }
        content.push_str("---\n");
    }
    content.push_str(&existing_html);
    std::fs::write(&html_path, &content)?;
    
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
    let status = std::process::Command::new(editor)
        .arg(&html_path)
        .status()?;
        
    if !status.success() {
        anyhow::bail!("Editor exited with non-zero status");
    }
    
    let new_content = std::fs::read_to_string(&html_path)?;
    if new_content == content {
        eprintln!("No changes made.");
        return Ok(());
    }
    
    let bytes = pack_html(&clan, &new_content, None, None, Some("interactive human edit".into()), None)?;
    std::fs::write(&file, &bytes)?;
    eprintln!("Updated {}", file.display());
    Ok(())
}

fn cmd_agent_help() {
    print!("CLAN v{} AGENT PROTOCOL\n", env!("CARGO_PKG_VERSION"));
    print!(r#"Format: ZIP archive (.clan). Mutate ONLY via CLI.

# READ
clan read agent <file>    => Context, state, data, history (USE THIS FIRST)
clan read human <file>    => Rendered HTML
clan info <file>          => Manifest/lineage

# WRITE (Full Replace)
1. JSON Mode: clan pack --output <out> [--schema <schema>] <in> <json_file>
API Input Wrapper:
{{"mode":"full-html","structured":{{...}},"human":{{"html":"...","css":"...","assets":{{"img.png":"..."}}}},"decision":{{"agent":"X","action":"Y","rationale":"Z"}}}}
*(Note: output-schema.json ONLY validates the 'structured' payload object, not this API wrapper)*

2. HTML Mode (Token-efficient): clan pack-html --output <out> [--schema <schema>] <in> <html_file>
API Input Wrapper:
---
structured: {{...}}
decision: {{agent: X, action: Y, rationale: Z}}
---
<!DOCTYPE html><html>...
(Hint: Use {{{{key}}}} for templating, or window.__CLAN__.data in JS)

# PATCH (In-place, Lowest Token Cost, Preferred)
1. DOM: clan patch-html <file> <patch_file>
Schema:
---
mode: patch-html
patch_selector: "div#app"
patch_action: "append" | "replace" | "prepend"
---
<div>New</div>

2. Data: clan patch-data <file> <json>       (RFC7396 Merge Patch shared/data.yaml; MUST conform to output-schema.json)
3. State: clan patch-state <file> <json>     (RFC7396 Merge Patch agent/state.yaml)
4. Notes: clan patch-context <file> <md> [--append]
5. History: clan patch-decision <file> --agent X --action Y --rationale Z
6. Asset: clan patch-asset <file> <path/in/zip> <local_file>
7. Schema: clan patch-schema <file> <schema.json>

# PARALLEL (fork/join, spec S24)
clan fork <file> --agents a,b,c --output-dir <dir>   => one branch per agent
On a BRANCH file: write ONLY your namespace agents/<you>/ :
  clan patch-data <branch> <json> --namespace        (your data)
  clan patch-decision <branch> --agent <you> ...     (auto-routed)
clan merge <branches...> --output <out> [--policy key=append|max|min|last-write|agent-priority]
clan read report <file>   => contested keys (adjudicate: patch-data + patch-decision)

# VIEW (optional, spec S23)
clan create/pack --no-render  => agent-only file (no HTML at each hop)
clan render <file>            => materialise the human view on demand

# VERIFY
clan validate <file>

Commands print `next:` hints gated on file state (suppress: --quiet / CLAN_NO_HINTS=1).
"#);
}

fn open(path: &PathBuf) -> Result<ClanFile> {
    ClanFile::open(path).with_context(|| format!("could not open {}", path.display()))
}

fn cmd_patch_data(file: PathBuf, json_file: String, namespace: bool, hints: &Hints) -> Result<()> {
    let clan = open(&file)?;
    let raw_json = if json_file == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&json_file).with_context(|| format!("could not read {json_file}"))?
    };

    let patch: serde_json::Value = serde_json::from_str(&raw_json)
        .context("invalid JSON patch provided")?;

    // Track which contested keys this write settles, for the adjudication hint.
    let contested_before: Vec<String> = clan
        .read_entry(MERGE_REPORT_PATH)
        .ok()
        .and_then(|b| clan_sdk::MergeReport::from_yaml(&b).ok())
        .map(|r| r.conflicts.iter().map(|c| c.key.clone()).collect())
        .unwrap_or_default();

    let bytes = if namespace {
        patch_data_namespaced(&clan, &patch)?
    } else {
        patch_data(&clan, &patch, None)?
    };
    std::fs::write(&file, &bytes)?;
    eprintln!("Patched data in-place: {}", file.display());

    let next = ClanFile::from_bytes(bytes)?;
    let mut lines: Vec<String> = Vec::new();
    if !namespace {
        let settled: Vec<&String> = contested_before
            .iter()
            .filter(|k| patch.get(k.as_str()).is_some())
            .collect();
        if !settled.is_empty() {
            lines.push(format!(
                "{} contested key(s) settled — record the adjudication: clan patch-decision {} --agent <you> --action \"adjudicated {}\" --rationale \"...\"",
                settled.len(),
                file.display(),
                settled
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
    }
    lines.extend(file_state_hints(&next, &file));
    hints.emit(&lines);
    Ok(())
}

fn cmd_patch_schema(file: PathBuf, schema_file: String) -> Result<()> {
    let clan = open(&file)?;
    let raw_schema = if schema_file == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&schema_file).with_context(|| format!("could not read {schema_file}"))?
    };

    let bytes = clan_sdk::pack::patch_schema(&clan, &raw_schema, None)?;
    std::fs::write(&file, &bytes)?;
    eprintln!("Patched schema in-place: {}", file.display());
    Ok(())
}

fn cmd_patch_decision(file: PathBuf, agent: String, action: String, rationale: String, pinned: bool, hints: &Hints) -> Result<()> {
    let clan = open(&file)?;
    let forked_ns = clan.manifest().fork_namespace().map(str::to_string);

    let entry = DecisionEntry {
        agent_name: agent,
        action,
        rationale,
        pinned,
    };

    let bytes = patch_decision(&clan, entry, None)?;
    std::fs::write(&file, &bytes)?;
    eprintln!("Appended decision in-place: {}", file.display());
    if let Some(ns) = forked_ns {
        hints.emit(&[format!(
            "decision recorded in your branch namespace ({ns}decisions.yaml); it folds into the shared chain at clan merge"
        )]);
    }
    Ok(())
}

fn cmd_patch_state(file: PathBuf, json_file: String) -> Result<()> {
    let clan = open(&file)?;
    let raw_json = if json_file == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&json_file).with_context(|| format!("could not read {json_file}"))?
    };

    let patch: serde_json::Value = serde_json::from_str(&raw_json)
        .context("invalid JSON patch provided")?;

    let bytes = patch_state(&clan, &patch)?;
    std::fs::write(&file, &bytes)?;
    eprintln!("Patched state in-place: {}", file.display());
    Ok(())
}

fn cmd_patch_context(file: PathBuf, markdown_file: String, append: bool) -> Result<()> {
    let clan = open(&file)?;
    let text = if markdown_file == "-" {
        use std::io::Read;
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        s
    } else {
        std::fs::read_to_string(&markdown_file).with_context(|| format!("could not read {markdown_file}"))?
    };

    let bytes = patch_context(&clan, &text, append)?;
    std::fs::write(&file, &bytes)?;
    eprintln!("Patched context in-place: {}", file.display());
    Ok(())
}

fn cmd_patch_asset(file: PathBuf, internal_path: String, local_file: PathBuf) -> Result<()> {
    let clan = open(&file)?;
    let bytes = std::fs::read(&local_file).with_context(|| format!("could not read {}", local_file.display()))?;

    let out_bytes = patch_asset(&clan, &internal_path, bytes)?;
    std::fs::write(&file, &out_bytes)?;
    eprintln!("Patched asset in-place: {}", file.display());
    Ok(())
}
