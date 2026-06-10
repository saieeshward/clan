// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! On-demand human-view materialisation (spec §23).
//!
//! The structured members are canonical; the HTML view is derivable. `render`
//! produces a deterministic default-theme view from `shared/data.yaml`, the
//! manifest, and the decision chain — nbviewer-style: agent-only chains skip
//! rendering at every hop, and any hop (or the human's tooling) materialises
//! the view once, on demand. Scalar fields are emitted as `{{key}}` bindings
//! so the viewer keeps resolving live data.

use chrono::Utc;
use uuid::Uuid;

use crate::container::{ClanBuilder, ClanFile};
use crate::decision::DecisionChain;
use crate::error::Result;
use crate::manifest::{FileEntry, Lineage, ViewState};

/// Materialise `human/index.html` (+ `human/index.txt`) from the structured
/// members and return the bytes of the new generation.
pub fn render(clan: &ClanFile) -> Result<Vec<u8>> {
    let now = Utc::now().to_rfc3339();
    let manifest = clan.manifest();

    let data: serde_yaml::Value = clan
        .read_entry("shared/data.yaml")
        .ok()
        .and_then(|b| serde_yaml::from_slice(&b).ok())
        .unwrap_or(serde_yaml::Value::Null);
    let chain = clan
        .read_entry("agent/decision-chain.yaml")
        .ok()
        .and_then(|b| DecisionChain::from_yaml(&b).ok())
        .unwrap_or_default();

    let html = default_theme_html(&manifest.title, &data, &chain);
    let txt = plain_text(&manifest.title, &data);

    let mut new_manifest = manifest.clone();
    new_manifest.id = Uuid::new_v4().to_string();
    new_manifest.updated_at = now.clone();
    new_manifest.lineage = Some(Lineage {
        parent_id: manifest.id.clone(),
        parent_uri: format!("file:///unknown/{}.clan", manifest.id),
        parent_sha256: Some(clan.sha256()),
        delta: "materialised human view (clan render)".into(),
        parents: Vec::new(),
        merge: false,
    });
    new_manifest.view = Some(ViewState {
        present: true,
        renderable: true,
        stale: false,
    });
    for (id, path, role, ct) in [
        ("human-view", "human/index.html", "human-view", "text/html"),
        ("human-text", "human/index.txt", "human-text", "text/plain"),
    ] {
        if !new_manifest.files.iter().any(|f| f.path == path) {
            new_manifest.files.push(FileEntry {
                id: id.into(),
                path: path.into(),
                role: role.into(),
                content_type: ct.into(),
                priority: Some(1),
                sha256: None,
            });
        }
    }

    let mut builder = ClanBuilder::new(new_manifest);
    for (path, bytes) in clan.read_all_entries()? {
        if path == crate::container::MANIFEST_PATH
            || path == "human/index.html"
            || path == "human/index.txt"
        {
            continue;
        }
        builder.add_entry(path, bytes);
    }
    builder.add_entry("human/index.html", html.into_bytes());
    builder.add_entry("human/index.txt", txt.into_bytes());
    builder.build()
}

/// Scalar values render as `{{key}}` bindings (resolved live by the viewer);
/// composite values render as preformatted YAML.
fn default_theme_html(title: &str, data: &serde_yaml::Value, chain: &DecisionChain) -> String {
    let mut rows = String::new();
    if let Some(map) = data.as_mapping() {
        for (key, value) in map {
            let Some(key) = key.as_str() else { continue };
            if key == "$schema" {
                continue;
            }
            let cell = if value.as_mapping().is_some() || value.as_sequence().is_some() {
                let yaml = serde_yaml::to_string(value).unwrap_or_default();
                format!("<pre>{}</pre>", escape(&yaml))
            } else {
                format!("<span data-adf-id=\"data-{key}\">{{{{{key}}}}}</span>")
            };
            rows.push_str(&format!(
                "      <tr><th>{}</th><td>{cell}</td></tr>\n",
                escape(key)
            ));
        }
    }

    let mut decisions = String::new();
    for d in chain.decisions.iter().take(5) {
        decisions.push_str(&format!(
            "      <li><strong>{}</strong> — {}<br><small>{}</small></li>\n",
            escape(&d.agent),
            escape(&d.action),
            escape(&d.rationale)
        ));
    }
    if decisions.is_empty() {
        decisions.push_str("      <li><small>No decisions recorded yet.</small></li>\n");
    }

    format!(
        "<!DOCTYPE html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n<title>{title_esc}</title>\n\
         <style>\n\
         body {{ background: #0f1117; color: #e2e4ee; font-family: ui-sans-serif, system-ui, sans-serif; max-width: 760px; margin: 0 auto; padding: 3rem 1.5rem; }}\n\
         h1 {{ color: #fff; border-bottom: 2px solid #6366f1; padding-bottom: .5rem; }}\n\
         h2 {{ color: #a5b4fc; margin-top: 2.5rem; font-size: 1rem; text-transform: uppercase; letter-spacing: .08em; }}\n\
         table {{ width: 100%; border-collapse: collapse; }}\n\
         th {{ text-align: left; color: #9ca3af; font-weight: 500; padding: .5rem .75rem .5rem 0; vertical-align: top; width: 35%; }}\n\
         td {{ padding: .5rem 0; }}\n\
         tr {{ border-bottom: 1px solid #1f2330; }}\n\
         pre {{ background: #161a26; padding: .6rem .8rem; border-radius: 6px; overflow-x: auto; font-size: .85rem; }}\n\
         ul {{ list-style: none; padding: 0; }}\n\
         li {{ border-left: 3px solid #6366f1; padding: .4rem 0 .4rem .8rem; margin-bottom: .6rem; }}\n\
         small {{ color: #9ca3af; }}\n\
         </style>\n</head>\n<body>\n\
         <h1 data-adf-id=\"render-title\">{title_esc}</h1>\n\
         <h2>Document Data</h2>\n    <table>\n{rows}    </table>\n\
         <h2>Recent Decisions</h2>\n    <ul>\n{decisions}    </ul>\n\
         </body>\n</html>\n",
        title_esc = escape(title),
    )
}

