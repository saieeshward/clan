//! `clan` — command-line tool for CLAN files.
//!
//! Usage:
//!   clan create --title "My Doc" --brief "…" [--type invoice] <output.clan>
//!   clan validate <file.clan>
//!   clan read agent <file.clan>
//!   clan read human <file.clan>
//!   clan read data <file.clan>
//!   clan info <file.clan>
//!   clan pack --output <next.clan> [--delta "…"] <parent.clan> <output.json>
//!   clan pack-html --output <next.clan> [--delta "…"] <parent.clan> <output.html>
//!   clan export-static <file.clan> [--output static.json]
//!   clan agent-help

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clan_sdk::{
    assemble, create, export_static, pack, pack_html, validate, AgentOutput, ClanFile,
    CreateOptions, InjectOptions, PackOptions,
};

#[derive(Parser)]
#[command(
    name = "clan",
    about = "CLAN — Context and Live Agent Notation. Read, write, validate, and pipeline .clan files.",
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
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
        output: PathBuf,
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
        /// Human-readable description of what changed.
        #[arg(long)]
        delta: Option<String>,
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
    },
    /// Print human/index.html.
    Human { file: PathBuf },
    /// Print shared/data.yaml.
    Data { file: PathBuf },
    /// Print agent/decision-chain.yaml.
    Chain { file: PathBuf },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Create {
            title,
            brief,
            doc_type,
            output,
        } => cmd_create(title, brief, doc_type, output),
        Commands::Validate { file, strict } => cmd_validate(file, strict),
        Commands::Read { section } => cmd_read(section),
        Commands::Info { file } => cmd_info(file),
        Commands::Pack {
            parent,
            output_json,
            output,
            delta,
        } => cmd_pack(parent, output_json, output, delta),
        Commands::ExportStatic { file, output } => cmd_export_static(file, output),
        Commands::PackHtml { parent, html_file, output, delta } => {
            cmd_pack_html(parent, html_file, output, delta)
        }
        Commands::AgentHelp => { cmd_agent_help(); Ok(()) }
    }
}

fn cmd_create(
    title: String,
    brief: String,
    doc_type: Option<String>,
    output: PathBuf,
) -> Result<()> {
    let bytes = create(CreateOptions {
        title: title.clone(),
        brief,
        document_type: doc_type,
    })
    .context("failed to create .clan file")?;
    std::fs::write(&output, &bytes)
        .with_context(|| format!("could not write {}", output.display()))?;
    eprintln!(
        "created {} ({} bytes)  id={}",
        output.display(),
        bytes.len(),
        ClanFile::from_bytes(bytes)?.manifest().id
    );
    Ok(())
}

fn cmd_validate(file: PathBuf, strict: bool) -> Result<()> {
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
    Ok(())
}

fn cmd_read(section: ReadSection) -> Result<()> {
    match section {
        ReadSection::Agent { file, no_patches } => {
            let clan = open(&file)?;
            let ctx = assemble(
                &clan,
                &InjectOptions {
                    include_patches: !no_patches,
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
    delta: Option<String>,
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
    let opts = PackOptions {
        delta,
        output_path: Some(parent_path.display().to_string()),
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
    delta: Option<String>,
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

    let bytes = pack_html(&parent, &raw_html, delta, None).context("pack-html failed")?;
    std::fs::write(&output, &bytes)
        .with_context(|| format!("could not write {}", output.display()))?;
    eprintln!("packed {} ({} bytes)", output.display(), bytes.len());
    Ok(())
}

fn cmd_agent_help() {
    // Deliberately terse — every line here costs agent tokens.
    print!(r#"CLAN agent-help  (clan {version})
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

STEP 1 — Read your task (ONE command, not two):
  clan read agent <file.clan>
  ⚠ This includes all data. Do NOT also run `clan read data` — same content, wasted tokens.

STEP 2a — Produce output as JSON (standard path):
  Write a JSON file matching this shape:
  {{
    "mode": "full-html",
    "structured": {{ "key": "value" }},
    "human": {{
      "html": "<!DOCTYPE html>...",
      "css": "",
      "assets": {{ "chart.svg": "<svg>...</svg>" }}
    }},
    "decision": {{
      "agent": "your-name", "action": "what you did", "rationale": "why"
    }}
  }}
  Then pack:  clan pack --output next.clan --delta "..." parent.clan output.json

STEP 2b — Produce output as a raw HTML file (lower token cost, preferred):
  Write a .html file. Optionally add YAML frontmatter at the very top:
  ---
  structured:
    key_finding: "example"
  decision:
    agent: "agent3"
    action: "produced final design"
    rationale: "..."
  ---
  <!DOCTYPE html>
  ...
  Then pack:  clan pack-html --output next.clan --delta "..." parent.clan output.html

STEP 3 — Verify:
  clan info next.clan
  clan validate next.clan

OTHER COMMANDS (don't waste tokens on these unless you need them):
  clan read human <file>   print current html
  clan read chain <file>   print decision history only
  clan info <file>         manifest metadata
  clan validate <file>     structural check
"#, version = env!("CARGO_PKG_VERSION"));
}

fn open(path: &PathBuf) -> Result<ClanFile> {
    ClanFile::open(path).with_context(|| format!("could not open {}", path.display()))
}
