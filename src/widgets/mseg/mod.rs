//! Multi-stage envelope generator widget

pub(crate) mod graph;
pub(crate) mod util;

use self::graph::{MsegGraph, MsegGraphHandle};
use std::{marker::PhantomData, ops::RangeInclusive};

use super::zoomer::{Zoomer, ZoomerHandle};
use crate::util::CurvePoints;
use glam::Vec2;
use vizia::*;

#[allow(clippy::enum_variant_names)]
enum MsegInternalEvent {
    OnChangingRangeStart(f32),
    OnChangingRangeEnd(f32),
    OnChangingRangeBoth(RangeInclusive<f32>),
    OnChangingPoint { index: usize, point: Vec2 },
    OnRemovePoint { index: usize },
    OnInsertPoint { index: usize, point: Vec2 },
}

pub(crate) type PointCallback = Box<dyn Fn(&mut Context, usize, Vec2)>;
pub(crate) type FloatCallback = Box<dyn Fn(&mut Context, f32)>;
pub(crate) type RangeCallback = Box<dyn Fn(&mut Context, RangeInclusive<f32>)>;
pub struct Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    points: P,
    range: PhantomData<R>,
    on_remove_point: Option<Box<dyn Fn(&mut Context, usize)>>,
    on_insert_point: Option<PointCallback>,
    on_changing_point: Option<PointCallback>,
    on_changing_range_start: Option<FloatCallback>,
    on_changing_range_end: Option<FloatCallback>,
    on_changing_range_both: Option<RangeCallback>,
}

impl<P, R> Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(cx: &mut Context, points: P, range: R, max: f32) -> Handle<Mseg<P, R>> {
        Self {
            points: points.clone(),
            range: Default::default(),
            on_changing_point: None,
            on_changing_range_start: None,
            on_changing_range_end: None,
            on_changing_range_both: None,
            on_remove_point: None,
            on_insert_point: None,
        }
        .build2(cx, |cx| {
            let background_color: Color = cx
                .style
                .background_color
                .get(cx.current)
                .cloned()
                .unwrap_or_default();
            MsegGraph::new(cx, points, range.clone(), max)
                .background_color(background_color)
                .on_changing_point(|cx, index, point| {
                    cx.emit(MsegInternalEvent::OnChangingPoint { index, point })
                })
                .on_remove_point(|cx, index| cx.emit(MsegInternalEvent::OnRemovePoint { index }))
                .on_insert_point(|cx, index, point| {
                    cx.emit(MsegInternalEvent::OnInsertPoint { index, point })
                })
                .class("graph");

            Zoomer::new(cx, range.clone())
                .on_changing_start(|cx, x| cx.emit(MsegInternalEvent::OnChangingRangeStart(x)))
                .on_changing_end(|cx, x| cx.emit(MsegInternalEvent::OnChangingRangeEnd(x)))
                .on_changing_both(|cx, x| cx.emit(MsegInternalEvent::OnChangingRangeBoth(x)));
        })
    }
}

impl<P, R> View for Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn element(&self) -> Option<String> {
        Some("mseg".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        if let Some(ev) = event.message.downcast::<MsegInternalEvent>() {
            match ev {
                MsegInternalEvent::OnChangingRangeStart(x) => {
                    if let Some(callback) = self.on_changing_range_start.take() {
                        (callback)(cx, *x);
                        self.on_changing_range_start = Some(callback);
                    }
                }
                MsegInternalEvent::OnChangingRangeEnd(x) => {
                    if let Some(callback) = self.on_changing_range_end.take() {
                        (callback)(cx, *x);
                        self.on_changing_range_end = Some(callback);
                    }
                }
                MsegInternalEvent::OnChangingRangeBoth(range) => {
                    if let Some(callback) = self.on_changing_range_both.take() {
                        (callback)(cx, range.clone());
                        self.on_changing_range_both = Some(callback);
                    }
                }
                MsegInternalEvent::OnChangingPoint { index, point } => {
                    if let Some(callback) = self.on_changing_point.take() {
                        (callback)(cx, *index, *point);
                        self.on_changing_point = Some(callback);
                    }
                }
                MsegInternalEvent::OnRemovePoint { index } => {
                    // Delete the point if not the first or last in the vector
                    if *index != 0 && *index != self.points.get(cx).len() - 1 {
                        if let Some(callback) = self.on_remove_point.take() {
                            (callback)(cx, *index);
                            self.on_remove_point = Some(callback);
                        }
                    }
                }
                MsegInternalEvent::OnInsertPoint { index, point } => {
                    if let Some(callback) = self.on_insert_point.take() {
                        (callback)(cx, *index, *point);
                        self.on_insert_point = Some(callback);
                    }
                }
            }
        }
    }
}

pub trait MsegHandle<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn on_changing_range_start<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32);
    fn on_changing_range_end<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32);
    fn on_changing_range_both<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, RangeInclusive<f32>);
    fn on_changing_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2);
    fn on_insert_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2);
    fn on_remove_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize);
}

impl<'a, P, R> MsegHandle<P, R> for Handle<'a, Mseg<P, R>>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn on_changing_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(zoomer) = view.downcast_mut::<Mseg<P, R>>() {
                zoomer.on_changing_point = Some(Box::new(callback));
            }
        }

        self
    }

    fn on_changing_range_start<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(zoomer) = view.downcast_mut::<Mseg<P, R>>() {
                zoomer.on_changing_range_start = Some(Box::new(callback));
            }
        }

        self
    }

    fn on_changing_range_end<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(zoomer) = view.downcast_mut::<Mseg<P, R>>() {
                zoomer.on_changing_range_end = Some(Box::new(callback));
            }
        }

        self
    }

    fn on_changing_range_both<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, RangeInclusive<f32>),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(mseg) = view.downcast_mut::<Mseg<P, R>>() {
                mseg.on_changing_range_both = Some(Box::new(callback));
            }
        }

        self
    }

    fn on_insert_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(mseg) = view.downcast_mut::<Mseg<P, R>>() {
                mseg.on_insert_point = Some(Box::new(callback));
            }
        }

        self
    }

    fn on_remove_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(mseg) = view.downcast_mut::<Mseg<P, R>>() {
                mseg.on_remove_point = Some(Box::new(callback));
            }
        }

        self
    }
}
