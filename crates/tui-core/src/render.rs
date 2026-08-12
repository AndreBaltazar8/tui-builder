use crate::model::*;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment as RAlignment, Constraint, Direction as RDirection, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::Marker,
    text::Line,
    widgets::{
        Axis, BarChart, Block, BorderType, Borders, Chart, Clear, Dataset, Fill, Gauge, GraphType,
        LineGauge, List, ListState, Padding, Paragraph, RatatuiLogo, RatatuiMascot, Row, Scrollbar,
        ScrollbarOrientation, ScrollbarState, Shadow, Sparkline, StatefulWidget, Table, TableState,
        Tabs, Widget, Wrap,
        calendar::{CalendarEventStore, Monthly},
        canvas::{
            Canvas, Circle, FilledLine, Line as CanvasLine, Map, MapResolution, Points, Rectangle,
        },
    },
};
use serde::{Deserialize, Serialize};
use time::{Date, Month};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderResult {
    pub columns: u16,
    pub rows: u16,
    pub cells: Vec<RenderedCell>,
    pub node_rects: Vec<NodeRect>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RenderedCell {
    pub symbol: String,
    pub foreground: String,
    pub background: String,
    pub modifiers: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeRect {
    pub id: String,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub path: String,
    pub message: String,
    pub severity: String,
}

pub fn validate(project: &Project) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    if project.schema_version != SCHEMA_VERSION {
        issues.push(issue(
            "schemaVersion",
            format!("Expected schema version {SCHEMA_VERSION}"),
            "error",
        ));
    }
    if project.frames.is_empty() {
        issues.push(issue(
            "frames",
            "Add at least one terminal frame",
            "warning",
        ));
    }
    let mut ids = std::collections::HashSet::new();
    for (index, frame) in project.frames.iter().enumerate() {
        if !(20..=300).contains(&frame.columns) || !(8..=120).contains(&frame.rows) {
            issues.push(issue(
                format!("frames[{index}]"),
                "Frame size must be between 20x8 and 300x120",
                "error",
            ));
        }
        validate_node(
            &frame.root,
            &format!("frames[{index}].root"),
            &mut ids,
            &mut issues,
        );
        for (overlay_index, overlay) in frame.overlays.iter().enumerate() {
            validate_node(
                &overlay.root,
                &format!("frames[{index}].overlays[{overlay_index}].root"),
                &mut ids,
                &mut issues,
            );
        }
    }
    issues
}

fn validate_node(
    node: &Node,
    path: &str,
    ids: &mut std::collections::HashSet<String>,
    issues: &mut Vec<ValidationIssue>,
) {
    if !ids.insert(node.id().to_owned()) {
        issues.push(issue(
            path,
            format!("Duplicate node id {}", node.id()),
            "error",
        ));
    }
    match node {
        Node::Layout { children, .. } => {
            if children.is_empty() {
                issues.push(issue(path, "Empty layout will render blank", "warning"));
            }
            for (index, child) in children.iter().enumerate() {
                if let ConstraintValue::Ratio([_, 0]) = child.constraint {
                    issues.push(issue(
                        format!("{path}.children[{index}]"),
                        "Ratio denominator cannot be zero",
                        "error",
                    ));
                }
                validate_node(
                    &child.node,
                    &format!("{path}.children[{index}].node"),
                    ids,
                    issues,
                );
            }
        }
        Node::Widget { widget, .. } => match widget {
            WidgetConfig::Gauge { ratio, .. } | WidgetConfig::LineGauge { ratio, .. }
                if !(0.0..=1.0).contains(ratio) =>
            {
                issues.push(issue(path, "Gauge ratio must be between 0 and 1", "error"))
            }
            WidgetConfig::Calendar { month, .. } if !(1..=12).contains(month) => issues.push(
                issue(path, "Calendar month must be between 1 and 12", "error"),
            ),
            _ => {}
        },
    }
}

fn issue(path: impl Into<String>, message: impl Into<String>, severity: &str) -> ValidationIssue {
    ValidationIssue {
        path: path.into(),
        message: message.into(),
        severity: severity.into(),
    }
}

pub fn render_project_frame(
    project: &Project,
    frame_id: &str,
    columns: Option<u16>,
    rows: Option<u16>,
) -> Result<RenderResult, String> {
    let frame = project
        .frames
        .iter()
        .find(|frame| frame.id == frame_id)
        .ok_or_else(|| format!("Unknown frame {frame_id}"))?;
    let columns = columns.unwrap_or(frame.columns).clamp(1, 400);
    let rows = rows.unwrap_or(frame.rows).clamp(1, 200);
    let area = Rect::new(0, 0, columns, rows);
    let mut buffer = Buffer::empty(area);
    let mut node_rects = Vec::new();
    render_node(&frame.root, area, &mut buffer, &mut node_rects);
    for overlay in &frame.overlays {
        let overlay_area = overlay_rect(area, overlay);
        Clear.render(overlay_area, &mut buffer);
        render_node(&overlay.root, overlay_area, &mut buffer, &mut node_rects);
    }
    let cells = buffer
        .content
        .iter()
        .map(|cell| RenderedCell {
            symbol: cell.symbol().to_owned(),
            foreground: color_to_css(cell.fg),
            background: color_to_css(cell.bg),
            modifiers: cell.modifier.bits(),
        })
        .collect();
    let warnings = validate(project)
        .into_iter()
        .filter(|i| i.severity == "warning")
        .map(|i| i.message)
        .collect();
    Ok(RenderResult {
        columns,
        rows,
        cells,
        node_rects,
        warnings,
    })
}

/// Render a designed frame directly into an existing Ratatui buffer. Generated
/// applications use this path so their runtime output shares the preview engine.
pub fn render_frame_to_buffer(
    project: &Project,
    frame_id: &str,
    area: Rect,
    buffer: &mut Buffer,
) -> Result<(), String> {
    let frame = project
        .frames
        .iter()
        .find(|frame| frame.id == frame_id)
        .ok_or_else(|| format!("Unknown frame {frame_id}"))?;
    let mut rects = Vec::new();
    render_node(&frame.root, area, buffer, &mut rects);
    for overlay in &frame.overlays {
        let overlay_area = overlay_rect(area, overlay);
        Clear.render(overlay_area, buffer);
        render_node(&overlay.root, overlay_area, buffer, &mut rects);
    }
    Ok(())
}

fn render_node(node: &Node, area: Rect, buffer: &mut Buffer, rects: &mut Vec<NodeRect>) {
    rects.push(NodeRect {
        id: node.id().into(),
        x: area.x,
        y: area.y,
        width: area.width,
        height: area.height,
    });
    match node {
        Node::Layout {
            direction,
            margin,
            spacing,
            flex,
            children,
            ..
        } => {
            if children.is_empty() {
                return;
            }
            let constraints: Vec<Constraint> = children
                .iter()
                .map(|child| to_constraint(child.constraint))
                .collect();
            let layout = Layout::new(
                match direction {
                    Direction::Horizontal => RDirection::Horizontal,
                    Direction::Vertical => RDirection::Vertical,
                },
                constraints,
            )
            .margin(*margin)
            .spacing(*spacing)
            .flex(to_flex(*flex));
            let chunks = layout.split(area);
            for (child, child_area) in children.iter().zip(chunks.iter()) {
                render_node(&child.node, *child_area, buffer, rects);
            }
        }
        Node::Widget {
            widget,
            style,
            block,
            ..
        } => {
            let mut content_area = area;
            if let Some(config) = block {
                let block_widget = make_block(config);
                content_area = block_widget.inner(area);
                block_widget.render(area, buffer);
            }
            render_widget(widget, to_style(style), content_area, buffer);
        }
    }
}

fn render_widget(config: &WidgetConfig, style: Style, area: Rect, buffer: &mut Buffer) {
    match config {
        WidgetConfig::Paragraph {
            text,
            alignment,
            wrap,
        } => Paragraph::new(text.as_str())
            .style(style)
            .alignment(to_alignment(*alignment))
            .wrap(Wrap { trim: *wrap })
            .render(area, buffer),
        WidgetConfig::List { items, selected } => {
            let mut state = ListState::default().with_selected(*selected);
            StatefulWidget::render(
                List::new(items.clone())
                    .style(style)
                    .highlight_symbol("› ")
                    .highlight_style(style.add_modifier(Modifier::REVERSED)),
                area,
                buffer,
                &mut state,
            );
        }
        WidgetConfig::Table {
            headers,
            rows,
            widths,
            selected,
        } => {
            let widths: Vec<Constraint> = if widths.is_empty() {
                (0..headers.len().max(1))
                    .map(|_| Constraint::Fill(1))
                    .collect()
            } else {
                widths.iter().copied().map(to_constraint).collect()
            };
            let table_rows: Vec<Row> = rows.iter().map(|row| Row::new(row.clone())).collect();
            let mut table = Table::new(table_rows, widths)
                .style(style)
                .highlight_symbol("› ")
                .row_highlight_style(style.add_modifier(Modifier::REVERSED));
            if !headers.is_empty() {
                table = table
                    .header(Row::new(headers.clone()).style(style.add_modifier(Modifier::BOLD)));
            }
            let mut state = TableState::default().with_selected(*selected);
            StatefulWidget::render(table, area, buffer, &mut state);
        }
        WidgetConfig::Tabs { labels, selected } => Tabs::new(labels.clone())
            .select((*selected).min(labels.len().saturating_sub(1)))
            .style(style)
            .highlight_style(style.add_modifier(Modifier::BOLD | Modifier::REVERSED))
            .divider(" │ ")
            .render(area, buffer),
        WidgetConfig::BarChart {
            bars,
            bar_width,
            gap,
        } => {
            let data: Vec<(&str, u64)> = bars
                .iter()
                .map(|bar| (bar.label.as_str(), bar.value))
                .collect();
            BarChart::default()
                .data(&data)
                .bar_width(*bar_width)
                .bar_gap(*gap)
                .style(style)
                .value_style(style.add_modifier(Modifier::BOLD))
                .render(area, buffer);
        }
        WidgetConfig::Chart {
            datasets,
            x_bounds,
            y_bounds,
        } => {
            let points: Vec<Vec<(f64, f64)>> = datasets
                .iter()
                .map(|set| {
                    set.points
                        .iter()
                        .map(|point| (point[0], point[1]))
                        .collect()
                })
                .collect();
            let sets: Vec<Dataset> = datasets
                .iter()
                .zip(points.iter())
                .map(|(set, points)| {
                    Dataset::default()
                        .name(set.name.clone())
                        .data(points)
                        .graph_type(match set.graph {
                            GraphKind::Line => GraphType::Line,
                            GraphKind::Scatter => GraphType::Scatter,
                        })
                        .marker(Marker::Braille)
                        .style(Style::default().fg(set.color.map(to_color).unwrap_or(Color::Cyan)))
                })
                .collect();
            Chart::new(sets)
                .style(style)
                .x_axis(Axis::default().bounds(*x_bounds).labels([
                    Line::from(format_number(x_bounds[0])),
                    Line::from(format_number(x_bounds[1])),
                ]))
                .y_axis(Axis::default().bounds(*y_bounds).labels([
                    Line::from(format_number(y_bounds[0])),
                    Line::from(format_number(y_bounds[1])),
                ]))
                .render(area, buffer);
        }
        WidgetConfig::Calendar {
            year,
            month,
            selected_day,
        } => {
            if let Ok(month) = Month::try_from(*month) {
                if let Ok(date) = Date::from_calendar_date(*year, month, 1) {
                    let mut events = CalendarEventStore::default();
                    if let Some(day) = selected_day
                        .and_then(|day| Date::from_calendar_date(*year, month, day).ok())
                    {
                        events.add(day, style.add_modifier(Modifier::REVERSED));
                    }
                    Monthly::new(date, events)
                        .default_style(style)
                        .show_month_header(style.add_modifier(Modifier::BOLD))
                        .show_weekdays_header(style)
                        .render(area, buffer);
                }
            }
        }
        WidgetConfig::Canvas {
            shapes,
            x_bounds,
            y_bounds,
        } => {
            Canvas::default()
                .x_bounds(*x_bounds)
                .y_bounds(*y_bounds)
                .marker(Marker::Braille)
                .paint(|ctx| {
                    for shape in shapes {
                        let white = Color::White;
                        match shape {
                            CanvasShape::Circle {
                                x,
                                y,
                                radius,
                                color,
                            } => ctx.draw(&Circle::new(
                                *x,
                                *y,
                                *radius,
                                color.map(to_color).unwrap_or(white),
                            )),
                            CanvasShape::Line {
                                x1,
                                y1,
                                x2,
                                y2,
                                color,
                            } => ctx.draw(&CanvasLine::new(
                                *x1,
                                *y1,
                                *x2,
                                *y2,
                                color.map(to_color).unwrap_or(white),
                            )),
                            CanvasShape::FilledLine {
                                x1,
                                y1,
                                x2,
                                y2,
                                fill_to_y,
                                color,
                            } => ctx.draw(&FilledLine::new(
                                *x1,
                                *y1,
                                *x2,
                                *y2,
                                *fill_to_y,
                                color.map(to_color).unwrap_or(white),
                            )),
                            CanvasShape::Rectangle {
                                x,
                                y,
                                width,
                                height,
                                color,
                            } => ctx.draw(&Rectangle::new(
                                *x,
                                *y,
                                *width,
                                *height,
                                color.map(to_color).unwrap_or(white),
                            )),
                            CanvasShape::Points { points, color } => {
                                let values: Vec<(f64, f64)> =
                                    points.iter().map(|p| (p[0], p[1])).collect();
                                ctx.draw(&Points {
                                    coords: &values,
                                    color: color.map(to_color).unwrap_or(white),
                                });
                            }
                            CanvasShape::Map {
                                high_resolution,
                                color,
                            } => ctx.draw(&Map {
                                resolution: if *high_resolution {
                                    MapResolution::High
                                } else {
                                    MapResolution::Low
                                },
                                color: color.map(to_color).unwrap_or(white),
                            }),
                            CanvasShape::Label { x, y, text, color } => ctx.print(
                                *x,
                                *y,
                                Line::styled(
                                    text.clone(),
                                    Style::default().fg(color.map(to_color).unwrap_or(white)),
                                ),
                            ),
                        }
                    }
                })
                .render(area, buffer);
        }
        WidgetConfig::Clear => Clear.render(area, buffer),
        WidgetConfig::Fill { symbol } => Fill::new(if symbol.is_empty() {
            " "
        } else {
            symbol.as_str()
        })
        .style(style)
        .render(area, buffer),
        WidgetConfig::Gauge { ratio, label } => Gauge::default()
            .ratio(ratio.clamp(0.0, 1.0))
            .label(if label.is_empty() {
                format!("{:.0}%", ratio * 100.0)
            } else {
                label.clone()
            })
            .gauge_style(style)
            .render(area, buffer),
        WidgetConfig::LineGauge { ratio, label } => LineGauge::default()
            .ratio(ratio.clamp(0.0, 1.0))
            .label(if label.is_empty() {
                format!("{:.0}%", ratio * 100.0)
            } else {
                label.clone()
            })
            .filled_style(style)
            .render(area, buffer),
        WidgetConfig::Scrollbar {
            content_length,
            position,
            orientation,
        } => {
            let mut state = ScrollbarState::new(*content_length).position(*position);
            StatefulWidget::render(
                Scrollbar::new(match orientation {
                    ScrollbarDirection::VerticalRight => ScrollbarOrientation::VerticalRight,
                    ScrollbarDirection::VerticalLeft => ScrollbarOrientation::VerticalLeft,
                    ScrollbarDirection::HorizontalTop => ScrollbarOrientation::HorizontalTop,
                    ScrollbarDirection::HorizontalBottom => ScrollbarOrientation::HorizontalBottom,
                })
                .style(style),
                area,
                buffer,
                &mut state,
            );
        }
        WidgetConfig::Sparkline { data } => Sparkline::default()
            .data(data)
            .style(style)
            .render(area, buffer),
        WidgetConfig::Shadow => Shadow::block().style(style).render(area, buffer),
        WidgetConfig::RatatuiLogo { small } => {
            if *small {
                RatatuiLogo::small().render(area, buffer)
            } else {
                RatatuiLogo::tiny().render(area, buffer)
            }
        }
        WidgetConfig::RatatuiMascot { red_eye } => RatatuiMascot::new()
            .set_eye(if *red_eye {
                ratatui::widgets::MascotEyeColor::Red
            } else {
                ratatui::widgets::MascotEyeColor::Default
            })
            .render(area, buffer),
    }
}

fn make_block(config: &BlockConfig) -> Block<'_> {
    let mut block = Block::default()
        .borders(match config.borders.as_str() {
            "none" => Borders::NONE,
            "top" => Borders::TOP,
            "bottom" => Borders::BOTTOM,
            "left" => Borders::LEFT,
            "right" => Borders::RIGHT,
            _ => Borders::ALL,
        })
        .border_type(match config.border_type {
            BorderKind::Plain => BorderType::Plain,
            BorderKind::Rounded => BorderType::Rounded,
            BorderKind::Double => BorderType::Double,
            BorderKind::Thick => BorderType::Thick,
            BorderKind::QuadrantInside => BorderType::QuadrantInside,
            BorderKind::QuadrantOutside => BorderType::QuadrantOutside,
        })
        .padding(Padding::new(
            config.padding[3],
            config.padding[1],
            config.padding[0],
            config.padding[2],
        ))
        .style(to_style(&config.style));
    if !config.title.is_empty() {
        block = block.title(config.title.as_str());
    }
    if config.shadow {
        block = block.shadow(Shadow::block().style(Style::default().fg(Color::DarkGray)));
    }
    block
}

