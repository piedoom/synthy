use crate::util::CurvePoints;
use femtovg::{Paint, Path};
use glam::Vec2;
use std::{cmp::Ordering, ops::RangeInclusive};
use vizia::*;

use super::{
    util::{data_to_ui_pos, ui_to_data_pos},
    PointCallback,
};

/// The distance in pixels before a node is considered hovered
const HOVER_RADIUS: f32 = 16f32;
/// The distance in seconds before two points cannot get closer
const MIN_RESOLUTION: f32 = 0.01f32;

/// The visuals of the graph
pub(crate) struct MsegGraph<P, R>
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
    on_changing_point: Option<PointCallback>,
    on_remove_point: Option<Box<dyn Fn(&mut Context, usize)>>,
    on_insert_point: Option<PointCallback>,
}

impl<P, R> MsegGraph<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    pub fn new(cx: &mut Context, points: P, range: R, max: f32) -> Handle<MsegGraph<P, R>> {
        Self {
            points,
            max,
            active_point_id: None,
            is_dragging_point: false,
            on_changing_point: None,
            range,
            on_remove_point: None,
            on_insert_point: None,
        }
        .build2(cx, |_cx| {})
    }
}

impl<P, R> View for MsegGraph<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn element(&self) -> Option<String> {
        Some("mseg-graph".to_string())
    }

    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        let points = self.points.get(cx).clone();
        let ui_points = points.iter().map(|point| {
            data_to_ui_pos(
                cx,
                Vec2::new(point.x, point.y),
                self.range.clone(),
                self.max,
            )
        });
        // Window events to move points
        if let Some(ev) = event.message.downcast::<WindowEvent>() {
            match ev {
                WindowEvent::MouseDown(button) => {
                    match button {
                        MouseButton::Left => {
                            if self.active_point_id.is_some() {
                                cx.capture();
                                self.is_dragging_point = true;
                            } else {
                                // TODO: create a new point
                            }
                        }
                        MouseButton::Right => {
                            // Delete a currently active point
                            if let Some(index) = self.active_point_id {
                                cx.release();
                                self.is_dragging_point = false;
                                if let Some(callback) = self.on_remove_point.take() {
                                    (callback)(cx, index);
                                    self.on_remove_point = Some(callback);
                                }
                            }
                        }
                        _ => (),
                    }
                }
                WindowEvent::MouseUp(button) => {
                    if *button == MouseButton::Left {
                        cx.release();
                        self.is_dragging_point = false;
                    }
                }
                WindowEvent::MouseMove(x, y) => {
                    let current_pos = Vec2::new(*x, *y);
                    if self.is_dragging_point {
                        // Up to the user to drag the current point around
                        if let Some(callback) = self.on_changing_point.take() {
                            let active_id = self.active_point_id.unwrap();
                            let mut new_v = if active_id != 0 {
                                ui_to_data_pos(cx, &current_pos, self.range.clone(), self.max)
                            } else {
                                Vec2::ZERO
                            };
                            if active_id == points.len() - 1 {
                                new_v.y = 0f32;
                            }

                            // Clamp the point (and check for left and right bounds)
                            let right_bound =
                                points.get(active_id + 1).map(|p| p.x).unwrap_or(self.max)
                                    - MIN_RESOLUTION;
                            let left_bound = points.get(active_id - 1).map(|p| p.x).unwrap_or(0f32)
                                + MIN_RESOLUTION;
                            let new_v = new_v
                                .clamp(Vec2::new(left_bound, 0f32), Vec2::new(right_bound, 1f32));

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
        let line_color: Color = cx
            .style
            .border_color
            .get(cx.current)
            .cloned()
            .unwrap_or_default();
        let active_color: Color = cx
            .style
            .font_color
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
                    data_to_ui_pos(
                        cx,
                        Vec2::new(point.1.x, point.1.y),
                        self.range.clone(),
                        self.max,
                    ),
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
            Paint::color(line_color.into()).with_line_width(2f32),
        );

        // Draw points
        for (i, point) in &ui_points {
            // Main node
            let mut path = Path::new();
            path.circle(point.x, point.y, 4.0);
            canvas.fill_path(&mut path, Paint::color(line_color.into()));
            // check for hover
            if self.active_point_id.map(|x| &x == i).unwrap_or_default() {
                let mut path = Path::new();
                path.circle(point.x, point.y, 8.0);
                canvas.stroke_path(
                    &mut path,
                    Paint::color(active_color.into()).with_line_width(2f32),
                );
            }
        }
    }
}

pub trait MsegGraphHandle<P, R>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
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

impl<'a, P, R> MsegGraphHandle<P, R> for Handle<'a, MsegGraph<P, R>>
where
    P: Lens<Target = CurvePoints>,
    R: Lens<Target = RangeInclusive<f32>>,
{
    fn on_changing_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(mseg_graph) = view.downcast_mut::<MsegGraph<P, R>>() {
                mseg_graph.on_changing_point = Some(Box::new(callback));
            }
        }
        self
    }
    fn on_insert_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(mseg_graph) = view.downcast_mut::<MsegGraph<P, R>>() {
                mseg_graph.on_insert_point = Some(Box::new(callback));
            }
        }

        self
    }
    fn on_remove_point<F>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize),
    {
        if let Some(view) = self.cx.views.get_mut(&self.entity) {
            if let Some(mseg_graph) = view.downcast_mut::<MsegGraph<P, R>>() {
                mseg_graph.on_remove_point = Some(Box::new(callback));
            }
        }
        self
    }
}
