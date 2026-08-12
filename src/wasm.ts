import type { GeneratedFile, Project, RenderResult, ValidationIssue } from "./types";

type CoreModule = {
  default: () => Promise<unknown>;
  render_frame: (json: string, frameId: string, columns: number, rows: number) => string;
  validate_project: (json: string) => string;
  export_ratatui: (json: string) => string;
};

let modulePromise: Promise<CoreModule> | undefined;
async function core(): Promise<CoreModule> {
  modulePromise ??= import("./wasm/tui_core.js").then(async (module) => { await module.default(); return module as CoreModule; });
  return modulePromise;
}

export async function renderFrame(project: Project, frameId: string, columns: number, rows: number): Promise<RenderResult> {
  return JSON.parse((await core()).render_frame(JSON.stringify(project), frameId, columns, rows));
}
export async function validateProject(project: Project): Promise<ValidationIssue[]> {
  return JSON.parse((await core()).validate_project(JSON.stringify(project)));
}
export async function exportRatatui(project: Project): Promise<GeneratedFile[]> {
  return JSON.parse((await core()).export_ratatui(JSON.stringify(project)));
}
