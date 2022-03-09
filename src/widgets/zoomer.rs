use std::{marker::PhantomData, ops::RangeInclusive};

use femtovg::{Paint, Path};
use vizia::{
    Binding, Color, Context, Data, Element, Handle, Lens, LensExt, Model, MouseButton, Orientation,
    Press,
    Units::{Percentage, Pixels, Stretch},
    View, WindowEvent,
};

pub struct Zoomer {
    thumb_size: f32,
    range: RangeInclusive<f32>,
    on_changing: Option<Box<dyn Fn(&mut Context, f32)>>,
    is_dragging: bool,
}

impl Model for Zoomer {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast::<ZoomerEventInternal>() {
            self.range = match ev {
                ZoomerEventInternal::SetStart(v) => *v..=*self.range.end(),
                ZoomerEventInternal::SetEnd(v) => *self.range.start()..=*v,
            }
        }
    }
}

#[derive(Debug)]
pub enum ZoomerEventInternal {
    /// Set the normalized position of the right view
    SetStart(f32),
    /// Set the normalized position of the left view
    SetEnd(f32),
}

impl Zoomer {
    pub fn new(cx: &mut Context) -> Handle<Self> {
        Self {
            thumb_size: 8f32,
            on_changing: Default::default(),
            is_dragging: false,
            range: 0f32..=1f32,
        }
        .build2(cx, move |cx| {
            // build
            Element::new(cx)
                .height(Stretch(1.0))
                .left(Pixels(0.0))
                .right(Stretch(1.0))
                .background_color(Color::white())
                .class("active");
        })
        .width(Stretch(1.0))
        .height(Pixels(24f32))
    }
}

impl View for Zoomer {
    fn element(&self) -> Option<String> {
        Some("zoomer".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast() {
            match ev {
                WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                    self.is_dragging = true;
                    self.range = 0.1..=0.6;
                }
                WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                    self.is_dragging = false;
                }
                _ => (),
            }
        }
    }

    fn draw(&self, cx: &mut Context, canvas: &mut vizia::Canvas) {
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
        // Draw background rect
        let mut path = Path::new();
        path.rect(0f32, 0f32, width, height);
        canvas.fill_path(&mut path, Paint::color(background_color));
    }
}

pub trait ZoomerHandle<'a> {
    type View: View;
    fn on_changing<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
        L: Lens<Target = RangeInclusive<f32>>;
}
impl<'a, V: View> ZoomerHandle<'a> for Handle<'a, V> {
    type View = V;

    fn on_changing<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
        L: Lens<Target = RangeInclusive<f32>>,
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(zoomer) = view.downcast_mut::<Zoomer>() {
                zoomer.on_changing = Some(Box::new(callback));
            }
        }

        self
    }
}
