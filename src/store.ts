import { create } from "zustand";
import { createSampleProject, defaultBlock, palette, widgetNode } from "./sample";
import { createId, findNode, findNodeParent, type LayoutNode, type Node, type Project, type TerminalFrame, type WidgetConfig } from "./types";

interface Snapshot { project: Project; selectedFrameId?: string; selectedNodeId?: string }
interface BuilderState extends Snapshot {
  past: Snapshot[]; future: Snapshot[];
  zoom: number; pan: { x: number; y: number };
  leftTab: "layers" | "widgets";
  setLeftTab: (tab: "layers" | "widgets") => void;
  selectFrame: (id: string) => void;
  selectNode: (id?: string) => void;
  setViewport: (pan: { x: number; y: number }, zoom?: number) => void;
  commit: (mutate: (project: Project) => void) => void;
  undo: () => void; redo: () => void;
  replaceProject: (project: Project) => void;
  renameProject: (name: string) => void;
  addFrame: () => void; duplicateFrame: (id: string) => void; removeFrame: (id: string) => void;
  addOverlay: () => void;
  updateFrame: (id: string, patch: Partial<TerminalFrame>) => void;
  addWidget: (config: WidgetConfig, name: string) => void;
  addLayout: (direction: "horizontal" | "vertical") => void;
  updateNode: (id: string, mutate: (node: Node) => void) => void;
  removeNode: (id: string) => void;
  duplicateNode: (id: string) => void;
}

const initialProject = createSampleProject();
const snapshot = (state: Snapshot): Snapshot => structuredClone({ project: state.project, selectedFrameId: state.selectedFrameId, selectedNodeId: state.selectedNodeId });
const regenerateIds = (node: Node): Node => {
  const copy = structuredClone(node); copy.id = createId();
  if (copy.type === "layout") copy.children.forEach((child) => { child.node = regenerateIds(child.node); });
  return copy;
};

