//! Multi-stage envelope generator widget

use std::{cmp::Ordering, ops::RangeInclusive};

use super::zoomer::{Zoomer, ZoomerHandle};
use crate::util::{CurvePoint, CurvePoints};
use femtovg::{Paint, Path};
use glam::Vec2;
use vizia::*;

/// The distance in pixels before a node is considered hovered
const HOVER_RADIUS: f32 = 16f32;

enum MsegInternalEvent {
    OnChangingRangeStart(f32),
    OnChangingRangeEnd(f32),
}


pub struct Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    points: P,
    range: R,
    on_changing_point: Option<Box<dyn Fn(&mut Context, usize, Vec2)>>,
    on_changing_range_start: Option<Box<dyn Fn(&mut Context, f32)>>,
    on_changing_range_end: Option<Box<dyn Fn(&mut Context, f32)>>,
}

impl<P, R> Mseg<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(
        cx: &mut Context,
        points: P,
        range: R,
    ) -> Handle<Mseg<P, R>> {
        Self {
            points: points.clone(),
            range: range.clone(),
            on_changing_point: None,
            on_changing_range_start: None,
            on_changing_range_end: None,
        }
        .build2(cx, |cx| {
            let background_color: Color = cx
                .style
                .background_color
                .get(cx.current)
                .cloned()
                .unwrap_or_default();

            MsegGraph::new(cx, points, range.clone()).background_color(background_color);
            Zoomer::new(
                cx,
                range.clone(),
            )
            .on_changing_start(|cx, x| 
                cx.emit(MsegInternalEvent::OnChangingRangeStart(x)))
            .on_changing_end(|cx, x|
                cx.emit(MsegInternalEvent::OnChangingRangeEnd(x)));
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
                },
                MsegInternalEvent::OnChangingRangeEnd(x) => {
                    if let Some(callback) = self.on_changing_range_end.take() {
                        (callback)(cx, *x);
                        self.on_changing_range_end = Some(callback);
                    }
                },
            }
        }
    }
}

pub trait MsegHandle<P, R> where P: Lens<Target = CurvePoints>,
R: Lens<Target = RangeInclusive<f32>> {
    fn on_changing_range_start<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32);
    fn on_changing_range_end<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, f32);
    fn on_changing_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2);
}

impl<'a, P, R> MsegHandle<P, R> for Handle<'a, Mseg<P,R>>
where
P: Lens<Target = CurvePoints>,
R: Lens<Target = RangeInclusive<f32>>,
{
    fn on_changing_point<F>(self, callback: F) -> Self 
        where F: 'static + Fn(&mut Context, usize, Vec2) {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(zoomer) = view.downcast_mut::<Mseg<P, R>>() {
                zoomer.on_changing_point = Some(Box::new(callback));
            }
        }

        self
    }

    fn on_changing_range_start<F>(self, callback: F) -> Self 
        where F: 'static + Fn(&mut Context, f32) {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(zoomer) = view.downcast_mut::<Mseg<P, R>>() {
                zoomer.on_changing_range_start = Some(Box::new(callback));
            }
        }

        self
    }

    fn on_changing_range_end<F>(self, callback: F) -> Self 
        where F: 'static + Fn(&mut Context, f32) {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(zoomer) = view.downcast_mut::<Mseg<P, R>>() {
                zoomer.on_changing_range_end = Some(Box::new(callback));
            }
        }

        self
    }
}

/// The visuals of the graph
struct MsegGraph<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    points: P,
    range: R,
    /// The max length of this MSEG, usually representing `f32` seconds. (The
    /// minimum is always `0`).
    max: f32,
    /// The temporary value of the currently hovered or active point
    active_point_id: Option<usize>,
    is_dragging_point: bool,
    on_changing_point: Option<Box<dyn Fn(&mut Context, usize, Vec2)>>,
    
}

impl<P, R> MsegGraph<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(cx: &mut Context, points: P, range: R) -> Handle<MsegGraph<P, R>> {
        Self {
            points,
            max: 8f32,
            active_point_id: None,
            is_dragging_point: false,
            on_changing_point: None,
            range,
            
        }
        .build2(cx, |cx| {})
    }
}

