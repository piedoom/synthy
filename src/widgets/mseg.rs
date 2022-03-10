//! Multi-stage envelope generator widget

use std::{cmp::Ordering, ops::RangeInclusive};

use super::zoomer::{Zoomer, ZoomerHandle};
use crate::util::{CurvePoint, CurvePoints};
use femtovg::{Paint, Path};
use glam::Vec2;
use vizia::*;

/// The distance in pixels before a node is considered hovered
const HOVER_RADIUS: f32 = 16f32;

pub struct Mseg<L>
where
    L: Lens<Target = CurvePoints>,
{
    points: L,
    on_changing_point: Option<Box<dyn Fn(&mut Context, usize, Vec2)>>,
}

#[derive(Clone, Debug, Data)]
pub struct MsegRangeInternal {
    pub range: RangeInclusive<f32>,
}

impl Lens for MsegRangeInternal {
    type Source = Self;

    type Target = RangeInclusive<f32>;

    fn view<O, F: FnOnce(Option<&Self::Target>) -> O>(&self, source: &Self::Source, map: F) -> O {
        map(Some(&source.range))
    }
}

impl Default for MsegRangeInternal {
    fn default() -> Self {
        Self {
            /// The current view section, describing the first and last view
            /// points as normalized values between `0..=1`.
            range: 0f32..=1f32,
        }
    }
}

impl Model for MsegRangeInternal {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast::<MsegRangeEventInternal>() {
            // set min / max so that it is never out of range or invalid
            self.range = match ev {
                MsegRangeEventInternal::SetViewRangeStart(x) => {
                    let x = x.max(0f32).min(*self.range.end());
                    x..=*self.range.end()
                }
                MsegRangeEventInternal::SetViewRangeEnd(x) => {
                    let x = x.max(*self.range.start()).min(1f32);
                    *self.range.start()..=x
                }
            };
        }
    }
}

pub enum MsegRangeEventInternal {
    SetViewRangeStart(f32),
    SetViewRangeEnd(f32),
}

pub enum MsegEvent {
    OnChangingPoint { index: usize, position: Vec2 },
}

impl<L: Lens<Target = CurvePoints>> Mseg<L> {
    pub fn new(cx: &mut Context, lens: L) -> Handle<Mseg<L>> {
        Self {
            points: lens.clone(),
            on_changing_point: None,
        }
        .build2(cx, move |cx| {
            let background_color: Color = cx
                .style
                .background_color
                .get(cx.current)
                .cloned()
                .unwrap_or_default();

            if cx.data::<MsegRangeInternal>().is_none() {
                MsegRangeInternal::default().build(cx);
            }

            MsegGraph::new(cx, lens)
                .background_color(background_color)
                .on_changing::<_, L>(|cx, i, v| {
                    cx.emit(MsegEvent::OnChangingPoint {
                        index: i,
                        position: v,
                    });
                });
            Zoomer::new(cx, cx.data::<MsegRangeInternal>().cloned().unwrap())
                .on_changing_range_start::<_, MsegRangeInternal>(|cx, val| {
                    cx.emit(MsegRangeEventInternal::SetViewRangeStart(val))
                })
                .on_changing_range_end::<_, MsegRangeInternal>(|cx, val| {
                    cx.emit(MsegRangeEventInternal::SetViewRangeEnd(val))
                });
        })
    }
}

impl<L: Lens<Target = CurvePoints>> Model for Mseg<L> {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        if let Some(ev) = event.message.downcast::<MsegEvent>() {
            match ev {
                MsegEvent::OnChangingPoint { index, position } => {
                    // use callback
                    if let Some(callback) = self.on_changing_point.take() {
                        (callback)(cx, *index, *position);
                        self.on_changing_point = Some(callback);
                    }
                }
            }
        }
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
    /// The temporary value of the currently hovered or active point
    active_point_id: Option<usize>,
    is_dragging_point: bool,
    on_changing_point: Option<Box<dyn Fn(&mut Context, usize, Vec2)>>,
}

impl<L> MsegGraph<L>
where
    L: Lens<Target = CurvePoints>,
{
    pub fn new(cx: &mut Context, lens: L) -> Handle<MsegGraph<L>> {
        Self {
            points: lens,
            max: 8f32,
            active_point_id: None,
            is_dragging_point: false,
            on_changing_point: None,
        }
        .build2(cx, |cx| {})
    }
}

