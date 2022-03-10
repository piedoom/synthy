use std::{marker::PhantomData, ops::RangeInclusive};

use femtovg::{Paint, Path};
use vizia::{
    Actions, Binding, Color, Context, Data, Element, Entity, Handle, Lens, LensExt, Model,
    MouseButton, Orientation, Press, PropSet, Release,
    Units::{Percentage, Pixels, Stretch},
    View, WindowEvent, ZStack,
};

const HANDLE_SIZE: f32 = 16.0;

pub struct Zoomer<L>
where
    L: Lens<Target = RangeInclusive<f32>>,
{
    _phantom_data: PhantomData<L>,
    status: ZoomerEvent,
    on_changing_range_end: Option<Box<dyn Fn(&mut Context, f32)>>,
    on_changing_range_start: Option<Box<dyn Fn(&mut Context, f32)>>,
}

#[derive(Clone, Debug, Lens, Data)]
pub struct ZoomerDataInternal {
    pub range: RangeInclusive<f32>,
}

impl Default for ZoomerDataInternal {
    fn default() -> Self {
        Self { range: 0f32..=1f32 }
    }
}

impl Model for ZoomerDataInternal {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast::<ZoomerEventInternal>() {
            // set min / max so that it is never out of range or invalid
            self.range = match ev {
                ZoomerEventInternal::SetViewRangeStart(x) => {
                    let x = x.max(0f32).min(*self.range.end());
                    x..=*self.range.end()
                }
                ZoomerEventInternal::SetViewRangeEnd(x) => {
                    let x = x.max(*self.range.start()).min(1f32);
                    *self.range.start()..=x
                }
            };
        }
    }
}

#[derive(Debug)]
pub enum ZoomerEventInternal {
    /// Set the normalized position of the right view
    SetViewRangeEnd(f32),
    /// Set the normalized position of the left view
    SetViewRangeStart(f32),
}

#[derive(Debug, Clone, Copy)]
pub enum ZoomerEvent {
    SetStart,
    SetEnd,
    FinishSet,
}

impl<L> Zoomer<L>
where
    L: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(cx: &mut Context, lens: L) -> Handle<Self> {
        Self {
            on_changing_range_start: Default::default(),
            on_changing_range_end: Default::default(),
            status: ZoomerEvent::FinishSet,
            _phantom_data: Default::default(),
        }
        .build2(cx, move |cx| {
            let parent_entity = cx.current;
            if cx.data::<ZoomerDataInternal>().is_none() {
                // Create some internal slider data (not exposed to the user)
                ZoomerDataInternal {
                    range: 0f32..=1f32,
                    ..Default::default()
                }
                .build(cx);
            }

            Binding::new(cx, ZoomerDataInternal::root, move |cx, internal| {
                ZStack::new(cx, |cx| {
                    // Active bar
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

                    // Start handle
                    Element::new(cx).height(Stretch(1.0)).bind(
                        lens.clone(),
                        move |handle, value| {
                            let val = value.get(handle.cx);
                            handle
                                .left(Percentage(*val.start() * 100.0))
                                .width(Pixels(HANDLE_SIZE))
                                .background_color(vizia::Color::rgb(0, 0, 255))
                                .on_press(move |cx| {
                                    cx.emit(ZoomerEvent::SetStart);
                                });
                        },
                    );

                    // End handle
                    let w = cx.cache.get_width(parent_entity);
                    Element::new(cx).height(Stretch(1.0)).bind(
                        lens.clone(),
                        move |handle, value| {
                            let val = value.get(handle.cx);
                            handle
                                .left(Stretch(1f32))
                                .right(Pixels((1f32 - *val.end()) * w))
                                .width(Pixels(HANDLE_SIZE))
                                .background_color(vizia::Color::rgb(255, 0, 0))
                                .on_press(move |cx| {
                                    cx.emit(ZoomerEvent::SetEnd);
                                });
                        },
                    );
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
        if let Some(ev) = event.message.downcast::<ZoomerEvent>() {
            match ev {
                ZoomerEvent::SetStart | ZoomerEvent::SetEnd => cx.capture(),
                ZoomerEvent::FinishSet => cx.release(),
            }
            self.status = *ev;
        }
        #[allow(clippy::collapsible_match)]
        if let Some(ev) = event.message.downcast::<WindowEvent>() {
            match ev {
                WindowEvent::MouseMove(x, _y) => {
                    let width = cx.cache.get_width(cx.current);
                    // adjust X to be relative
                    let x = *x - cx.cache.get_bounds(cx.current).x;
                    // get new data x
                    let x = x / width;
                    match self.status {
                        ZoomerEvent::SetStart => {
                            // Set the zoomer amount based on the mouse positioning
                            cx.emit(ZoomerEventInternal::SetViewRangeStart(x));
                            if let Some(callback) = self.on_changing_range_start.take() {
                                (callback)(cx, x);
                                self.on_changing_range_start = Some(callback);
                            }
                        }
                        ZoomerEvent::SetEnd => {
                            cx.emit(ZoomerEventInternal::SetViewRangeStart(x));
                            if let Some(callback) = self.on_changing_range_end.take() {
                                (callback)(cx, x);
                                self.on_changing_range_end = Some(callback);
                            }
                        }
                        ZoomerEvent::FinishSet => (),
                    }
                }
                WindowEvent::MouseUp(button) => {
                    if button == &MouseButton::Left {
                        cx.emit(ZoomerEvent::FinishSet);
                    }
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
    fn on_changing_range_start<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
        L: Lens<Target = RangeInclusive<f32>>;
    fn on_changing_range_end<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
        L: Lens<Target = RangeInclusive<f32>>;
}
impl<'a, V: View> ZoomerHandle<'a> for Handle<'a, V> {
    type View = V;

    fn on_changing_range_start<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
        L: Lens<Target = RangeInclusive<f32>>,
    {
        if let Some(zoomer) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|f| f.downcast_mut::<Zoomer<L>>())
        {
            zoomer.on_changing_range_start = Some(Box::new(callback));
        }

        self
    }

    fn on_changing_range_end<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
        L: Lens<Target = RangeInclusive<f32>>,
    {
        if let Some(zoomer) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|f| f.downcast_mut::<Zoomer<L>>())
        {
            zoomer.on_changing_range_end = Some(Box::new(callback));
        }

        self
    }
}