fn overlay_rect(area: Rect, overlay: &Overlay) -> Rect {
    let width = resolve_constraint(overlay.width, area.width).min(area.width);
    let height = resolve_constraint(overlay.height, area.height).min(area.height);
    let x = match overlay.anchor {
        Anchor::TopLeft | Anchor::Left | Anchor::BottomLeft => area.x,
        Anchor::Top | Anchor::Center | Anchor::Bottom => {
            area.x + area.width.saturating_sub(width) / 2
        }
        _ => area.right().saturating_sub(width),
    };
    let y = match overlay.anchor {
        Anchor::TopLeft | Anchor::Top | Anchor::TopRight => area.y,
        Anchor::Left | Anchor::Center | Anchor::Right => {
            area.y + area.height.saturating_sub(height) / 2
        }
        _ => area.bottom().saturating_sub(height),
    };
    Rect::new(x, y, width, height)
}

fn resolve_constraint(value: ConstraintValue, total: u16) -> u16 {
    match value {
        ConstraintValue::Length(v) | ConstraintValue::Min(v) | ConstraintValue::Max(v) => v,
        ConstraintValue::Percentage(v) => total.saturating_mul(v.min(100)) / 100,
        ConstraintValue::Ratio([a, b]) => {
            if b == 0 {
                0
            } else {
                (u32::from(total) * a / b).min(u32::from(u16::MAX)) as u16
            }
        }
        ConstraintValue::Fill(_) => total,
    }
}
fn to_constraint(value: ConstraintValue) -> Constraint {
    match value {
        ConstraintValue::Fill(v) => Constraint::Fill(v),
        ConstraintValue::Length(v) => Constraint::Length(v),
        ConstraintValue::Min(v) => Constraint::Min(v),
        ConstraintValue::Max(v) => Constraint::Max(v),
        ConstraintValue::Percentage(v) => Constraint::Percentage(v),
        ConstraintValue::Ratio([a, b]) => Constraint::Ratio(a, b.max(1)),
    }
}
fn to_flex(value: FlexMode) -> Flex {
    match value {
        FlexMode::Legacy => Flex::Legacy,
        FlexMode::Start => Flex::Start,
        FlexMode::End => Flex::End,
        FlexMode::Center => Flex::Center,
        FlexMode::SpaceBetween => Flex::SpaceBetween,
        FlexMode::SpaceEvenly => Flex::SpaceEvenly,
        FlexMode::SpaceAround => Flex::SpaceAround,
    }
}
fn to_alignment(value: Alignment) -> RAlignment {
    match value {
        Alignment::Left => RAlignment::Left,
        Alignment::Center => RAlignment::Center,
        Alignment::Right => RAlignment::Right,
    }
}