impl<P, R> View for MsegGraph<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        let points = self.points.get(cx).clone();
        let ui_points = points
            .iter()
            .map(|point| data_to_ui_pos(cx, point, self.range.clone(), self.max, cx.current));
        // Window events to move points
        if let Some(ev) = event.message.downcast::<WindowEvent>() {
            match ev {
                WindowEvent::MouseDown(button) => if *button == MouseButton::Left {
                    if self.active_point_id.is_some() {
                        self.is_dragging_point = true;
                    }
                }
                WindowEvent::MouseUp(button) => if *button == MouseButton::Left {
                    self.is_dragging_point = false;
                }
                WindowEvent::MouseMove(x, y) => {
                    let current_pos = Vec2::new(*x, *y);
                    if self.is_dragging_point {
                        // Up to the user to drag the current point around
                        if let Some(callback) = self.on_changing_point.take() {
                            let active_id = self.active_point_id.unwrap();
                            let new_v = ui_to_data_pos(
                                cx,
                                &current_pos,
                                self.range.clone(),
                                self.max,
                                cx.current,
                            );
                            (callback)(cx, active_id, new_v);
                            self.on_changing_point = Some(callback);
                        } // asdasdasdasd
                    } else {
                        // determine if we are hovering within the range of a
                        //point if we are not currently dragging points
                        let mut filtered_points: Vec<(usize, Vec2)> = ui_points
                            .enumerate()
                            .filter_map(|(i, point)| {
                                if point.distance_squared(current_pos) <= HOVER_RADIUS.powi(2) {
                                    Some((i, point))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        filtered_points.sort_by(|a, b| {
                            a.1.distance_squared(current_pos)
                                .partial_cmp(&b.1.distance_squared(current_pos))
                                .unwrap_or(Ordering::Equal)
                        });
                        // Store our point ID
                        match filtered_points.first() {
                            Some((closest_point_id, ..)) => {
                                self.active_point_id = Some(*closest_point_id);
                            }
                            _ => self.active_point_id = None,
                        }
                    }
                }
                // WindowEvent::MouseOut => todo!(),
                _ => (),
            }
        }
    }
    fn draw(&self, cx: &mut Context, canvas: &mut Canvas) {
        let (width, height) = (
            cx.cache.get_width(cx.current),
            cx.cache.get_height(cx.current),
        );
        let background_color: Color = cx
            .style
            .background_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default();

        // points
        let points = &**self.points.get(cx);
        let ui_points: Vec<(_, _)> = points
            .iter()
            .enumerate()
            .map(|point| {
                (
                    point.0,
                    data_to_ui_pos(cx, point.1, self.range.clone(), self.max, cx.current),
                )
            })
            .collect();

        // Draw background rect
        let mut path = Path::new();
        path.rect(0f32, 0f32, width, height);
        canvas.fill_path(&mut path, Paint::color(background_color.into()));

        // Draw lines
        let mut lines = Path::new();
        for (i, point) in &ui_points {
            if i == &0 {
                lines.move_to(point.x, point.y);
            }
            // Lines
            lines.line_to(point.x, point.y);
        }
        canvas.stroke_path(
            &mut lines,
            Paint::color(Color::white().into()).with_line_width(2f32),
        );

        // Draw points
        for (i, point) in &ui_points {
            // Main node
            let mut path = Path::new();
            path.circle(point.x, point.y, 4.0);

            // check for hover
            let mut color = Color::white();
            if self.active_point_id.map(|x| &x == i).unwrap_or_default() {
                color = Color::rgb(255, 0, 0);
            }

            canvas.fill_path(&mut path, Paint::color(color.into()));
        }
    }
}

/// Convert a screen value to its data position
pub fn ui_to_data_pos(
    cx: &Context,
    ui_point: &Vec2,
    range_data: impl Lens<Target = RangeInclusive<f32>>,
    max_data: f32,
    container: Entity,
) -> Vec2 {
    let (width, height) = (
        cx.cache.get_width(container),
        cx.cache.get_height(container),
    );
    // Assume `ui_point` is an absolute coordinate. We must convert it to relative coordinates
    let mut ui_point = *ui_point;
    let bounds = {
        let b = cx.cache.get_bounds(cx.current);
        Vec2::new(b.x, b.y)
    };
    // Convert to relative point
    ui_point -= bounds;
    // Scale points to fit within `(x,y) = ([0..=max], [0..=1])`
    // This assumes mouse coordinates are relative and not absolute. Which is possibly not true!
    let y = (height - ui_point.y) / height;
    let range = range_data.get(cx);
    let offset_data = range.start() * max_data;
    let x = ((ui_point.x / width) * (range.end() - range.start())) + offset_data;
    Vec2::new(x, y)
}
pub fn data_to_ui_pos(
    cx: &Context,
    point: &CurvePoint,
    range_data: impl Lens<Target = RangeInclusive<f32>>,
    max: f32,
    container: Entity,
) -> Vec2 {
    let (width, height) = (
        cx.cache.get_width(container),
        cx.cache.get_height(container),
    );
    // y value is a simple scale
    let y = height - (point.y * height);
    // x value requires us to calculate our zoomed position TODO: Zoom too

    // Calculate the x-offset determined by the current view zoom window
    // This value shifts points to the left and right We calculate the
    // offset by getting the view's starting x value, which is normalized.
    // We then see how much time that offsets by multiplying that normalized
    // value times the maximum X of our MSEG.
    let range = range_data.get(cx);
    let offset = range.start() * max;
    // Calculate the x-zoom scale to apply to points
    let scale = 1f32 / ((range.end() - range.start()) * max);
    let x = ((point.x - offset) * scale) * width;
    let relative = Vec2::new(x, y);
    // adjust to be absolute by adding the container coords
    let bounds = {
        let b = cx.cache.get_bounds(container);
        Vec2::new(b.x, b.y)
    };
    relative + bounds
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gets_ui_point_from_data() {
        //  ui_to_data_pos(cx, ui_point, max_data, container)
    }

    #[test]
    fn gets_data_point_from_ui() {}
}
