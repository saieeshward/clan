#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::Mutex;

use clan_sdk::{apply_patch_and_repack, validate, ClanFile};
use serde::{Deserialize, Serialize};
use tauri::{State, Manager, Emitter};

// ── File logger (writes to /tmp/clan-debug.log) ──────────────────────────────
fn log(msg: &str) {
    use std::io::Write;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true).append(true).open("/tmp/clan-debug.log")
    {
        let _ = writeln!(f, "[{ts}] {msg}");
    }
}

struct AppState {
    current: Mutex<Option<LoadedClan>>,
    edit_mode: Mutex<bool>,
    preview_html: Mutex<String>,
}

struct LoadedClan {
    path: PathBuf,
    clan: ClanFile,
    raw_bytes: Vec<u8>,
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

    let raw_bytes = std::fs::read(&p).map_err(|e| e.to_string())?;
    *state.current.lock().unwrap() = Some(LoadedClan { path: p.clone(), clan, raw_bytes });

    Ok(OpenResult {
        path: p.display().to_string(),
        manifest: info,
        validation: report.display(),
        has_human_view,
    })
}

#[tauri::command]
fn get_human_html(state: State<AppState>) -> Result<String, String> {
    log("get_human_html: called");
    let guard = state.current.lock().unwrap();
    let loaded = guard.as_ref().ok_or("no file open")?;
    let html = loaded.clan.read_entry_string("human/index.html").map_err(|e| e.to_string())?;
    let resolved = if let Ok(bytes) = loaded.clan.read_entry("shared/data.yaml") {
        if let Ok(data) = serde_yaml::from_slice::<serde_yaml::Value>(&bytes) {
            resolve_bindings(&html, &data)
        } else { html }
    } else { html };
    // Auto-inject data-adf-id on editable elements that the agent didn't annotate.
    // Must happen before apply_patches so patch lookups find the stable auto IDs.
    let with_ids = auto_inject_adf_ids(&resolved);
    let patched = if loaded.clan.has_entry("human/patches.yaml") {
        if let Ok(yaml) = loaded.clan.read_entry_string("human/patches.yaml") {
            apply_patches(&with_ids, &yaml)
        } else { with_ids }
    } else { with_ids };

    // Inject human/styles.css into the document if present.
    // For full HTML docs: inject into <head>. For fragments: prepend a <style> block.
    let css = loaded.clan.read_entry_string("human/styles.css").unwrap_or_default();
    let final_html = inject_styles(&patched, &css);
    Ok(final_html)
}

fn inject_styles(html: &str, css: &str) -> String {
    if css.is_empty() {
        return html.to_string();
    }
    let style_tag = format!("<style>{}</style>", css);
    let lower = html.to_lowercase();
    if lower.contains("</head>") {
        html.replacen("</head>", &format!("{}</head>", style_tag), 1)
    } else if lower.contains("<body") {
        // Fragment with no <head>: prepend style block
        format!("{}\n{}", style_tag, html)
    } else {
        format!("{}\n{}", style_tag, html)
    }
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
        let Some(attr_pos) = result.find(&marker) else {
            log(&format!("apply_patches: id={:?} NOT FOUND in HTML", p.id));
            continue;
        };

        // Walk back to the opening `<` of the tag that holds this attribute.
        let tag_open = result[..attr_pos].rfind('<').unwrap_or(0);

        // Extract the tag name (e.g. "h1", "p", "div").
        let tag_name: String = result[tag_open + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_lowercase();

        // Find the `>` that closes the opening tag.
        let Some(gt_rel) = result[attr_pos..].find('>') else {
            log(&format!("apply_patches: id={:?} no closing > for opening tag", p.id));
            continue;
        };
        let content_start = attr_pos + gt_rel + 1;

        // Find the matching closing tag, respecting nesting.
        let close_pos = find_closing_tag(&result, content_start, &tag_name);
        let Some(close_pos) = close_pos else {
            log(&format!("apply_patches: id={:?} tag={tag_name:?} no matching closing tag", p.id));
            continue;
        };

        log(&format!(
            "apply_patches: id={:?} tag={tag_name:?} content_start={content_start} close_pos={close_pos} \
             old={:?}… new={:?}…",
            p.id,
            &result[content_start..close_pos.min(content_start + 60)],
            &p.content[..p.content.len().min(60)],
        ));

        result = format!(
            "{}{}{}",
            &result[..content_start],
            p.content,
            &result[close_pos..]
        );
    }
    result
}

/// Find the absolute position of the matching closing `</tag>` in `html`,
/// starting the search at `from`. Tracks nested same-tag pairs so a `<div>` that
/// contains another `<div>` resolves to its own `</div>`, not the inner one.
fn find_closing_tag(html: &str, from: usize, tag: &str) -> Option<usize> {
    let lower = html.to_lowercase();
    let open_pat = format!("<{}", tag);
    let close_pat = format!("</{}", tag);
    let mut depth: i32 = 0;
    let mut pos = from;

    loop {
        let slice = &lower[pos..];

        // Find the next opening tag of the same type (could be a nested child).
        let next_open = slice.find(open_pat.as_str()).and_then(|rel| {
            let after = pos + rel + open_pat.len();
            // Confirm it is really this tag and not a prefix (e.g. <td vs <thead).
            matches!(lower.as_bytes().get(after), Some(b' ' | b'\t' | b'\n' | b'\r' | b'>') | None)
                .then_some(pos + rel)
        });

        // Find the next closing tag.
        let next_close = slice.find(close_pat.as_str()).map(|rel| pos + rel);

        match (next_open, next_close) {
            (Some(o), Some(c)) if o < c => {
                // Nested open before close: go deeper.
                depth += 1;
                pos = o + open_pat.len();
            }
            (_, Some(c)) => {
                if depth == 0 {
                    return Some(c); // This is the matching close.
                }
                depth -= 1;
                pos = c + close_pat.len();
            }
            _ => return None,
        }
    }
}

