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
//!   clan export-static <file.clan> [--output static.json]

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use clan_sdk::{
    assemble, create, export_static, pack, validate, AgentOutput, ClanFile, CreateOptions,
    InjectOptions, PackOptions,
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

fn open(path: &PathBuf) -> Result<ClanFile> {
    ClanFile::open(path).with_context(|| format!("could not open {}", path.display()))
}
