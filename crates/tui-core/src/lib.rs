//! Shared project model, renderer, validator, and exporter for TUI Builder.

mod export;
mod model;
mod render;

pub use export::*;
pub use model::*;
pub use render::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn validate_project(json: &str) -> String {
    match serde_json::from_str::<Project>(json) {
        Ok(project) => serde_json::to_string(&validate(&project)).unwrap_or_else(|_| "[]".into()),
        Err(error) => serde_json::to_string(&vec![ValidationIssue {
            path: "$".into(),
            message: error.to_string(),
            severity: "error".into(),
        }])
        .unwrap_or_else(|_| "[]".into()),
    }
}

#[wasm_bindgen]
pub fn render_frame(
    json: &str,
    frame_id: &str,
    columns: u16,
    rows: u16,
) -> Result<String, JsValue> {
    let project: Project =
        serde_json::from_str(json).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let result = render_project_frame(&project, frame_id, Some(columns), Some(rows))
        .map_err(|error| JsValue::from_str(&error))?;
    serde_json::to_string(&result).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn export_ratatui(json: &str) -> Result<String, JsValue> {
    let project: Project =
        serde_json::from_str(json).map_err(|error| JsValue::from_str(&error.to_string()))?;
    let files = generate_ratatui_project(&project).map_err(|error| JsValue::from_str(&error))?;
    serde_json::to_string(&files).map_err(|error| JsValue::from_str(&error.to_string()))
}

#[wasm_bindgen]
pub fn project_schema() -> String {
    r#"{"title":"TUI Builder Project","schemaVersion":1,"root":"Project","nodeTypes":["layout","widget"],"constraintKinds":["fill","length","min","max","percentage","ratio"]}"#.into()
}