/// Inject `data-adf-id` on editable block elements that the agent didn't annotate.
/// IDs are stable: same HTML always produces the same IDs (tag-type + sequential index).
fn auto_inject_adf_ids(html: &str) -> String {
    const EDITABLE: &[&str] = &["h1", "h2", "h3", "h4", "h5", "h6", "p", "li", "td", "th"];
    let mut result = html.to_string();
    for tag in EDITABLE {
        result = inject_ids_for_tag(&result, tag);
    }
    result
}

fn inject_ids_for_tag(html: &str, tag: &str) -> String {
    let mut out = String::with_capacity(html.len() + 64);
    let mut pos = 0;
    let mut count = 0usize;
    let open = format!("<{}", tag);

    while pos < html.len() {
        let Some(rel) = html[pos..].find(open.as_str()) else {
            out.push_str(&html[pos..]);
            break;
        };
        let tag_start = pos + rel;
        let after_name = tag_start + open.len();

        // Verify next char is a tag boundary, not part of a longer tag name (e.g. <pre> vs <p>).
        let next = html.as_bytes().get(after_name).copied().unwrap_or(0);
        if !matches!(next, b' ' | b'\t' | b'\n' | b'\r' | b'>') {
            out.push_str(&html[pos..after_name]);
            pos = after_name;
            continue;
        }

        // Find the `>` ending this opening tag.
        let Some(rel_end) = html[tag_start..].find('>') else {
            out.push_str(&html[pos..]);
            break;
        };
        let tag_end = tag_start + rel_end;
        let tag_src = &html[tag_start..=tag_end];

        out.push_str(&html[pos..tag_end]);

        if !tag_src.contains("data-adf-id") && !tag_src.ends_with("/>") {
            out.push_str(&format!(" data-adf-id=\"auto-{}-{}\"", tag, count));
            count += 1;
        }
        out.push('>');
        pos = tag_end + 1;
    }
    out
}



#[tauri::command]
fn set_edit_mode(active: bool, state: State<AppState>) {
    *state.edit_mode.lock().unwrap() = active;
}

#[tauri::command]
fn update_preview_html(html: String, state: State<AppState>) {
    *state.preview_html.lock().unwrap() = html;
}

fn do_save_patch(id: String, content: String, state: &AppState) -> Result<(), String> {
    log(&format!("save_patch: id={id:?} content={:?}…", &content[..content.len().min(80)]));

    let mut guard = state.current.lock().unwrap();
    let loaded = guard.as_mut().ok_or("no file open")?;

    let new_bytes = apply_patch_and_repack(&loaded.clan, id.clone(), content.clone())
        .map_err(|e| e.to_string())?;

    std::fs::write(&loaded.path, &new_bytes).map_err(|e| e.to_string())?;

    loaded.raw_bytes = new_bytes.clone();
    loaded.clan = ClanFile::from_bytes(new_bytes).map_err(|e| e.to_string())?;

    log(&format!("save_patch: done, file repacked. id={id:?}"));
    Ok(())
}

#[tauri::command]
fn save_patch(id: String, content: String, state: State<AppState>) -> Result<(), String> {
    do_save_patch(id, content, &*state)
}

fn main() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("clan", |app, request| {
            let uri = request.uri().to_string();
            let state = app.app_handle().state::<AppState>();
            
            if uri.contains("edit-mode") {
                let mode = *state.edit_mode.lock().unwrap();
                let body = if mode { "true" } else { "false" };
                tauri::http::Response::builder()
                    .header("Access-Control-Allow-Origin", "*")
                    .status(200)
                    .body(body.as_bytes().to_vec())
                    .unwrap()
            } else if uri.contains("document") {
                let html = state.preview_html.lock().unwrap().clone();
                tauri::http::Response::builder()
                    .header("Content-Type", "text/html")
                    .header("Access-Control-Allow-Origin", "*")
                    .status(200)
                    .body(html.into_bytes())
                    .unwrap()
            } else if uri.contains("patch") {
                if let Ok(body_str) = String::from_utf8(request.body().clone()) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                        if let (Some(id), Some(content)) = (json["id"].as_str(), json["content"].as_str()) {
                            let _ = do_save_patch(id.to_string(), content.to_string(), &*state);
                            let _ = app.app_handle().emit("clan-patch-saved", serde_json::json!({ "id": id, "content": content }));
                        }
                    }
                }
                tauri::http::Response::builder()
                    .header("Access-Control-Allow-Origin", "*")
                    .status(200)
                    .body(Vec::new())
                    .unwrap()
            } else {
                tauri::http::Response::builder().status(404).body(Vec::new()).unwrap()
            }
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { 
            current: Mutex::new(None), 
            edit_mode: Mutex::new(false),
            preview_html: Mutex::new(String::new()),
        })
        .invoke_handler(tauri::generate_handler![
            open_clan, get_human_html, get_data, get_chain, get_agent_state, get_context,
            save_patch, set_edit_mode, update_preview_html
        ])
        .run(tauri::generate_context!())
        .expect("error while running CLAN Viewer");
}
