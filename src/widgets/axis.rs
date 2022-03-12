use femtovg::{Paint, Path};
use glam::Vec2;
use vizia::*;

use crate::util::{BoundingBoxExt, StyleExt};

/// Controls a single point along a normalized XY axis `(-1,-1)..=(1,1)`.
pub struct Axis<P>
where
    P: Lens<Target = Vec2>,
{
    point: P,
    on_changing_point: Option<Box<dyn Fn(&mut Context, Vec2)>>,
}

impl<P> Axis<P>
where
    P: Lens<Target = Vec2>,
{
    pub fn new(cx: &mut Context, point: P) -> Handle<Self> {
        Self {
            point,
            on_changing_point: None,
        }
        .build(cx)
    }
}

impl<P> View for Axis<P>
where
    P: Lens<Target = Vec2>,
{
    fn element(&self) -> Option<String> {
        Some("axis".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {}

    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let rect = cx.cache.get_bounds(cx.current);
        let bg = cx.style.background_color(cx);
        let border = cx.style.border_color(cx);

        // Draw background shapes
        {
            // Background
            let mut path = Path::new();
            path.rect(rect.x, rect.y, rect.w, rect.h);
            canvas.fill_path(&mut path, Paint::color(bg.into()));
        }
        {
            // XY center lines
            let mut path = Path::new();
            path.move_to(rect.center_x(), rect.top());
            path.line_to(rect.center_x(), rect.bottom());
            path.move_to(rect.left(), rect.center_y());
            path.line_to(rect.right(), rect.center_y());
            canvas.stroke_path(&mut path, Paint::color(border.into()));
        }
    }
}

pub trait AxisHandle<P>
where
    P: Lens<Target = Vec2>,
{
    fn on_changing_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, Vec2);
}

impl<'a, P> AxisHandle<P> for Handle<'a, Axis<P>>
where
    P: Lens<Target = Vec2>,
{
    fn on_changing_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, Vec2),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(axis) = view.downcast_mut::<Axis<P>>() {
                axis.on_changing_point = Some(Box::new(callback));
            }
        }
        self
    }
}
