use std::ops::RangeInclusive;

use femtovg::{Paint, Path};
use vizia::{
    Actions, Binding, Color, Context, Element, Handle, Lens, LensExt, MouseButton,
    Units::{Percentage, Pixels, Stretch},
    View, WindowEvent, ZStack,
};

const HANDLE_SIZE: f32 = 16.0;
const SMALLEST_RANGE: f32 = 0.1;

pub struct Zoomer<R>
where
    R: Lens<Target = RangeInclusive<f32>>,
{
    range: R,
    status: ZoomerEvent,
    on_changing_range_end: Option<Box<dyn Fn(&mut Context, f32)>>,
    on_changing_range_start: Option<Box<dyn Fn(&mut Context, f32)>>,
}

#[derive(Debug, Clone, Copy)]
pub enum ZoomerEvent {
    SetStart,
    SetEnd,
    FinishSet,
}

impl<R> Zoomer<R>
where
    R: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(
        cx: &mut Context,
        range: R,
    ) -> Handle<Self> {
        Self {
            on_changing_range_start: None,
            on_changing_range_end: None,
            status: ZoomerEvent::FinishSet,
            range: range.clone(),
        }
        .build2(cx, move |cx| {
            let parent_entity = cx.current;

            Binding::new(cx, range.clone(), move |cx, internal| {
                ZStack::new(cx, |cx| {
                    // Active bar
                    Element::new(cx)
                        .height(Stretch(1.0))
                        .left(Pixels(0.0))
                        .right(Stretch(1.0))
                        .background_color(Color::white())
                        .class("active")
                        .bind(range.clone(), move |handle, value| {
                            let val = value.get(handle.cx);
                            let width = val.end() - val.start();
                            handle
                                .width(Percentage(width * 100.0))
                                .left(Percentage(val.start() * 100.0));
                        });

                    // Start handle
                    Element::new(cx).height(Stretch(1.0)).bind(
                        range.clone(),
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
                        range.clone(),
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

impl<R> View for Zoomer<R>
where
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn element(&self) -> Option<String> {
        Some("zoomer".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast::<ZoomerEvent>() { self.status = *ev; }
        #[allow(clippy::collapsible_match)]
        if let Some(ev) = event.message.downcast::<WindowEvent>() {
            match ev {
                // Respond to cursor movements when we are setting the start or end
                WindowEvent::MouseMove(x, _y) => {
                    let width = cx.cache.get_width(cx.current);
                    let range = self.range.get(cx);
                    // adjust X to be relative
                    let mut x = *x - cx.cache.get_bounds(cx.current).x;
                    // get new data x
                    x /= width;
                    match self.status {
                        ZoomerEvent::SetStart => {
                            // Set the zoomer amount based on the mouse positioning
                            let x = x.clamp(0f32, *range.end() - SMALLEST_RANGE);
                            if let Some(callback) = self.on_changing_range_start.take() {
                                (callback)(cx, x);
                                self.on_changing_range_start = Some(callback);
                            }
                        }
                        ZoomerEvent::SetEnd => {
                            let x = x.clamp(*range.start() + SMALLEST_RANGE, 1f32);
                            if let Some(callback) = self.on_changing_range_end.take() {
                                (callback)(cx, x);
                                self.on_changing_range_end = Some(callback);
                            }
                        }
                        _ => ()
                    }
                }
                WindowEvent::MouseDown(button) => if *button == MouseButton::Left {
                    cx.capture();
                }
                WindowEvent::MouseUp(button) => {
                    if button == &MouseButton::Left {
                        cx.emit(ZoomerEvent::FinishSet);
                        cx.release();
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

pub trait ZoomerHandle<R> where R: Lens<Target = RangeInclusive<f32>> {
    fn on_changing_start<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32);
    fn on_changing_end<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32);
}

impl<'a, R> ZoomerHandle<R> for Handle<'a, Zoomer<R>>
where
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn on_changing_start<F>(self, callback: F) -> Self 
        where F: 'static + Fn(&mut Context, f32) {
        if let Some(zoomer) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|f| f.downcast_mut::<Zoomer<R>>())
        {
            zoomer.on_changing_range_start = Some(Box::new(callback));
        }

        self
    }

    fn on_changing_end<F>(self, callback: F) -> Self
        where F: 'static + Fn(&mut Context, f32) {
        if let Some(zoomer) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|f| f.downcast_mut::<Zoomer<R>>())
        {
            zoomer.on_changing_range_end = Some(Box::new(callback));
        }

        self
    }
}