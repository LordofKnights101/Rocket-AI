//! 🌳 The quest-tree graph — a constellation of scholarly trials.
//!
//! An interactive canvas: circular nodes (❓ available, ✓ sealed, · locked)
//! laid out in an automatic hierarchy that mirrors prerequisite relations,
//! with glowing edges lighting the forward path as nodes seal. Clicking a
//! node selects it — the synchronized text panel beside it shows the full
//! definition and the Mark Done action.

use crate::codex_tree::QuestTree;
use iced::widget::canvas;
use iced::{alignment, Color, Point, Rectangle};

/// Layout constants shared by draw + hit-testing.
const NODE_RADIUS: f32 = 15.0;
const MARGIN: f32 = 36.0;

/// Visual palette handed in from the theme.
#[derive(Debug, Clone, Copy)]
pub struct GraphColors {
    pub surface: Color,
    pub border: Color,
    pub highlight: Color,
    pub text: Color,
    pub text_dim: Color,
    pub danger: Color,
    pub dark: Color,
}

/// One rendered node (layout already resolved by `codex_tree::layout`).
#[derive(Debug, Clone)]
struct GNode {
    id: String,
    tier: u32,
    state: u8, // 0 locked, 1 available, 2 completed
    optional: bool,
    stale: bool,
}

/// The canvas program. `on_click` maps a node id to the app message.
pub struct TreeGraph<Message> {
    nodes: Vec<GNode>,
    edges: Vec<(usize, usize)>,
    selected: Option<String>,
    colors: GraphColors,
    on_click: Box<dyn Fn(String) -> Message>,
    started: std::time::Instant,
}

impl<Message> TreeGraph<Message> {
    pub fn new(
        tree: &QuestTree,
        selected: Option<&str>,
        colors: GraphColors,
        on_click: Box<dyn Fn(String) -> Message>,
    ) -> Self {
        let laid = crate::codex_tree::layout(tree);
        let edges = crate::codex_tree::layout_edges(tree, &laid);
        let nodes = laid
            .into_iter()
            .map(|g| GNode {
                id: g.id,
                tier: g.tier,
                state: g.state,
                optional: g.optional,
                stale: g.stale,
            })
            .collect();
        Self {
            nodes,
            edges,
            selected: selected.map(|s| s.to_string()),
            colors,
            on_click,
            started: std::time::Instant::now(),
        }
    }

    /// Node centers inside `bounds`: tiers as columns (left → right), rows
    /// centered vertically per column. Mirrors between draw and hit-test.
    fn centers(&self, bounds: Rectangle) -> Vec<(Point, &GNode)> {
        let mut tiers: std::collections::BTreeMap<u32, Vec<&GNode>> = Default::default();
        for n in &self.nodes {
            tiers.entry(n.tier).or_default().push(n);
        }
        let col_count = tiers.keys().copied().max().unwrap_or(0) + 1;
        let width = (bounds.width - 2.0 * MARGIN).max(60.0);
        let height = (bounds.height - 2.0 * MARGIN).max(60.0);
        let col_w = width / col_count as f32;
        let mut out = Vec::new();
        for (&tier, nodes) in &tiers {
            let count = nodes.len();
            let row_h = if count > 1 {
                (height / (count - 1) as f32).min(70.0)
            } else {
                0.0
            };
            let block = row_h * (count as f32 - 1.0);
            let top = MARGIN + (height - block) / 2.0;
            let x = MARGIN + col_w * (tier as f32 + 0.5);
            for (row, n) in nodes.iter().enumerate() {
                let y = if count == 1 {
                    MARGIN + height / 2.0
                } else {
                    top + row_h * row as f32
                };
                out.push((Point::new(x, y), *n));
            }
        }
        out
    }
}

fn edge_path(a: Point, b: Point) -> canvas::Path {
    let mut b2 = canvas::path::Builder::new();
    b2.move_to(a);
    b2.line_to(b);
    b2.build()
}

fn ring_path(c: Point, r: f32) -> canvas::Path {
    let mut b = canvas::path::Builder::new();
    b.circle(c, r);
    b.build()
}

fn check_path(c: Point) -> canvas::Path {
    let mut b = canvas::path::Builder::new();
    b.move_to(Point::new(c.x - 6.0, c.y + 0.5));
    b.line_to(Point::new(c.x - 1.5, c.y + 5.5));
    b.line_to(Point::new(c.x + 6.5, c.y - 5.0));
    b.build()
}