fn to_style(value: &CellStyle) -> Style {
    let mut style = Style::default();
    if let Some(color) = value.foreground {
        style = style.fg(to_color(color));
    }
    if let Some(color) = value.background {
        style = style.bg(to_color(color));
    }
    let mut modifier = Modifier::empty();
    if value.bold {
        modifier |= Modifier::BOLD;
    }
    if value.dim {
        modifier |= Modifier::DIM;
    }
    if value.italic {
        modifier |= Modifier::ITALIC;
    }
    if value.underlined {
        modifier |= Modifier::UNDERLINED;
    }
    if value.reversed {
        modifier |= Modifier::REVERSED;
    }
    if value.crossed_out {
        modifier |= Modifier::CROSSED_OUT;
    }
    style.add_modifier(modifier)
}

fn to_color(value: TuiColor) -> Color {
    match value {
        TuiColor::Indexed(v) => Color::Indexed(v),
        TuiColor::Rgb([r, g, b]) => Color::Rgb(r, g, b),
        TuiColor::Named(v) => match v {
            NamedColor::Black => Color::Black,
            NamedColor::Red => Color::Red,
            NamedColor::Green => Color::Green,
            NamedColor::Yellow => Color::Yellow,
            NamedColor::Blue => Color::Blue,
            NamedColor::Magenta => Color::Magenta,
            NamedColor::Cyan => Color::Cyan,
            NamedColor::Gray => Color::Gray,
            NamedColor::DarkGray => Color::DarkGray,
            NamedColor::LightRed => Color::LightRed,
            NamedColor::LightGreen => Color::LightGreen,
            NamedColor::LightYellow => Color::LightYellow,
            NamedColor::LightBlue => Color::LightBlue,
            NamedColor::LightMagenta => Color::LightMagenta,
            NamedColor::LightCyan => Color::LightCyan,
            NamedColor::White => Color::White,
        },
    }
}

