import { Focus, Minus, Plus } from "lucide-react";
import { useEffect, useMemo, useRef, useState } from "react";
import { getSelectedFrame, useBuilder, widgetFromPalette } from "../store";
import { renderFrame } from "../wasm";
import type { NodeRect, Project, RenderResult, TerminalFrame } from "../types";

const CELL_W = 8; const CELL_H = 16;

function TerminalPreview({ frame, project }: { frame: TerminalFrame; project: Project }) {
  const [rendered, setRendered] = useState<RenderResult>(); const [error, setError] = useState<string>(); const selectedNodeId = useBuilder((state) => state.selectedNodeId); const selectNode = useBuilder((state) => state.selectNode);
  useEffect(() => { let current = true; renderFrame(project, frame.id, frame.columns, frame.rows).then((result) => { if (current) { setRendered(result); setError(undefined); } }).catch((reason) => current && setError(String(reason))); return () => { current = false; }; }, [project, frame.id, frame.columns, frame.rows]);
  const rows = useMemo(() => { if (!rendered) return []; return Array.from({ length: rendered.rows }, (_, y) => rendered.cells.slice(y * rendered.columns, (y + 1) * rendered.columns)); }, [rendered]);
  const selectedRect = rendered?.nodeRects.findLast((rect) => rect.id === selectedNodeId);
  const hitTest = (event: React.MouseEvent) => { if (!rendered) return; const bounds = event.currentTarget.getBoundingClientRect(); const x = Math.floor((event.clientX - bounds.left) / CELL_W); const y = Math.floor((event.clientY - bounds.top) / CELL_H); const found = rendered.nodeRects.findLast((rect) => x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height); if (found) { event.stopPropagation(); selectNode(found.id); } };
  return <div className="terminal" style={{ width: frame.columns * CELL_W, height: frame.rows * CELL_H }} onClick={hitTest}>
    {!rendered && !error && <div className="terminal-loading">Rendering Ratatui…</div>}{error && <div className="terminal-error">{error}</div>}
    {rows.map((row, y) => <div className="terminal-row" key={y}>{row.map((cell, x) => <span key={x} style={{ color: cell.foreground === "transparent" ? undefined : cell.foreground, background: cell.background === "transparent" ? undefined : cell.background, fontWeight: cell.modifiers & 4 ? 700 : undefined, fontStyle: cell.modifiers & 16 ? "italic" : undefined, textDecoration: cell.modifiers & 32 ? "underline" : undefined }}>{cell.symbol || " "}</span>)}</div>)}
    {selectedRect && <div className="node-outline" style={{ left: selectedRect.x * CELL_W, top: selectedRect.y * CELL_H, width: selectedRect.width * CELL_W, height: selectedRect.height * CELL_H }}><span>{selectedNodeId?.slice(0, 4)}</span></div>}
  </div>;
}

function FrameView({ frame, project }: { frame: TerminalFrame; project: Project }) {
  const selected = useBuilder((state) => state.selectedFrameId === frame.id); const selectFrame = useBuilder((state) => state.selectFrame); const selectNode = useBuilder((state) => state.selectNode); const updateFrame = useBuilder((state) => state.updateFrame); const addWidget = useBuilder((state) => state.addWidget);
  const [offset, setOffset] = useState({ x: 0, y: 0 }); const drag = useRef<{ x: number; y: number } | undefined>(undefined);
  const start = (event: React.PointerEvent) => { event.stopPropagation(); selectFrame(frame.id); drag.current = { x: event.clientX, y: event.clientY }; event.currentTarget.setPointerCapture(event.pointerId); };
  const move = (event: React.PointerEvent) => { if (!drag.current) return; setOffset({ x: (event.clientX - drag.current.x) / useBuilder.getState().zoom, y: (event.clientY - drag.current.y) / useBuilder.getState().zoom }); };
  const end = () => { if (!drag.current) return; drag.current = undefined; if (offset.x || offset.y) updateFrame(frame.id, { x: Math.round(frame.x + offset.x), y: Math.round(frame.y + offset.y) }); setOffset({ x: 0, y: 0 }); };
  return <article className={`design-frame ${selected ? "selected" : ""}`} style={{ transform: `translate(${frame.x + offset.x}px, ${frame.y + offset.y}px)` }} onClick={(event) => { event.stopPropagation(); selectFrame(frame.id); }} onDragOver={(event) => event.preventDefault()} onDrop={(event) => { selectFrame(frame.id); const item = widgetFromPalette(event.dataTransfer.getData("application/tui-widget")); if (item) addWidget(item.config(), item.name); }}>
    <header onPointerDown={start} onPointerMove={move} onPointerUp={end}><strong>{frame.name}</strong><span>{frame.columns} × {frame.rows}</span></header>
    <div className="frame-body" onClick={() => { selectFrame(frame.id); selectNode(frame.root.id); }}><TerminalPreview frame={frame} project={project} /></div>
  </article>;
}

export function Board() {
  const project = useBuilder((state) => state.project); const zoom = useBuilder((state) => state.zoom); const pan = useBuilder((state) => state.pan); const setViewport = useBuilder((state) => state.setViewport); const selectNode = useBuilder((state) => state.selectNode); const addFrame = useBuilder((state) => state.addFrame); const selectedFrame = useBuilder(getSelectedFrame);
  const drag = useRef<{ x: number; y: number; panX: number; panY: number } | undefined>(undefined);
  const pointerDown = (event: React.PointerEvent) => { if (event.button !== 1 && !(event.button === 0 && event.altKey)) return; drag.current = { x: event.clientX, y: event.clientY, panX: pan.x, panY: pan.y }; event.currentTarget.setPointerCapture(event.pointerId); };
  const pointerMove = (event: React.PointerEvent) => { if (drag.current) setViewport({ x: drag.current.panX + event.clientX - drag.current.x, y: drag.current.panY + event.clientY - drag.current.y }); };
  return <section className="board" onPointerDown={pointerDown} onPointerMove={pointerMove} onPointerUp={() => { drag.current = undefined; }} onClick={() => selectNode(undefined)}>
    <div className="board-world" style={{ transform: `translate(${pan.x}px, ${pan.y}px) scale(${zoom})` }}>{project.frames.map((frame) => <FrameView key={frame.id} frame={frame} project={project} />)}</div>
    {project.frames.length === 0 && <div className="empty-board"><div><strong>No terminal frames</strong><p>Create a frame to begin designing.</p><button className="button primary" onClick={addFrame}><Plus size={14} /> New frame</button></div></div>}
    <div className="zoom-controls"><button onClick={() => setViewport(pan, Math.max(0.25, zoom - 0.1))}><Minus size={14} /></button><span>{Math.round(zoom * 100)}%</span><button onClick={() => setViewport(pan, Math.min(2, zoom + 0.1))}><Plus size={14} /></button><button onClick={() => selectedFrame && setViewport({ x: 80 - selectedFrame.x * 0.8, y: 70 - selectedFrame.y * 0.8 }, 0.8)} title="Focus selection"><Focus size={14} /></button></div>
    <div className="board-hint">Alt + drag or middle mouse to pan</div>
  </section>;
}