impl<'a, L: Lens<Target = CurvePoints>> View for MsegGraph<L> {
    fn event(&mut self, cx: &mut Context, event: &mut vizia::Event) {
        let points = self.points.get(cx).clone();
        let ui_points = points
            .iter()
            .map(|point| data_to_ui_pos(cx, point, self.max, cx.current));
        // Window events to move points
        if let Some(ev) = event.message.downcast::<WindowEvent>() {
            match ev {
                WindowEvent::MouseDown(button) => {
                    if self.active_point_id.is_some() {
                        self.is_dragging_point = true;
                    }
                }
                WindowEvent::MouseUp(button) => {
                    self.is_dragging_point = false;
                }
                WindowEvent::MouseMove(x, y) => {
                    let current_pos = Vec2::new(*x, *y);
                    if self.is_dragging_point {
                        // Up to the user to drag the current point around
                        if let Some(callback) = self.on_changing_point.take() {
                            let active_id = self.active_point_id.unwrap();
                            let new_v = ui_to_data_pos(cx, &current_pos, self.max, cx.current);
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
        let ui_points = points
            .iter()
            .map(|point| data_to_ui_pos(cx, point, self.max, cx.current));

        // Draw background rect
        let mut path = Path::new();
        path.rect(0f32, 0f32, width, height);
        canvas.fill_path(&mut path, Paint::color(background_color.into()));

        // Draw points
        let mut lines = Path::new();

        for (i, point) in ui_points.enumerate() {
            if i == 0 {
                lines.move_to(point.x, point.y);
            }
            // Main node
            let mut path = Path::new();
            path.circle(point.x, point.y, 4.0);

            // check for hover
            let mut color = Color::white();
            if self.active_point_id.map(|x| x == i).unwrap_or_default() {
                color = Color::rgb(255, 0, 0);
            }

            canvas.fill_path(&mut path, Paint::color(color.into()));

            // Lines
            lines.line_to(point.x, point.y);
        }
        canvas.stroke_path(
            &mut lines,
            Paint::color(Color::white().into()).with_line_width(2f32),
        );
    }
}

/// Convert a screen value to its data position
pub fn ui_to_data_pos(cx: &Context, ui_point: &Vec2, max_data: f32, container: Entity) -> Vec2 {
    let data = cx.data::<MsegRangeInternal>().unwrap();
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
    let offset_data = data.range.start() * max_data;
    let x = ((ui_point.x / width) * (data.range.end() - data.range.start())) + offset_data;
    Vec2::new(x, y)
}
pub fn data_to_ui_pos(cx: &Context, point: &CurvePoint, max: f32, container: Entity) -> Vec2 {
    let data = cx.data::<MsegRangeInternal>().unwrap();
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
    let offset = data.range.start() * max;
    // Calculate the x-zoom scale to apply to points
    let scale = 1f32 / ((data.range.end() - data.range.start()) * max);
    let x = ((point.x - offset) * scale) * width;
    let relative = Vec2::new(x, y);
    // adjust to be absolute by adding the container coords
    let bounds = {
        let b = cx.cache.get_bounds(container);
        Vec2::new(b.x, b.y)
    };
    relative + bounds
}

pub trait MsegGraphHandle<'a> {
    type View: View;
    fn on_changing<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
        L: Lens<Target = CurvePoints>;
}
impl<'a, V: View> MsegGraphHandle<'a> for Handle<'a, V> {
    type View = V;

    fn on_changing<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
        L: Lens<Target = CurvePoints>,
    {
        if let Some(mseg) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|f| f.downcast_mut::<MsegGraph<L>>())
        {
            mseg.on_changing_point = Some(Box::new(callback));
        }

        self
    }
}

pub trait MsegHandle<'a> {
    type View: View;
    fn on_changing_point<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
        L: Lens<Target = CurvePoints>;
}
impl<'a, V: View> MsegHandle<'a> for Handle<'a, V> {
    type View = V;

    fn on_changing_point<F, L>(self, callback: F) -> Self
    where
        F: 'static + Fn(&mut Context, usize, Vec2),
        L: Lens<Target = CurvePoints>,
    {
        if let Some(mseg) = self
            .cx
            .views
            .get_mut(&self.entity)
            .and_then(|f| f.downcast_mut::<Mseg<L>>())
        {
            mseg.on_changing_point = Some(Box::new(callback));
        }

        self
    }
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
