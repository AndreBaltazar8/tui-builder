import { ChevronDown, ChevronRight, Columns2, Frame, Layers3, Plus, Rows3, Search, Shapes } from "lucide-react";
import { useMemo, useState } from "react";
import { getSelectedFrame, useBuilder } from "../store";
import { palette } from "../sample";
import type { Node } from "../types";

function TreeNode({ node, depth = 0 }: { node: Node; depth?: number }) {
  const selected = useBuilder((state) => state.selectedNodeId === node.id); const select = useBuilder((state) => state.selectNode); const [open, setOpen] = useState(true);
  return <div className="tree-group"><button className={`tree-row ${selected ? "selected" : ""}`} style={{ paddingLeft: 10 + depth * 14 }} onClick={() => select(node.id)}>
    {node.type === "layout" ? <span onClick={(event) => { event.stopPropagation(); setOpen(!open); }}>{open ? <ChevronDown size={12} /> : <ChevronRight size={12} />}</span> : <span className="tree-spacer" />}
    {node.type === "layout" ? (node.direction === "horizontal" ? <Columns2 size={13} /> : <Rows3 size={13} />) : <Shapes size={13} />}<span>{node.name}</span><em>{node.type === "widget" ? node.widget.kind : ""}</em>
  </button>{node.type === "layout" && open && node.children.map((child) => <TreeNode key={child.node.id} node={child.node} depth={depth + 1} />)}</div>;
}

export function Sidebar() {
  const tab = useBuilder((state) => state.leftTab); const setTab = useBuilder((state) => state.setLeftTab); const frame = useBuilder(getSelectedFrame);
  const frames = useBuilder((state) => state.project.frames); const selectedFrame = useBuilder((state) => state.selectedFrameId); const selectFrame = useBuilder((state) => state.selectFrame); const addFrame = useBuilder((state) => state.addFrame);
  const addWidget = useBuilder((state) => state.addWidget); const addLayout = useBuilder((state) => state.addLayout); const addOverlay = useBuilder((state) => state.addOverlay); const [query, setQuery] = useState("");
  const groups = useMemo(() => { const result = new Map<string, typeof palette>(); for (const item of palette.filter((item) => item.name.toLowerCase().includes(query.toLowerCase()))) result.set(item.category, [...(result.get(item.category) ?? []), item]); return result; }, [query]);
  return <aside className="sidebar left-panel"><div className="panel-tabs"><button className={tab === "layers" ? "active" : ""} onClick={() => setTab("layers")}><Layers3 size={14} /> Layers</button><button className={tab === "widgets" ? "active" : ""} onClick={() => setTab("widgets")}><Shapes size={14} /> Widgets</button></div>
    {tab === "layers" ? <>
      <div className="section-heading"><span>FRAMES</span><button onClick={addFrame} title="Add frame"><Plus size={14} /></button></div>
      <div className="frame-list">{frames.map((item) => <button key={item.id} className={selectedFrame === item.id ? "active" : ""} onClick={() => selectFrame(item.id)}><Frame size={13} /><span>{item.name}</span><em>{item.columns}×{item.rows}</em></button>)}</div>
      <div className="section-heading"><span>LAYERS</span><span>{frame ? "1 frame" : ""}</span></div><div className="tree">{frame && <TreeNode node={frame.root} />}{frame && frame.overlays.length > 0 && <div className="overlay-label">OVERLAYS</div>}{frame?.overlays.map((overlay) => <TreeNode key={overlay.id} node={overlay.root} />)}</div>
      {frame && <button className="add-overlay" onClick={addOverlay}><Plus size={13} /> Add overlay</button>}
    </> : <>
      <label className="search"><Search size={13} /><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search widgets" /></label>
      <div className="layout-tools"><button onClick={() => addLayout("horizontal")}><Columns2 size={17} /><span>Row</span></button><button onClick={() => addLayout("vertical")}><Rows3 size={17} /><span>Column</span></button></div>
      <div className="palette">{[...groups].map(([category, items]) => <section key={category}><h3>{category}</h3>{items.map((item) => <button key={item.name} draggable onDragStart={(event) => event.dataTransfer.setData("application/tui-widget", item.name)} onClick={() => addWidget(item.config(), item.name)}><span className="widget-glyph">{item.name.slice(0, 2).toUpperCase()}</span><span><strong>{item.name}</strong><small>{item.description}</small></span><Plus size={13} /></button>)}</section>)}</div>
    </>}
  </aside>;
}
