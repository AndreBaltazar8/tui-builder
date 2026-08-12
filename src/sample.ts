import type { BlockConfig, CellStyle, LayoutNode, Node, Project, WidgetConfig, WidgetNode } from "./types";
import { createId } from "./types";

export const palette: { name: string; description: string; category: string; config: () => WidgetConfig }[] = [
  { name: "Paragraph", description: "Styled multiline text", category: "Content", config: () => ({ kind: "paragraph", text: "Your terminal UI starts here.", alignment: "left", wrap: true }) },
  { name: "List", description: "Selectable items", category: "Data", config: () => ({ kind: "list", items: ["Dashboard", "Deployments", "Settings"], selected: 0 }) },
  { name: "Table", description: "Rows and columns", category: "Data", config: () => ({ kind: "table", headers: ["Service", "Status", "CPU"], rows: [["api", "healthy", "12%"], ["worker", "healthy", "31%"], ["cron", "idle", "2%"]], widths: [{ kind: "fill", value: 2 }, { kind: "fill", value: 1 }, { kind: "length", value: 6 }], selected: 0 }) },
  { name: "Tabs", description: "Horizontal navigation", category: "Navigation", config: () => ({ kind: "tabs", labels: ["Overview", "Logs", "Metrics"], selected: 0 }) },
  { name: "Bar chart", description: "Categorical values", category: "Charts", config: () => ({ kind: "barChart", bars: [{ label: "Mon", value: 42 }, { label: "Tue", value: 68 }, { label: "Wed", value: 55 }, { label: "Thu", value: 83 }], barWidth: 5, gap: 2 }) },
  { name: "Chart", description: "Line or scatter data", category: "Charts", config: () => ({ kind: "chart", datasets: [{ name: "Requests", graph: "line", color: named("cyan"), points: [[0, 20], [20, 48], [40, 35], [60, 76], [80, 59], [100, 92]] }], xBounds: [0, 100], yBounds: [0, 100] }) },
  { name: "Calendar", description: "Monthly calendar", category: "Data", config: () => ({ kind: "calendar", year: 2026, month: 8, selectedDay: 12 }) },
  { name: "Canvas", description: "Coordinate-based shapes", category: "Charts", config: () => ({ kind: "canvas", xBounds: [0, 100], yBounds: [0, 100], shapes: [{ kind: "rectangle", x: 8, y: 10, width: 84, height: 70, color: named("darkGray") }, { kind: "line", x1: 10, y1: 15, x2: 88, y2: 72, color: named("lightCyan") }, { kind: "circle", x: 72, y: 64, radius: 8, color: named("yellow") }, { kind: "label", x: 13, y: 62, text: "latency", color: named("white") }] }) },
  { name: "Gauge", description: "Progress bar", category: "Feedback", config: () => ({ kind: "gauge", ratio: 0.68, label: "68%" }) },
  { name: "Line gauge", description: "Compact progress", category: "Feedback", config: () => ({ kind: "lineGauge", ratio: 0.42, label: "Build 42%" }) },
  { name: "Sparkline", description: "Compact time series", category: "Charts", config: () => ({ kind: "sparkline", data: [2, 5, 3, 8, 6, 9, 4, 7, 10, 8, 12, 9] }) },
  { name: "Scrollbar", description: "Scroll position", category: "Navigation", config: () => ({ kind: "scrollbar", contentLength: 100, position: 28, orientation: "verticalRight" }) },
  { name: "Fill", description: "Repeated cell symbol", category: "Utility", config: () => ({ kind: "fill", symbol: "·" }) },
  { name: "Clear", description: "Clear an overlay area", category: "Utility", config: () => ({ kind: "clear" }) },
  { name: "Shadow", description: "Block shadow layer", category: "Utility", config: () => ({ kind: "shadow" }) },
  { name: "Ratatui logo", description: "Official text logo", category: "Brand", config: () => ({ kind: "ratatuiLogo", small: true }) },
  { name: "Ratatui mascot", description: "Official terminal rat", category: "Brand", config: () => ({ kind: "ratatuiMascot", redEye: false }) },
];

export const named = (value: "cyan" | "white" | "yellow" | "darkGray" | "lightCyan" | "green" | "red") => ({ kind: "named" as const, value });
export const defaultStyle = (foreground = named("white")): CellStyle => ({ foreground });
export const defaultBlock = (title = ""): BlockConfig => ({ title, borders: "all", borderType: "rounded", padding: [0, 1, 0, 1], style: { foreground: named("darkGray") }, shadow: false });

export function widgetNode(name: string, widget: WidgetConfig, block?: BlockConfig): WidgetNode {
  return { type: "widget", id: createId(), name, widget, style: defaultStyle(), block };
}
export function layoutNode(name: string, direction: "horizontal" | "vertical", children: { constraint: { kind: "fill" | "length"; value: number }; node: Node }[]): LayoutNode {
  return { type: "layout", id: createId(), name, direction, margin: 0, spacing: 0, flex: "legacy", children };
}

export function createSampleProject(): Project {
  const header = widgetNode("Header", { kind: "tabs", labels: [" OVERVIEW ", " SERVICES ", " LOGS "], selected: 0 }, defaultBlock("ORBIT CONTROL"));
  header.style = { foreground: named("lightCyan"), bold: true };
  const sidebar = widgetNode("Navigation", { kind: "list", items: ["● Dashboard", "  Deployments", "  Machines", "  Alerts", "  Settings"], selected: 0 }, defaultBlock("NAVIGATION"));
  const chart = widgetNode("Request volume", palette.find((item) => item.name === "Chart")!.config(), defaultBlock("REQUEST VOLUME / 24H"));
  const gauge = widgetNode("Health", { kind: "gauge", ratio: 0.87, label: "SYSTEM HEALTH 87%" }, defaultBlock("STATUS"));
  gauge.style = { foreground: named("green"), bold: true };
  const table = widgetNode("Services", palette.find((item) => item.name === "Table")!.config(), defaultBlock("LIVE SERVICES"));
  const main = layoutNode("Main content", "vertical", [
    { constraint: { kind: "fill", value: 2 }, node: chart },
    { constraint: { kind: "length", value: 5 }, node: gauge },
    { constraint: { kind: "fill", value: 1 }, node: table },
  ]);
  main.spacing = 1;
  const body = layoutNode("Body", "horizontal", [
    { constraint: { kind: "length", value: 24 }, node: sidebar },
    { constraint: { kind: "fill", value: 1 }, node: main },
  ]);
  body.spacing = 1;
  const root = layoutNode("Desktop layout", "vertical", [
    { constraint: { kind: "length", value: 4 }, node: header },
    { constraint: { kind: "fill", value: 1 }, node: body },
  ]);
  root.margin = 1;
  return { schemaVersion: 1, name: "Orbit Control", frames: [{ id: createId(), name: "Operations dashboard", x: 170, y: 110, columns: 100, rows: 34, root, overlays: [] }] };
}
