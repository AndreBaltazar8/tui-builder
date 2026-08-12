export type Id = string;
export type Constraint =
  | { kind: "fill" | "length" | "min" | "max" | "percentage"; value: number }
  | { kind: "ratio"; value: [number, number] };

export type NamedColor =
  | "black" | "red" | "green" | "yellow" | "blue" | "magenta" | "cyan" | "gray"
  | "darkGray" | "lightRed" | "lightGreen" | "lightYellow" | "lightBlue" | "lightMagenta" | "lightCyan" | "white";
export type TuiColor = { kind: "named"; value: NamedColor } | { kind: "indexed"; value: number } | { kind: "rgb"; value: [number, number, number] };
export interface CellStyle { foreground?: TuiColor; background?: TuiColor; bold?: boolean; dim?: boolean; italic?: boolean; underlined?: boolean; reversed?: boolean; crossedOut?: boolean }
export interface BlockConfig { title: string; borders: string; borderType: "plain" | "rounded" | "double" | "thick" | "quadrantInside" | "quadrantOutside"; padding: [number, number, number, number]; style: CellStyle; shadow: boolean }

export interface LayoutNode {
  type: "layout"; id: Id; name: string; direction: "horizontal" | "vertical"; margin: number; spacing: number;
  flex: "legacy" | "start" | "end" | "center" | "spaceBetween" | "spaceEvenly" | "spaceAround";
  children: { constraint: Constraint; node: Node }[];
}
export interface WidgetNode { type: "widget"; id: Id; name: string; widget: WidgetConfig; style: CellStyle; block?: BlockConfig }
export type Node = LayoutNode | WidgetNode;

export interface ValueLabel { label: string; value: number }
export interface ChartDataset { name: string; points: [number, number][]; graph: "line" | "scatter"; color?: TuiColor }
export type CanvasShape =
  | { kind: "circle"; x: number; y: number; radius: number; color?: TuiColor }
  | { kind: "line"; x1: number; y1: number; x2: number; y2: number; color?: TuiColor }
  | { kind: "filledLine"; x1: number; y1: number; x2: number; y2: number; fillToY: number; color?: TuiColor }
  | { kind: "rectangle"; x: number; y: number; width: number; height: number; color?: TuiColor }
  | { kind: "points"; points: [number, number][]; color?: TuiColor }
  | { kind: "map"; highResolution: boolean; color?: TuiColor }
  | { kind: "label"; x: number; y: number; text: string; color?: TuiColor };

export type WidgetConfig =
  | { kind: "paragraph"; text: string; alignment: "left" | "center" | "right"; wrap: boolean }
  | { kind: "list"; items: string[]; selected?: number }
  | { kind: "table"; headers: string[]; rows: string[][]; widths: Constraint[]; selected?: number }
  | { kind: "tabs"; labels: string[]; selected: number }
  | { kind: "barChart"; bars: ValueLabel[]; barWidth: number; gap: number }
  | { kind: "chart"; datasets: ChartDataset[]; xBounds: [number, number]; yBounds: [number, number] }
  | { kind: "calendar"; year: number; month: number; selectedDay?: number }
  | { kind: "canvas"; shapes: CanvasShape[]; xBounds: [number, number]; yBounds: [number, number] }
  | { kind: "clear" }
  | { kind: "fill"; symbol: string }
  | { kind: "gauge" | "lineGauge"; ratio: number; label: string }
  | { kind: "scrollbar"; contentLength: number; position: number; orientation: "verticalRight" | "verticalLeft" | "horizontalTop" | "horizontalBottom" }
  | { kind: "sparkline"; data: number[] }
  | { kind: "shadow" }
  | { kind: "ratatuiLogo"; small: boolean }
  | { kind: "ratatuiMascot"; redEye: boolean };

export interface Overlay { id: Id; name: string; anchor: "topLeft" | "top" | "topRight" | "left" | "center" | "right" | "bottomLeft" | "bottom" | "bottomRight"; width: Constraint; height: Constraint; root: Node }
export interface TerminalFrame { id: Id; name: string; x: number; y: number; columns: number; rows: number; root: Node; overlays: Overlay[] }
export interface Project { schemaVersion: 1; name: string; frames: TerminalFrame[] }

export interface RenderedCell { symbol: string; foreground: string; background: string; modifiers: number }
export interface NodeRect { id: Id; x: number; y: number; width: number; height: number }
export interface RenderResult { columns: number; rows: number; cells: RenderedCell[]; nodeRects: NodeRect[]; warnings: string[] }
export interface ValidationIssue { path: string; message: string; severity: "error" | "warning" }
export interface GeneratedFile { path: string; contents: string }

export const createId = () => crypto.randomUUID();

export function walkNode(node: Node, visit: (node: Node) => void): void {
  visit(node);
  if (node.type === "layout") node.children.forEach((child) => walkNode(child.node, visit));
}

export function findNode(root: Node, id?: string): Node | undefined {
  if (!id) return undefined;
  if (root.id === id) return root;
  if (root.type === "layout") for (const child of root.children) { const found = findNode(child.node, id); if (found) return found; }
}

export function findNodeParent(root: Node, id: string): LayoutNode | undefined {
  if (root.type !== "layout") return undefined;
  if (root.children.some((child) => child.node.id === id)) return root;
  for (const child of root.children) { const found = findNodeParent(child.node, id); if (found) return found; }
}