fn color_to_css(color: Color) -> String {
    match color {
        Color::Reset => "transparent".into(),
        Color::Black => "#000000".into(),
        Color::Red => "#cd3131".into(),
        Color::Green => "#0dbc79".into(),
        Color::Yellow => "#e5e510".into(),
        Color::Blue => "#2472c8".into(),
        Color::Magenta => "#bc3fbc".into(),
        Color::Cyan => "#11a8cd".into(),
        Color::Gray => "#cccccc".into(),
        Color::DarkGray => "#666666".into(),
        Color::LightRed => "#f14c4c".into(),
        Color::LightGreen => "#23d18b".into(),
        Color::LightYellow => "#f5f543".into(),
        Color::LightBlue => "#3b8eea".into(),
        Color::LightMagenta => "#d670d6".into(),
        Color::LightCyan => "#29b8db".into(),
        Color::White => "#ffffff".into(),
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(v) => indexed_color(v),
    }
}
fn indexed_color(value: u8) -> String {
    if value < 16 {
        let defaults = [
            "#000000", "#800000", "#008000", "#808000", "#000080", "#800080", "#008080", "#c0c0c0",
            "#808080", "#ff0000", "#00ff00", "#ffff00", "#0000ff", "#ff00ff", "#00ffff", "#ffffff",
        ];
        defaults[value as usize].into()
    } else if value < 232 {
        let n = value - 16;
        let component = |part: u8| if part == 0 { 0 } else { 55 + 40 * part };
        format!(
            "#{:02x}{:02x}{:02x}",
            component(n / 36),
            component((n / 6) % 6),
            component(n % 6)
        )
    } else {
        let v = 8 + (value - 232) * 10;
        format!("#{v:02x}{v:02x}{v:02x}")
    }
}
fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{}", value as i64)
    } else {
        format!("{value:.1}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_and_renders_basic_project() {
        let project = Project {
            schema_version: 1,
            name: "Test".into(),
            frames: vec![TerminalFrame {
                id: "frame".into(),
                name: "Frame".into(),
                x: 0.0,
                y: 0.0,
                columns: 40,
                rows: 12,
                root: Node::Widget {
                    id: "title".into(),
                    name: "Title".into(),
                    widget: WidgetConfig::Paragraph {
                        text: "Hello".into(),
                        alignment: Alignment::Center,
                        wrap: true,
                    },
                    style: CellStyle::default(),
                    block: None,
                },
                overlays: vec![],
            }],
        };
        assert!(validate(&project).is_empty());
        let result = render_project_frame(&project, "frame", None, None).unwrap();
        assert_eq!(result.cells.len(), 480);
        assert!(result.cells.iter().any(|cell| cell.symbol == "H"));
    }

    #[test]
    fn every_widget_variant_renders() {
        let widgets = vec![
            WidgetConfig::Paragraph {
                text: "Hello".into(),
                alignment: Alignment::Left,
                wrap: true,
            },
            WidgetConfig::List {
                items: vec!["One".into(), "Two".into()],
                selected: Some(0),
            },
            WidgetConfig::Table {
                headers: vec!["A".into()],
                rows: vec![vec!["B".into()]],
                widths: vec![ConstraintValue::Fill(1)],
                selected: Some(0),
            },
            WidgetConfig::Tabs {
                labels: vec!["A".into(), "B".into()],
                selected: 0,
            },
            WidgetConfig::BarChart {
                bars: vec![ValueLabel {
                    label: "A".into(),
                    value: 5,
                }],
                bar_width: 3,
                gap: 1,
            },
            WidgetConfig::Chart {
                datasets: vec![ChartDataset {
                    name: "A".into(),
                    points: vec![[0.0, 0.0], [10.0, 10.0]],
                    graph: GraphKind::Line,
                    color: None,
                }],
                x_bounds: [0.0, 10.0],
                y_bounds: [0.0, 10.0],
            },
            WidgetConfig::Calendar {
                year: 2026,
                month: 8,
                selected_day: Some(12),
            },
            WidgetConfig::Canvas {
                shapes: vec![
                    CanvasShape::Circle {
                        x: 5.0,
                        y: 5.0,
                        radius: 2.0,
                        color: None,
                    },
                    CanvasShape::Line {
                        x1: 0.0,
                        y1: 0.0,
                        x2: 10.0,
                        y2: 10.0,
                        color: None,
                    },
                    CanvasShape::FilledLine {
                        x1: 0.0,
                        y1: 2.0,
                        x2: 10.0,
                        y2: 8.0,
                        fill_to_y: 0.0,
                        color: None,
                    },
                    CanvasShape::Rectangle {
                        x: 1.0,
                        y: 1.0,
                        width: 5.0,
                        height: 4.0,
                        color: None,
                    },
                    CanvasShape::Points {
                        points: vec![[2.0, 3.0]],
                        color: None,
                    },
                    CanvasShape::Map {
                        high_resolution: false,
                        color: None,
                    },
                    CanvasShape::Label {
                        x: 2.0,
                        y: 2.0,
                        text: "x".into(),
                        color: None,
                    },
                ],
                x_bounds: [0.0, 10.0],
                y_bounds: [0.0, 10.0],
            },
            WidgetConfig::Clear,
            WidgetConfig::Fill {
                symbol: "·".into()
            },
            WidgetConfig::Gauge {
                ratio: 0.5,
                label: "50%".into(),
            },
            WidgetConfig::LineGauge {
                ratio: 0.5,
                label: "50%".into(),
            },
            WidgetConfig::Scrollbar {
                content_length: 100,
                position: 10,
                orientation: ScrollbarDirection::VerticalRight,
            },
            WidgetConfig::Sparkline {
                data: vec![1, 4, 2, 8],
            },
            WidgetConfig::Shadow,
            WidgetConfig::RatatuiLogo { small: false },
            WidgetConfig::RatatuiMascot { red_eye: true },
        ];
        for widget in widgets {
            let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 24));
            render_widget(
                &widget,
                Style::default().fg(Color::White),
                buffer.area,
                &mut buffer,
            );
        }
    }
}
