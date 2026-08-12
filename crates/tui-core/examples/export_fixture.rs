use std::{env, fs, path::Path};
use tui_builder_core::{Project, generate_ratatui_project};

fn main() {
    let output = env::args()
        .nth(1)
        .expect("usage: export_fixture OUTPUT_DIRECTORY");
    let project: Project = serde_json::from_str(include_str!("../../../fixtures/basic.tuib.json"))
        .expect("fixture must parse");
    for file in generate_ratatui_project(&project).expect("fixture must export") {
        let path = Path::new(&output).join(file.path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create output directory");
        }
        fs::write(path, file.contents).expect("write generated file");
    }
}