export const useBuilder = create<BuilderState>((set, get) => ({
  project: initialProject,
  selectedFrameId: initialProject.frames[0]?.id,
  selectedNodeId: initialProject.frames[0]?.root.id,
  past: [], future: [], zoom: 0.8, pan: { x: 0, y: 0 }, leftTab: "layers",
  setLeftTab: (leftTab) => set({ leftTab }),
  selectFrame: (selectedFrameId) => set({ selectedFrameId }),
  selectNode: (selectedNodeId) => set({ selectedNodeId }),
  setViewport: (pan, zoom) => set((state) => ({ pan, zoom: zoom ?? state.zoom })),
  commit: (mutate) => set((state) => {
    const before = snapshot(state); const project = structuredClone(state.project); mutate(project);
    return { project, past: [...state.past.slice(-79), before], future: [] };
  }),
  undo: () => set((state) => { const previous = state.past.at(-1); if (!previous) return state; return { ...previous, past: state.past.slice(0, -1), future: [snapshot(state), ...state.future] }; }),
  redo: () => set((state) => { const next = state.future[0]; if (!next) return state; return { ...next, past: [...state.past, snapshot(state)], future: state.future.slice(1) }; }),
  replaceProject: (project) => set((state) => ({ project, selectedFrameId: project.frames[0]?.id, selectedNodeId: project.frames[0]?.root.id, past: [...state.past, snapshot(state)], future: [] })),
  renameProject: (name) => get().commit((project) => { project.name = name; }),
  addFrame: () => {
    const source = get().project.frames[0]; const frameId = createId();
    get().commit((project) => project.frames.push(source ? { ...structuredClone(source), id: frameId, name: `Frame ${project.frames.length + 1}`, x: source.x + 120, y: source.y + 100, root: regenerateIds(source.root), overlays: [] } : { id: frameId, name: "New frame", x: 160, y: 120, columns: 80, rows: 24, root: { type: "layout", id: createId(), name: "Root", direction: "vertical", margin: 1, spacing: 0, flex: "legacy", children: [] }, overlays: [] }));
    set({ selectedFrameId: frameId });
  },
  addOverlay: () => {
    const frameId = get().selectedFrameId; if (!frameId) return;
    const root = widgetNode("Dialog content", { kind: "paragraph", text: "A centered overlay for confirmations, details, and focused workflows.", alignment: "center", wrap: true }, defaultBlock("DIALOG"));
    get().commit((project) => { const frame = project.frames.find((item) => item.id === frameId)!; frame.overlays.push({ id: createId(), name: `Overlay ${frame.overlays.length + 1}`, anchor: "center", width: { kind: "length", value: 46 }, height: { kind: "length", value: 10 }, root }); });
    set({ selectedNodeId: root.id });
  },
  duplicateFrame: (id) => { const frame = get().project.frames.find((item) => item.id === id); if (!frame) return; const copy = structuredClone(frame); copy.id = createId(); copy.name += " copy"; copy.x += 80; copy.y += 80; copy.root = regenerateIds(copy.root); get().commit((project) => project.frames.push(copy)); set({ selectedFrameId: copy.id, selectedNodeId: copy.root.id }); },
  removeFrame: (id) => { get().commit((project) => { project.frames = project.frames.filter((frame) => frame.id !== id); }); const next = get().project.frames[0]; set({ selectedFrameId: next?.id, selectedNodeId: next?.root.id }); },
  updateFrame: (id, patch) => get().commit((project) => { const frame = project.frames.find((item) => item.id === id); if (frame) Object.assign(frame, patch); }),
  addWidget: (config, name) => {
    const frame = get().project.frames.find((item) => item.id === get().selectedFrameId); if (!frame) return;
    const id = createId();
    get().commit((project) => {
      const current = project.frames.find((item) => item.id === frame.id)!;
      let target = findNode(current.root, get().selectedNodeId);
      if (target?.type !== "layout" && target) target = findNodeParent(current.root, target.id);
      if (!target || target.type !== "layout") target = current.root.type === "layout" ? current.root : undefined;
      const node = widgetNode(name, config, defaultBlock(name.toUpperCase())); node.id = id;
      if (target?.type === "layout") target.children.push({ constraint: { kind: "fill", value: 1 }, node });
      else current.root = node;
    });
    set({ selectedNodeId: id });
  },
  addLayout: (direction) => {
    const id = createId(); const frameId = get().selectedFrameId; if (!frameId) return;
    get().commit((project) => { const frame = project.frames.find((item) => item.id === frameId)!; const selected = findNode(frame.root, get().selectedNodeId); const target = selected?.type === "layout" ? selected : findNodeParent(frame.root, selected?.id ?? ""); if (target) target.children.push({ constraint: { kind: "fill", value: 1 }, node: { type: "layout", id, name: direction === "horizontal" ? "Row" : "Column", direction, margin: 0, spacing: 0, flex: "legacy", children: [] } }); });
    set({ selectedNodeId: id });
  },
  updateNode: (id, mutate) => get().commit((project) => { for (const frame of project.frames) { const node = findNode(frame.root, id) ?? frame.overlays.map((overlay) => findNode(overlay.root, id)).find(Boolean); if (node) { mutate(node); return; } } }),
  removeNode: (id) => {
    const frameId = get().selectedFrameId; if (!frameId) return;
    get().commit((project) => { const frame = project.frames.find((item) => item.id === frameId)!; const overlayIndex = frame.overlays.findIndex((overlay) => overlay.root.id === id); if (overlayIndex >= 0) { frame.overlays.splice(overlayIndex, 1); return; } const parent = findNodeParent(frame.root, id) ?? frame.overlays.map((overlay) => findNodeParent(overlay.root, id)).find(Boolean); if (parent) parent.children = parent.children.filter((child) => child.node.id !== id); });
    set({ selectedNodeId: undefined });
  },
  duplicateNode: (id) => {
    const frameId = get().selectedFrameId; if (!frameId) return; let newId: string | undefined;
    get().commit((project) => { const frame = project.frames.find((item) => item.id === frameId)!; const overlay = frame.overlays.find((item) => item.root.id === id); if (overlay) { const copy = structuredClone(overlay); copy.id = createId(); copy.name += " copy"; copy.root = regenerateIds(copy.root); newId = copy.root.id; frame.overlays.push(copy); return; } const parent = findNodeParent(frame.root, id) ?? frame.overlays.map((item) => findNodeParent(item.root, id)).find(Boolean); const index = parent?.children.findIndex((child) => child.node.id === id) ?? -1; if (parent && index >= 0) { const child = structuredClone(parent.children[index]); child.node = regenerateIds(child.node); newId = child.node.id; parent.children.splice(index + 1, 0, child); } });
    if (newId) set({ selectedNodeId: newId });
  },
}));

export const getSelectedFrame = (state: BuilderState) => state.project.frames.find((frame) => frame.id === state.selectedFrameId);
export const getSelectedNode = (state: BuilderState) => { const frame = getSelectedFrame(state); return frame ? findNode(frame.root, state.selectedNodeId) ?? frame.overlays.map((overlay) => findNode(overlay.root, state.selectedNodeId)).find(Boolean) : undefined; };
export const widgetFromPalette = (name: string) => palette.find((item) => item.name === name);
