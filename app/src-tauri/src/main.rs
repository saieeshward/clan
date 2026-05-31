#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use clan_sdk::{validate, ClanFile};
use serde::{Deserialize, Serialize};
use tauri::State;

struct AppState {
    current: Mutex<Option<LoadedClan>>,
}

struct LoadedClan {
    path: PathBuf,
    clan: ClanFile,
}

#[derive(Serialize, Deserialize)]
struct ManifestInfo {
    title: String,
    id: String,
    version: String,
    created_at: String,
    updated_at: String,
    document_type: Option<String>,
    sha256: String,
    file_count: usize,
    lineage: Option<LineageInfo>,
}

#[derive(Serialize, Deserialize)]
struct LineageInfo {
    parent_id: String,
    parent_uri: String,
    parent_sha256: Option<String>,
    delta: String,
}

#[derive(Serialize, Deserialize)]
struct OpenResult {
    path: String,
    manifest: ManifestInfo,
    validation: String,
    has_human_view: bool,
}

#[tauri::command]
fn open_clan(path: String, state: State<AppState>) -> Result<OpenResult, String> {
    let p = PathBuf::from(&path);
    let clan = ClanFile::open(&p).map_err(|e| e.to_string())?;
    let manifest = clan.manifest().clone();
    let report = validate(&clan);
    let has_human_view = clan.has_entry("human/index.html");
    let sha256 = clan.sha256();

    let info = ManifestInfo {
        title: manifest.title.clone(),
        id: manifest.id.clone(),
        version: format!("{}.{}", manifest.clan_version, manifest.clan_version_minor),
        created_at: manifest.created_at.clone(),
        updated_at: manifest.updated_at.clone(),
        document_type: manifest.document_type.clone(),
        sha256,
        file_count: manifest.files.len(),
        lineage: manifest.lineage.as_ref().map(|l| LineageInfo {
            parent_id: l.parent_id.clone(),
            parent_uri: l.parent_uri.clone(),
            parent_sha256: l.parent_sha256.clone(),
            delta: l.delta.clone(),
        }),
    };

    *state.current.lock().unwrap() = Some(LoadedClan { path: p.clone(), clan });

    Ok(OpenResult {
        path: p.display().to_string(),
        manifest: info,
        validation: report.display(),
        has_human_view,
    })
}

#[tauri::command]
fn get_human_html(state: State<AppState>) -> Result<String, String> {
    let guard = state.current.lock().unwrap();
    let loaded = guard.as_ref().ok_or("no file open")?;
    let html = loaded.clan.read_entry_string("human/index.html").map_err(|e| e.to_string())?;
    let resolved = if let Ok(bytes) = loaded.clan.read_entry("shared/data.yaml") {
        if let Ok(data) = serde_yaml::from_slice::<serde_yaml::Value>(&bytes) {
            resolve_bindings(&html, &data)
        } else { html }
    } else { html };
    let patched = if loaded.clan.has_entry("human/patches.yaml") {
        if let Ok(yaml) = loaded.clan.read_entry_string("human/patches.yaml") {
            apply_patches(&resolved, &yaml)
        } else { resolved }
    } else { resolved };
    Ok(patched)
}

#[tauri::command]
fn get_data(state: State<AppState>) -> Result<String, String> {
    let guard = state.current.lock().unwrap();
    guard.as_ref().ok_or("no file open")?.clan
        .read_entry_string("shared/data.yaml").map_err(|e| e.to_string())
}

#[tauri::command]
fn get_chain(state: State<AppState>) -> Result<String, String> {
    let guard = state.current.lock().unwrap();
    guard.as_ref().ok_or("no file open")?.clan
        .read_entry_string("agent/decision-chain.yaml").map_err(|e| e.to_string())
}

#[tauri::command]
fn get_agent_state(state: State<AppState>) -> Result<String, String> {
    let guard = state.current.lock().unwrap();
    guard.as_ref().ok_or("no file open")?.clan
        .read_entry_string("agent/state.yaml").map_err(|e| e.to_string())
}

#[tauri::command]
fn get_context(state: State<AppState>) -> Result<String, String> {
    let guard = state.current.lock().unwrap();
    guard.as_ref().ok_or("no file open")?.clan
        .read_entry_string("agent/context.md").map_err(|e| e.to_string())
}

fn resolve_bindings(html: &str, data: &serde_yaml::Value) -> String {
    let mut output = String::with_capacity(html.len());
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '{' && chars[i+1] == '{' {
            let rest: String = chars[i+2..].iter().collect();
            if let Some(end) = rest.find("}}") {
                let key = rest[..end].trim();
                output.push_str(&resolve_key(key, data));
                i += 2 + end + 2;
                continue;
            }
        }
        output.push(chars[i]);
        i += 1;
    }
    output
}

fn resolve_key(key: &str, data: &serde_yaml::Value) -> String {
    let mut current = data;
    for part in key.split('.') {
        current = match current {
            serde_yaml::Value::Mapping(m) => match m.get(serde_yaml::Value::String(part.to_string())) {
                Some(v) => v,
                None => return format!("{{{{{key}}}}}"),
            },
            serde_yaml::Value::Sequence(s) => match part.parse::<usize>().ok().and_then(|i| s.get(i)) {
                Some(v) => v,
                None => return format!("{{{{{key}}}}}"),
            },
            _ => return format!("{{{{{key}}}}}"),
        };
    }
    match current {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        _ => String::new(),
    }
}

fn apply_patches(html: &str, patch_yaml: &str) -> String {
    #[derive(Deserialize)] struct Patches { #[serde(default)] patches: Vec<Patch> }
    #[derive(Deserialize)] struct Patch { id: String, content: String }
    let Ok(ps) = serde_yaml::from_str::<Patches>(patch_yaml) else { return html.to_string() };
    let mut result = html.to_string();
    for p in ps.patches {
        let marker = format!("data-adf-id=\"{}\"", p.id);
        if let Some(ts) = result.find(&marker) {
            if let Some(te) = result[ts..].find('>') {
                let cs = ts + te + 1;
                if let Some(ce) = result[cs..].find('<') {
                    result = format!("{}{}{}", &result[..cs], p.content, &result[cs+ce..]);
                }
            }
        }
    }
    result
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { current: Mutex::new(None) })
        .invoke_handler(tauri::generate_handler![
            open_clan, get_human_html, get_data, get_chain, get_agent_state, get_context,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CLAN Viewer");
}
