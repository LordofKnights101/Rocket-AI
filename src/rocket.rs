//! The ROCKET mascot — drawn on canvas, rendered fully static.
//!
//! Every redraw produces the exact same picture, so mouse movement / hover
//! redraws can never make it jiggle. The only visual change is the flame
//! growing while you type — a discrete state change, not an animation.

use iced::widget::canvas;
use iced::widget::canvas::Stroke;
use iced::{Color, Point, Rectangle};

// `std::time::Instant` panics on wasm32 — use the web implementation there.
#[cfg(target_arch = "wasm32")]
use web_time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

const BODY: Color = Color::from_rgb8(0xe9, 0xee, 0xfb);
const NOSE: Color = Color::from_rgb8(0xff, 0x7a, 0x18);
const WINDOW_BG: Color = Color::from_rgb8(0x10, 0x1a, 0x33);
const RING: Color = Color::from_rgb8(0x8f, 0xa3, 0xc8);
const GLOW: Color = Color::from_rgb8(0x7c, 0xf9, 0xff);
const FIN: Color = Color::from_rgb8(0xe6, 0x39, 0x46);
const FLAME_OUT: Color = Color::from_rgb8(0xff, 0x9f, 0x1c);
const FLAME_IN: Color = Color::from_rgb8(0xff, 0xd1, 0x66);

/// Rocket dimensions in "rocket space" (scaled to fit the canvas).
const W: f32 = 48.0;
const H: f32 = 66.0;

#[derive(Debug, Clone)]
pub struct Rocket {
    /// Animation clock origin.
    pub started: Instant,
    /// Increments every tick — forces a fresh frame to be drawn.
    pub tick: u64,
    /// Engine warms up while you're typing.
    pub warming: bool,
    /// Last reply — triggers a launch bounce.
    pub last_launch: Option<Instant>,
}

impl Rocket {
    pub fn view<Message: 'static>(&self) -> iced::Element<'static, Message> {
        canvas(Rocket {
            started: self.started,
            tick: self.tick,
            warming: self.warming,
            last_launch: self.last_launch,
        })
        .width(48)
        .height(78)
        .into()
    }
}

fn glow_stroke(color: Color, width: f32) -> Stroke<'static> {
    Stroke {
        width,
        style: color.into(),
        line_cap: canvas::LineCap::Round,
        ..Default::default()
    }
}

impl<Message> canvas::Program<Message> for Rocket {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        // Deterministic draw: no wall-clock time, no phase that drifts between
        // redraws — so mouse-driven redraws render an identical rocket.
        let mut frame = canvas::Frame::new(renderer, bounds.size());

        let scale = (bounds.width / W).min(bounds.height / H);

        // Flame size: static — just a bit bigger while the engine "warms up"
        // as you type.
        let flame = if self.warming { 1.35 } else { 1.0 };

        frame.with_save(|frame| {
            frame.translate(iced::Vector::new(
                bounds.width / 2.0 - (W / 2.0) * scale,
                bounds.height - H * scale - 4.0,
            ));
            frame.scale(scale);

            // --- flame (drawn first, behind the body) --------------------------
            let flame_len = 12.0 * flame;
            let flame_w = 7.0 * flame;
            let mut b = canvas::path::Builder::new();
            b.move_to(Point::new(W / 2.0 - flame_w, 58.0));
            b.quadratic_curve_to(
                Point::new(W / 2.0 - 2.0, 58.0 + flame_len * 0.6),
                Point::new(W / 2.0, 60.0 + flame_len),
            );
            b.quadratic_curve_to(
                Point::new(W / 2.0 + 2.0, 58.0 + flame_len * 0.6),
                Point::new(W / 2.0 + flame_w, 58.0),
            );
            b.close();
            frame.fill(&b.build(), FLAME_OUT);

            let mut b = canvas::path::Builder::new();
            b.move_to(Point::new(W / 2.0 - flame_w * 0.5, 59.0));
            b.quadratic_curve_to(
                Point::new(W / 2.0 - 1.0, 59.0 + flame_len * 0.35),
                Point::new(W / 2.0, 60.0 + flame_len * 0.6),
            );
            b.quadratic_curve_to(
                Point::new(W / 2.0 + 1.0, 59.0 + flame_len * 0.35),
                Point::new(W / 2.0 + flame_w * 0.5, 59.0),
            );
            b.close();
            frame.fill(&b.build(), FLAME_IN);

            // --- fins ----------------------------------------------------------
            let mut b = canvas::path::Builder::new();
            b.move_to(Point::new(11.0, 42.0));
            b.line_to(Point::new(3.0, 56.0));
            b.line_to(Point::new(12.0, 53.0));
            b.close();
            b.move_to(Point::new(37.0, 42.0));
            b.line_to(Point::new(45.0, 56.0));
            b.line_to(Point::new(36.0, 53.0));
            b.close();
            frame.fill(&b.build(), FIN);

            // --- body (capsule with tapered nose) -------------------------------
            let mut b = canvas::path::Builder::new();
            b.move_to(Point::new(24.0, 2.0));
            b.bezier_curve_to(
                Point::new(10.0, 14.0),
                Point::new(10.0, 34.0),
                Point::new(10.0, 46.0),
            );
            b.quadratic_curve_to(Point::new(10.0, 55.0), Point::new(24.0, 58.0));
            b.quadratic_curve_to(Point::new(38.0, 55.0), Point::new(38.0, 46.0));
            b.bezier_curve_to(
                Point::new(38.0, 34.0),
                Point::new(38.0, 14.0),
                Point::new(24.0, 2.0),
            );
            b.close();
            frame.fill(&b.build(), BODY);

            // --- nose cone -------------------------------------------------------
            let mut b = canvas::path::Builder::new();
            b.move_to(Point::new(24.0, 2.0));
            b.bezier_curve_to(
                Point::new(30.0, 8.0),
                Point::new(33.0, 12.0),
                Point::new(34.5, 17.0),
            );
            b.quadratic_curve_to(Point::new(24.0, 20.0), Point::new(13.5, 17.0));
            b.bezier_curve_to(
                Point::new(15.0, 12.0),
                Point::new(18.0, 8.0),
                Point::new(24.0, 2.0),
            );
            b.close();
            frame.fill(&b.build(), NOSE);

            // --- window + robot face ----------------------------------------------
            let mut b = canvas::path::Builder::new();
            b.circle(Point::new(24.0, 32.0), 8.5);
            b.close();
            let window = b.build();
            frame.fill(&window, WINDOW_BG);
            frame.stroke(&window, glow_stroke(RING, 1.5));

            let mut b = canvas::path::Builder::new();
            b.circle(Point::new(21.0, 31.0), 1.6);
            b.close();
            b.circle(Point::new(27.0, 31.0), 1.6);
            b.close();
            frame.fill(&b.build(), GLOW);

            let mut b = canvas::path::Builder::new();
            b.move_to(Point::new(20.5, 35.0));
            b.quadratic_curve_to(Point::new(24.0, 37.5), Point::new(27.5, 35.0));
            frame.stroke(&b.build(), glow_stroke(GLOW, 1.2));

            // --- antenna ------------------------------------------------------------
            let mut b = canvas::path::Builder::new();
            b.move_to(Point::new(24.0, 2.0));
            b.line_to(Point::new(24.0, -3.0));
            frame.stroke(&b.build(), glow_stroke(NOSE, 1.5));

            let mut b = canvas::path::Builder::new();
            b.circle(Point::new(24.0, -5.0), 1.8);
            b.close();
            frame.fill(&b.build(), FLAME_IN);
        });

        vec![frame.into_geometry()]
    }
}