impl<Message: Clone> canvas::Program<Message> for TreeGraph<Message> {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: iced::mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let iced::event::Event::Mouse(iced::mouse::Event::ButtonPressed(
            iced::mouse::Button::Left,
        )) = event
        {
            if let Some(pos) = cursor.position_in(bounds) {
                for (center, node) in self.centers(bounds) {
                    let dx = pos.x - center.x;
                    let dy = pos.y - center.y;
                    if dx * dx + dy * dy <= (NODE_RADIUS + 6.0) * (NODE_RADIUS + 6.0) {
                        return Some(
                            canvas::Action::publish((self.on_click)(node.id.clone()))
                                .and_capture(),
                        );
                    }
                }
            }
        }
        None
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let c = self.colors;
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        frame.fill_rectangle(
            Point::new(0.0, 0.0),
            bounds.size(),
            Color::from_rgba8(0x0d, 0x12, 0x24, 1.0),
        );

        let centers = self.centers(bounds);
        let center_of = |idx: usize| -> Point {
            centers
                .get(idx)
                .map(|(p, _)| *p)
                .unwrap_or(Point::new(bounds.x, bounds.y))
        };

        // --- edges: dim for locked paths, glowing for lit ones ---------------
        let pulse = 0.75 + 0.25 * (self.started.elapsed().as_secs_f32() * 2.2).sin();
        for (from, to) in &self.edges {
            let a = center_of(*from);
            let b = center_of(*to);
            let lit = self.nodes.get(*from).map(|n| n.state == 2).unwrap_or(false);
            let mut stroke = canvas::Stroke {
                width: if lit { 2.6 * pulse } else { 1.2 },
                style: if lit {
                    Color { a: 0.95, ..c.highlight }.into()
                } else {
                    Color { a: 0.5, ..c.border }.into()
                },
                ..Default::default()
            };
            let path = edge_path(a, b);
            if lit {
                // Soft under-glow for lit paths.
                let mut glow = stroke;
                glow.width = 6.0 * pulse;
                glow.style = Color { a: 0.22, ..c.highlight }.into();
                frame.stroke(&path, glow);
            }
            frame.stroke(&path, stroke);
        }

        // --- nodes ------------------------------------------------------------
        for (center, node) in &centers {
            let selected = self.selected.as_deref() == Some(node.id.as_str());
            let ring_color = if node.stale {
                c.danger
            } else {
                match node.state {
                    2 | 1 => c.highlight,
                    _ => c.border,
                }
            };
            // Body.
            let body = ring_path(*center, NODE_RADIUS);
            frame.fill(
                &body,
                match node.state {
                    2 => Color { a: 0.9, ..c.highlight },
                    1 => Color { a: 1.0, ..c.surface },
                    _ => Color { a: 0.6, ..c.dark },
                },
            );
            frame.stroke(
                &ring_path(*center, NODE_RADIUS),
                canvas::Stroke {
                    width: match node.state {
                        1 => 2.4,
                        2 => 1.6,
                        _ => 1.0,
                    },
                    style: ring_color.into(),
                    ..Default::default()
                },
            );
            if selected {
                frame.stroke(
                    &ring_path(*center, NODE_RADIUS + 4.0),
                    canvas::Stroke {
                        width: 1.8,
                        style: Color { a: 0.9, ..c.text }.into(),
                        ..Default::default()
                    },
                );
            }
            if node.optional {
                // Optional side-branch: a thin outer halo.
                frame.stroke(
                    &ring_path(*center, NODE_RADIUS + 2.0),
                    canvas::Stroke {
                        width: 0.8,
                        style: Color { a: 0.5, ..c.text_dim }.into(),
                        ..Default::default()
                    },
                );
            }
            // Emblem: ✓ drawn as strokes (font-independent); ? as text.
            match node.state {
                2 => {
                    frame.stroke(
                        &check_path(*center),
                        canvas::Stroke {
                            width: 2.6,
                            style: c.dark.into(),
                            line_cap: canvas::LineCap::Round,
                            ..Default::default()
                        },
                    );
                }
                1 => {
                    frame.fill_text(canvas::Text {
                        content: "?".into(),
                        position: *center,
                        color: c.highlight,
                        size: 16.0.into(),
                        align_x: alignment::Horizontal::Center.into(),
                        align_y: alignment::Vertical::Center,
                        ..Default::default()
                    });
                }
                _ => {
                    frame.fill(
                        &ring_path(Point::new(center.x, center.y), 3.0),
                        Color { a: 0.7, ..c.text_dim },
                    );
                }
            }
        }
        vec![frame.into_geometry()]
    }
}
