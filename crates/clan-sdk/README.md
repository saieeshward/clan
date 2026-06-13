# clan-sdk

Reference SDK for **CLAN — Context and Live Agent Notation**.

A `.clan` file is a ZIP container that is simultaneously machine-readable for AI
agents and human-renderable, carrying its own specification and a verifiable
lineage chain. `clan-sdk` lets you read, write, validate, and pipeline `.clan`
files from Rust.

For the full format specification, see [`CLAN-SPEC.md`](https://github.com/saieeshward/clan/blob/main/spec/CLAN-SPEC.md).

## Installation

```toml
[dependencies]
clan-sdk = "1"
```

## Features

- **Read / write** `.clan` containers (`ClanFile`, `ClanBuilder`)
- **Create** new files and **export** a static human view (`create`, `export_static`)
- **Validate** against the CLAN schema (`validate`)
- **Assemble** a token-optimized agent prompt from a container (`assemble`)
- **Patch** data, state, context, decisions, and assets (`patch_*`)
- **Decision chains** with verifiable lineage (`Decision`, `DecisionChain`)
- **TOON** compact serialization (`to_toon`, `yaml_to_toon`)
- **Chain compression** for token savings (`compress_chain`, `nlp_compress`)

## Example

```rust
use clan_sdk::{ClanFile, assemble, InjectOptions, validate};

// Open and validate a container.
let clan = ClanFile::open("example.clan")?;
let report = validate(&clan)?;
assert!(report.is_valid());

// Build a token-optimized prompt for an agent.
let ctx = assemble(&clan, &InjectOptions::default())?;
println!("{}", ctx.prompt);
# Ok::<(), clan_sdk::Error>(())
```

## Related crates

- [`clan-cli`](https://crates.io/crates/clan-cli) — the `clan` command-line tool built on this SDK.

## License

Licensed under the Mozilla Public License, Version 2.0. See [LICENSE](https://github.com/saieeshward/clan/blob/main/LICENSE).
