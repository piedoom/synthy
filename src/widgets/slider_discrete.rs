use crate::util::{BoundingBoxExt, StyleExt};
use femtovg::{Paint, Path};
use vizia::*;

const GAP: f32 = 4f32;

pub struct SliderDiscrete<P>
where
    P: Lens<Target = f32>,
{
    value: P,
    steps: usize,
    vertical: bool,
    on_changing: Option<Box<dyn Fn(&mut Context, f32)>>,
}

impl<P> SliderDiscrete<P>
where
    P: Lens<Target = f32>,
{
    pub fn new(cx: &mut Context, value: P, steps: usize, vertical: bool) -> Handle<Self> {
        Self {
            value,
            steps,
            vertical,
            on_changing: None,
        }
        .build(cx)
    }
}

impl<P> View for SliderDiscrete<P>
where
    P: Lens<Target = f32>,
{
    fn element(&self) -> Option<String> {
        Some("slider-discrete".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {}

    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let rect = cx.cache.get_bounds(cx.current);
        let bg_color = cx.style.background_color(cx.current);
        let fg_color = cx.style.color(cx.current);

        // Get the size of each rect
        let dependent_side = {
            let side = match self.vertical {
                true => rect.height(),
                false => rect.width(),
            };
            (side / self.steps as f32) - GAP
        };
        let independent_side = match self.vertical {
            true => rect.width(),
            false => rect.height(),
        };
        let rects = (0..self.steps).map(|i| {
            let offset = (dependent_side + GAP) * i as f32;
            match self.vertical {
                true => BoundingBox {
                    x: rect.x,
                    y: rect.bottom() - offset,
                    w: rect.w,
                    h: dependent_side,
                },
                false => BoundingBox {
                    x: rect.left() + offset,
                    y: rect.y,
                    w: dependent_side,
                    h: rect.h,
                },
            }
        });

        for rect in rects {
            // Draw the background rects
            let mut path = Path::new();
            path.rect(rect.x, rect.y, rect.w, rect.h);
            canvas.fill_path(&mut path, Paint::color(bg_color.into()));
        }
        // Draw the filled bar
    }
}
