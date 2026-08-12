declare module "./wasm/tui_core.js" {
  const init: () => Promise<unknown>;
  export default init;
  export function render_frame(json: string, frameId: string, columns: number, rows: number): string;
  export function validate_project(json: string): string;
  export function export_ratatui(json: string): string;
}
