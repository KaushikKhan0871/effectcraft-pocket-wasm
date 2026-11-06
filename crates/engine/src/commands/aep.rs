//! Native After Effects project import, adapted to the v0.6 command API.
use effectcraft_aep::import::{AepImportError, ImportOptions};
use effectcraft_project::{FootageKind, ItemKind};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use super::{CommandSpec, always, b_p, bad, str_p};
use crate::{EngineError, Result, Session, cmd};

fn import_aep(s: &mut Session, p: &Value) -> Result<Value> {
    let path = str_p(p, "path").ok_or_else(|| bad("file.importAep", "missing `path`"))?.to_string();
    let bytes = s.services.read_file(&path).map_err(|e| EngineError::Other(format!("cannot read {path}: {e}")))?;
    let allow_losses = b_p(p, "allowLosses").unwrap_or(false);
    let dry_run = b_p(p, "dryRun").unwrap_or(false);
    let apply = b_p(p, "applySettings").unwrap_or_else(|| s.project.comps().next().is_none());
    let services = s.services.clone();
    let base: Option<PathBuf> = Path::new(&path).parent().filter(|v| !v.as_os_str().is_empty()).map(Path::to_path_buf);
    let resolve_path = move |raw: &str| -> String {
        if raw.is_empty() || raw.contains("://") || raw.as_bytes().get(1) == Some(&b':') || Path::new(raw).is_absolute() {
            raw.to_string()
        } else {
            base.as_ref().map(|b| b.join(raw).to_string_lossy().into_owned()).unwrap_or_else(|| raw.to_string())
        }
    };
    let exists = |f: &str| services.exists(&resolve_path(f));
    let has_font = |font: &str| !effectcraft_text::resolve(font, "Regular").missing;
    let name = Path::new(&path).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| "After Effects Project".into());
    let options = ImportOptions { allow_losses, exists: &exists, has_font: &has_font, folder_name: &name, settings: apply };
    if dry_run {
        let report = effectcraft_aep::import::analyze_with(&bytes, &options).map_err(|e| EngineError::Other(format!("cannot analyze {path}: {e}")))?;
        return Ok(json!({"dryRun": true, "report": report}));
    }
    let importer = s.importer.clone();
    let result = s.edit("Import After Effects Project", None, |project, state| {
        let r = effectcraft_aep::import::import(project, &bytes, &options).map_err(|e| match e {
            AepImportError::Refused(_) => EngineError::Other(format!("{e} (nothing imported; pass allowLosses or use dryRun)")),
            other => EngineError::Other(format!("cannot import {path}: {other}")),
        })?;
        if apply { project.settings = r.settings.clone(); }
        let created: std::collections::HashSet<_> = r.items.iter().copied().collect();
        for item in project.items.values_mut().filter(|i| created.contains(&i.id)) {
            if let ItemKind::Footage(f) = &mut item.kind {
                f.path = resolve_path(&f.path);
                if let Some(media) = &importer && f.kind == FootageKind::Video && !f.missing
                    && let Ok(probed) = media.probe(&f.path) { f.has_audio = probed.has_audio; }
            }
        }
        state.project_selection = vec![r.folder];
        Ok(r)
    })?;
    if let Some(first) = result.comps.first() { s.open_comp(*first); }
    let losses = result.report.total_losses();
    s.toast(if losses == 0 { format!("Imported {name}") } else { format!("Imported {name} ({losses} unsupported items not imported)") });
    Ok(json!({"folder": result.folder.0, "comps": result.comps.iter().map(|c| c.0).collect::<Vec<_>>(), "items": result.items.iter().map(|i| i.0).collect::<Vec<_>>(), "report": result.report}))
}

pub fn specs() -> Vec<CommandSpec> {
    vec![cmd!("file.importAep", "After Effects Project...", ["File", "Import"], None,
        "{path (.aep or .aepx), allowLosses?: bool, dryRun?: bool, applySettings?: bool}", always, import_aep)]
}