fn plain_text(title: &str, data: &serde_yaml::Value) -> String {
    let yaml = serde_yaml::to_string(data).unwrap_or_default();
    format!("{title}\n{}\n\n{yaml}", "=".repeat(title.chars().count()))
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::create::{create, CreateOptions};

    fn agent_only_root() -> ClanFile {
        let bytes = create(CreateOptions {
            title: "Render <Test>".into(),
            brief: "brief".into(),
            document_type: None,
            no_render: true,
        })
        .unwrap();
        ClanFile::from_bytes(bytes).unwrap()
    }

    #[test]
    fn no_render_create_has_no_view_and_is_renderable() {
        let clan = agent_only_root();
        assert!(!clan.has_entry("human/index.html"));
        let view = clan.manifest().view.as_ref().unwrap();
        assert!(!view.present);
        assert!(view.renderable);
        // The agent-only file still validates structurally.
        let report = crate::validate(&clan);
        assert!(report.is_valid(), "{}", report.display());
    }

    #[test]
    fn render_materialises_view_and_flips_flags() {
        let clan = agent_only_root();
        let clan = ClanFile::from_bytes(
            crate::pack::patch_data(&clan, &serde_json::json!({"vendor": "Acme", "total": 12}), None)
                .unwrap(),
        )
        .unwrap();

        let rendered = ClanFile::from_bytes(render(&clan).unwrap()).unwrap();
        let view = rendered.manifest().view.as_ref().unwrap();
        assert!(view.present);
        assert!(!view.stale);
        let html = rendered.read_entry_string("human/index.html").unwrap();
        // Scalars become live bindings; the title is escaped.
        assert!(html.contains("{{vendor}}"), "{html}");
        assert!(html.contains("Render &lt;Test&gt;"), "{html}");
        assert!(rendered.has_entry("human/index.txt"));
        assert_eq!(
            rendered.manifest().lineage.as_ref().unwrap().parent_id,
            clan.manifest().id
        );
    }

    #[test]
    fn render_is_deterministic_modulo_metadata() {
        let clan = agent_only_root();
        let a = ClanFile::from_bytes(render(&clan).unwrap()).unwrap();
        let b = ClanFile::from_bytes(render(&clan).unwrap()).unwrap();
        assert_eq!(
            a.read_entry_string("human/index.html").unwrap(),
            b.read_entry_string("human/index.html").unwrap()
        );
    }

    #[test]
    fn data_update_marks_existing_view_stale() {
        // Default create has a view; a data-only patch must mark it stale.
        let bytes = create(CreateOptions {
            title: "T".into(),
            brief: "b".into(),
            document_type: None,
            no_render: false,
        })
        .unwrap();
        let clan = ClanFile::from_bytes(bytes).unwrap();
        assert!(!clan.manifest().view.as_ref().unwrap().stale);

        let next = ClanFile::from_bytes(
            crate::pack::patch_data(&clan, &serde_json::json!({"k": "v"}), None).unwrap(),
        )
        .unwrap();
        let view = next.manifest().view.as_ref().unwrap();
        assert!(view.present);
        assert!(view.stale, "data change without view update must mark the view stale");

        // Re-rendering clears the flag.
        let rendered = ClanFile::from_bytes(render(&next).unwrap()).unwrap();
        assert!(!rendered.manifest().view.as_ref().unwrap().stale);
    }
}
