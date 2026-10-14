//! A picked Android document keeps its named mirror path and original provider destination.
use photocraft_ui_egui::FileDialogAnswer;
use serde_json::Value;

pub fn picked_files(files: &[Value]) -> Result<FileDialogAnswer, String> {
    let paths = files
        .iter()
        .map(|file| {
            file.get("path").and_then(Value::as_str).filter(|path| !path.is_empty()).map(str::to_string).ok_or_else(|| "Missing imported file path".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    // ContentsMany discards the source identity, so File > Save always asks for a new file.
    // Paths records the source even when importing on the background worker.
    Ok(FileDialogAnswer::Paths(paths))
}

#[cfg(test)]
#[path = "documents/tests.rs"]
mod tests;
