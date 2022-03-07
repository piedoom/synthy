use std::{marker::PhantomData, ops::RangeInclusive};

use femtovg::{Paint, Path};
use vizia::{
    Binding, Color, Context, Data, Element, Handle, Lens, LensExt, Model, MouseButton, Orientation,
    Press,
    Units::{Percentage, Pixels, Stretch},
    View, WindowEvent,
};

pub struct Zoomer<L>
where
    L: Lens<Target = RangeInclusive<f32>>,
{
    _phantom_data: PhantomData<L>,
    thumb_size: f32,
    on_change: Option<Box<dyn Fn(&mut Context, f32)>>,
    is_dragging: bool,
}

#[derive(Clone, Debug, Lens, Data)]
pub struct ZoomerDataInternal {
    pub thumb_size: f32,
    pub range: RangeInclusive<f32>,
}

impl Default for ZoomerDataInternal {
    fn default() -> Self {
        Self {
            thumb_size: 16f32,
            range: 0f32..=1f32,
        }
    }
}

impl Model for ZoomerDataInternal {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast::<ZoomerEventInternal>() {
            match ev {
                ZoomerEventInternal::SetRight(v) => todo!(),
                ZoomerEventInternal::SetLeft(v) => todo!(),
            }
        }
    }
}

#[derive(Debug)]
pub enum ZoomerEventInternal {
    /// Set the normalized position of the right view
    SetRight(f32),
    /// Set the normalized position of the left view
    SetLeft(f32),
}

impl<L> Zoomer<L>
where
    L: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(cx: &mut Context, lens: L) -> Handle<Self> {
        Self {
            _phantom_data: PhantomData::default(),
            thumb_size: 8f32,
            on_change: Default::default(),
            is_dragging: false,
        }
        .build2(cx, move |cx| {
            if cx.data::<ZoomerDataInternal>().is_none() {
                // Create some internal slider data (not exposed to the user)
                ZoomerDataInternal {
                    thumb_size: 0.0,
                    range: 0f32..=1f32,
                    ..Default::default()
                }
                .build(cx);
            }

            Binding::new(cx, ZoomerDataInternal::root, move |cx, internal| {
                // build
                Element::new(cx)
                    .height(Stretch(1.0))
                    .left(Pixels(0.0))
                    .right(Stretch(1.0))
                    .background_color(Color::white())
                    .class("active")
                    .bind(lens.clone(), move |handle, value| {
                        let val = value.get(handle.cx);
                        let width = val.end() - val.start();
                        handle
                            .width(Percentage(width * 100.0))
                            .left(Percentage(val.start() * 100.0));
                    });
            });
        })
        .width(Stretch(1.0))
        .height(Pixels(24f32))
    }
}

impl<L> View for Zoomer<L>
where
    L: Lens<Target = RangeInclusive<f32>>,
{
    fn element(&self) -> Option<String> {
        Some("zoomer".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast() {
            match ev {
                WindowEvent::MouseDown(button) if *button == MouseButton::Left => {
                    self.is_dragging = true;
                }
                WindowEvent::MouseUp(button) if *button == MouseButton::Left => {
                    self.is_dragging = false;
                }
                _ => (),
            }
        } else if let Some(ev) = event.message.downcast() {
            match ev {
                ZoomerEventInternal::SetLeft(v) => todo!(),
                ZoomerEventInternal::SetRight(v) => todo!(),
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
    fn overlay<B>(self, cx: &mut Context, builder: B) -> Handle<'a, Self::View>
    where
        B: 'static + FnOnce(&mut Context);
}
impl<'a, V: View> ZoomerHandle<'a> for Handle<'a, V> {
    type View = V;
    fn overlay<B>(self, cx: &mut Context, builder: B) -> Handle<'a, Self::View>
    where
        B: 'static + FnOnce(&mut Context),
    {
        (builder)(cx);
        self
    }
}
