//! Multi-stage envelope generator widget

use std::{
    ops::RangeInclusive,
    rc::Rc,
    sync::{RwLock, RwLockReadGuard},
};

use femtovg::{Color, Paint, Path};
use vizia::*;
use Units::{Percentage, Pixels, Stretch};

use crate::util::{CurvePoint, CurvePoints};

use super::zoomer::Zoomer;

pub struct Mseg<L>
where
    L: Lens<Target = CurvePoints>,
{
    points: L,
}

#[derive(Clone, Debug, Data)]
pub struct MsegDataInternal {
    pub range: RangeInclusive<f32>,
}

impl Lens for MsegDataInternal {
    type Source = Self;

    type Target = RangeInclusive<f32>;

    fn view<O, F: FnOnce(Option<&Self::Target>) -> O>(&self, source: &Self::Source, map: F) -> O {
        map(Some(&source.range))
    }
}

impl Default for MsegDataInternal {
    fn default() -> Self {
        Self { range: 0f32..=1f32 }
    }
}

impl Model for MsegDataInternal {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast::<MsegEventInternal>() {
            match ev {
                MsegEventInternal::NoOp => todo!(),
            }
        }
    }
}

pub enum MsegEventInternal {
    NoOp,
}

impl<L: Lens<Target = CurvePoints>> Mseg<L> {
    pub fn new(cx: &mut Context, lens: L) -> Handle<Mseg<L>> {
        Self {
            points: lens.clone(),
        }
        .build2(cx, move |cx| {
            if cx.data::<MsegDataInternal>().is_none() {
                // Create some internal slider data (not exposed to the user)
                MsegDataInternal { range: 0f32..=1f32 }.build(cx);
            }
            VStack::new(cx, |cx| {
                MsegGraph::new(cx, lens);
                Zoomer::new(cx, cx.data::<MsegDataInternal>().unwrap().clone());
            });
        })
    }
}

impl<'a, L: Lens<Target = CurvePoints>> View for Mseg<L> {
    fn element(&self) -> Option<String> {
        Some("mseg".to_string())
    }
}

/// The visuals of the graph
struct MsegGraph<L> {
    points: L,
    /// The max length of this MSEG, usually representing `f32` seconds. (The
    /// minimum is always `0`).
    max: f32,
    /// The current view section, describing the first and last view points as
    /// normalized values between `0..=1`.
    view: RangeInclusive<f32>,
}

impl<L> MsegGraph<L>
where
    L: Lens<Target = CurvePoints>,
{
    pub fn new(cx: &mut Context, lens: L) -> Handle<MsegGraph<L>> {
        Self {
            points: lens,
            max: 8f32,
            view: 0f32..=1f32,
        }
        .build2(cx, |cx| {})
    }
}

impl<'a, L: Lens<Target = CurvePoints>> View for MsegGraph<L> {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {}
    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let (width, height) = (
            cx.cache.get_width(cx.current),
            cx.cache.get_height(cx.current),
        );

        let background_color: femtovg::Color = cx
            .style
            .background_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default()
            .into();

        let data_to_ui = |point: &CurvePoint| -> (f32, f32) {
            // y value is a simple scale
            let y = height - (point.y * height);
            // x value requires us to calculate our zoomed position TODO: Zoom
            // too

            // Calculate the x-offset determined by the current view zoom window
            // This value shifts points to the left and right
            // We calculate the offset by getting the view's starting x value, which is normalized.
            // We then see how much time that offsets by multiplying that normalized value times the
            // maximum X of our MSEG.
            let offset = self.view.start() * self.max;
            // Calculate the x-zoom scale to apply to points
            let scale = 1f32 / ((self.view.end() - self.view.start()) * self.max);
            let x = ((point.x * scale) - offset) * width;
            (x, y)
        };

        // points
        let points = &**self.points.get(cx);
        let ui_points = points.iter().map(data_to_ui);

        // Draw background rect
        let mut path = Path::new();
        path.rect(0f32, 0f32, width, height);
        canvas.fill_path(&mut path, Paint::color(background_color));

        // Draw points
        let mut lines = Path::new();

        for (i, (x, y)) in ui_points.enumerate() {
            if i == 0 {
                lines.move_to(x, y);
            }
            // Main node
            let mut path = Path::new();
            path.circle(x, y, 4.0);
            canvas.fill_path(&mut path, Paint::color(Color::white()));

            // Lines
            lines.line_to(x, y);
        }
        canvas.stroke_path(
            &mut lines,
            Paint::color(Color::white()).with_line_width(2f32),
        );
    }
}
