use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub schema_version: u32,
    pub name: String,
    #[serde(default)]
    pub frames: Vec<TerminalFrame>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TerminalFrame {
    pub id: String,
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub columns: u16,
    pub rows: u16,
    pub root: Node,
    #[serde(default)]
    pub overlays: Vec<Overlay>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Overlay {
    pub id: String,
    pub name: String,
    pub anchor: Anchor,
    pub width: ConstraintValue,
    pub height: ConstraintValue,
    pub root: Node,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Node {
    Layout {
        id: String,
        name: String,
        direction: Direction,
        #[serde(default)]
        margin: u16,
        #[serde(default)]
        spacing: u16,
        #[serde(default)]
        flex: FlexMode,
        #[serde(default)]
        children: Vec<LayoutChild>,
    },
    Widget {
        id: String,
        name: String,
        widget: WidgetConfig,
        #[serde(default)]
        style: CellStyle,
        #[serde(default)]
        block: Option<BlockConfig>,
    },
}

impl Node {
    pub fn id(&self) -> &str {
        match self {
            Self::Layout { id, .. } | Self::Widget { id, .. } => id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutChild {
    pub constraint: ConstraintValue,
    pub node: Node,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum FlexMode {
    #[default]
    Legacy,
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum ConstraintValue {
    Fill(u16),
    Length(u16),
    Min(u16),
    Max(u16),
    Percentage(u16),
    Ratio([u32; 2]),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct CellStyle {
    #[serde(default)]
    pub foreground: Option<TuiColor>,
    #[serde(default)]
    pub background: Option<TuiColor>,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub dim: bool,
    #[serde(default)]
    pub italic: bool,
    #[serde(default)]
    pub underlined: bool,
    #[serde(default)]
    pub reversed: bool,
    #[serde(default)]
    pub crossed_out: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum TuiColor {
    Named(NamedColor),
    Indexed(u8),
    Rgb([u8; 3]),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BlockConfig {
    #[serde(default)]
    pub title: String,
    #[serde(default = "default_borders")]
    pub borders: String,
    #[serde(default)]
    pub border_type: BorderKind,
    #[serde(default)]
    pub padding: [u16; 4],
    #[serde(default)]
    pub style: CellStyle,
    #[serde(default)]
    pub shadow: bool,
}

fn default_borders() -> String {
    "all".into()
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum BorderKind {
    #[default]
    Plain,
    Rounded,
    Double,
    Thick,
    QuadrantInside,
    QuadrantOutside,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum WidgetConfig {
    Paragraph {
        text: String,
        #[serde(default)]
        alignment: Alignment,
        #[serde(default = "yes")]
        wrap: bool,
    },
    List {
        items: Vec<String>,
        #[serde(default)]
        selected: Option<usize>,
    },
    Table {
        headers: Vec<String>,
        rows: Vec<Vec<String>>,
        #[serde(default)]
        widths: Vec<ConstraintValue>,
        #[serde(default)]
        selected: Option<usize>,
    },
    Tabs {
        labels: Vec<String>,
        #[serde(default)]
        selected: usize,
    },
    BarChart {
        bars: Vec<ValueLabel>,
        #[serde(default = "default_bar_width")]
        bar_width: u16,
        #[serde(default = "default_gap")]
        gap: u16,
    },
    Chart {
        #[serde(default)]
        datasets: Vec<ChartDataset>,
        #[serde(default = "default_bounds")]
        x_bounds: [f64; 2],
        #[serde(default = "default_bounds")]
        y_bounds: [f64; 2],
    },
    Calendar {
        year: i32,
        month: u8,
        #[serde(default)]
        selected_day: Option<u8>,
    },
    Canvas {
        #[serde(default)]
        shapes: Vec<CanvasShape>,
        #[serde(default = "default_bounds")]
        x_bounds: [f64; 2],
        #[serde(default = "default_bounds")]
        y_bounds: [f64; 2],
    },
    Clear,
    Fill {
        symbol: String,
    },
    Gauge {
        ratio: f64,
        #[serde(default)]
        label: String,
    },
    LineGauge {
        ratio: f64,
        #[serde(default)]
        label: String,
    },
    Scrollbar {
        content_length: usize,
        #[serde(default)]
        position: usize,
        #[serde(default)]
        orientation: ScrollbarDirection,
    },
    Sparkline {
        data: Vec<u64>,
    },
    Shadow,
    RatatuiLogo {
        #[serde(default)]
        small: bool,
    },
    RatatuiMascot {
        #[serde(default)]
        red_eye: bool,
    },
}

fn yes() -> bool {
    true
}
fn default_bar_width() -> u16 {
    5
}
fn default_gap() -> u16 {
    1
}
fn default_bounds() -> [f64; 2] {
    [0.0, 100.0]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ValueLabel {
    pub label: String,
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChartDataset {
    pub name: String,
    #[serde(default)]
    pub points: Vec<[f64; 2]>,
    #[serde(default)]
    pub graph: GraphKind,
    #[serde(default)]
    pub color: Option<TuiColor>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum GraphKind {
    #[default]
    Line,
    Scatter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CanvasShape {
    Circle {
        x: f64,
        y: f64,
        radius: f64,
        #[serde(default)]
        color: Option<TuiColor>,
    },
    Line {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        #[serde(default)]
        color: Option<TuiColor>,
    },
    FilledLine {
        x1: f64,
        y1: f64,
        x2: f64,
        y2: f64,
        fill_to_y: f64,
        #[serde(default)]
        color: Option<TuiColor>,
    },
    Rectangle {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        #[serde(default)]
        color: Option<TuiColor>,
    },
    Points {
        points: Vec<[f64; 2]>,
        #[serde(default)]
        color: Option<TuiColor>,
    },
    Map {
        #[serde(default)]
        high_resolution: bool,
        #[serde(default)]
        color: Option<TuiColor>,
    },
    Label {
        x: f64,
        y: f64,
        text: String,
        #[serde(default)]
        color: Option<TuiColor>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum ScrollbarDirection {
    #[default]
    VerticalRight,
    VerticalLeft,
    HorizontalTop,
    HorizontalBottom,
}
