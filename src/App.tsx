import { useEffect, useRef, useState } from "react";
import { get, set } from "idb-keyval";
import { Board } from "./components/Board";
import { Inspector } from "./components/Inspector";
import { Sidebar } from "./components/Sidebar";
import { Topbar } from "./components/Topbar";
import { useBuilder } from "./store";
import type { Project } from "./types";

const STORAGE_KEY = "tui-builder:last-project:v1";

export function App() {
  const replaceProject = useBuilder((state) => state.replaceProject);
  const [ready, setReady] = useState(false);
  const project = useBuilder((state) => state.project);
  const timer = useRef<number | undefined>(undefined);
  useEffect(() => { get<Project>(STORAGE_KEY).then((saved) => { if (saved?.schemaVersion === 1) replaceProject(saved); setReady(true); }).catch(() => setReady(true)); }, [replaceProject]);
  useEffect(() => { if (!ready) return; window.clearTimeout(timer.current); timer.current = window.setTimeout(() => void set(STORAGE_KEY, project), 250); return () => window.clearTimeout(timer.current); }, [project, ready]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement;
      if (target.matches("input, textarea, select") || target.isContentEditable) return;
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "z") { event.preventDefault(); event.shiftKey ? useBuilder.getState().redo() : useBuilder.getState().undo(); }
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "d") { event.preventDefault(); const state = useBuilder.getState(); if (state.selectedNodeId) state.duplicateNode(state.selectedNodeId); }
      if (event.key === "Backspace" || event.key === "Delete") { const state = useBuilder.getState(); if (state.selectedNodeId) state.removeNode(state.selectedNodeId); }
    };
    window.addEventListener("keydown", onKey); return () => window.removeEventListener("keydown", onKey);
  }, []);
  return <main className="app-shell"><Topbar /><Sidebar /><Board /><Inspector /><footer className="statusbar"><span><i className="status-dot" /> Local project</span><span>Ratatui 0.30.2 · schema v1</span><span>Autosaved</span></footer></main>;
}
