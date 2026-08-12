import { useRef, useState } from "react";
import { zipSync, strToU8 } from "fflate";
import { Box, ChevronDown, Download, FileJson, Play, Redo2, Undo2 } from "lucide-react";
import { useBuilder } from "../store";
import { exportRatatui, validateProject } from "../wasm";
import type { Project } from "../types";

function downloadBlob(name: string, blob: Blob) { const url = URL.createObjectURL(blob); const link = document.createElement("a"); link.href = url; link.download = name; link.click(); setTimeout(() => URL.revokeObjectURL(url), 0); }
const slug = (value: string) => value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "tui-project";

export function Topbar() {
  const project = useBuilder((state) => state.project); const rename = useBuilder((state) => state.renameProject);
  const undo = useBuilder((state) => state.undo); const redo = useBuilder((state) => state.redo); const canUndo = useBuilder((state) => state.past.length > 0); const canRedo = useBuilder((state) => state.future.length > 0);
  const replace = useBuilder((state) => state.replaceProject); const input = useRef<HTMLInputElement>(null); const [busy, setBusy] = useState(false); const [notice, setNotice] = useState<string>();
  const exportJson = () => downloadBlob(`${slug(project.name)}.tuib.json`, new Blob([JSON.stringify(project, null, 2)], { type: "application/json" }));
  const importJson = async (file?: File) => { if (!file) return; try { const candidate = JSON.parse(await file.text()) as Project; const issues = await validateProject(candidate); const errors = issues.filter((issue) => issue.severity === "error"); if (errors.length) throw new Error(errors[0].message); replace(candidate); setNotice("Project imported"); } catch (error) { setNotice(error instanceof Error ? error.message : "Could not import project"); } finally { if (input.current) input.current.value = ""; } };
  const exportCode = async () => { setBusy(true); try { const files = await exportRatatui(project); const entries = Object.fromEntries(files.map((file) => [file.path, strToU8(file.contents)])); downloadBlob(`${slug(project.name)}-ratatui.zip`, new Blob([zipSync(entries) as BlobPart], { type: "application/zip" })); setNotice("Ratatui project exported"); } catch (error) { setNotice(error instanceof Error ? error.message : "Export failed"); } finally { setBusy(false); } };
  return <header className="topbar">
    <div className="brand"><span className="brand-mark"><Box size={15} /></span><span>TUI BUILDER</span></div>
    <div className="project-title"><input value={project.name} onChange={(event) => rename(event.target.value)} aria-label="Project name" /><ChevronDown size={13} /></div>
    <nav className="menu"><button>File</button><button>Edit</button><button>View</button></nav>
    <div className="top-actions">
      {notice && <button className="notice" onClick={() => setNotice(undefined)}>{notice}</button>}
      <button className="icon-button" onClick={undo} disabled={!canUndo} title="Undo"><Undo2 size={15} /></button><button className="icon-button" onClick={redo} disabled={!canRedo} title="Redo"><Redo2 size={15} /></button>
      <span className="divider" /><input ref={input} hidden type="file" accept=".json,.tuib.json" onChange={(event) => void importJson(event.target.files?.[0])} />
      <button className="button subtle" onClick={() => input.current?.click()}><FileJson size={14} /> Import</button><button className="button subtle" onClick={exportJson}><Download size={14} /> Design</button>
      <button className="button primary" onClick={() => void exportCode()} disabled={busy}><Play size={14} fill="currentColor" /> {busy ? "Building…" : "Export Rust"}</button>
    </div>
  </header>;
}
