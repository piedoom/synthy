use std::{
    ops::{AddAssign, RangeInclusive},
    rc::Rc,
    sync::RwLock,
};

use super::theme::Theme;
use egui::*;
use nih_plug::prelude::*;

const HINT_SIZE: f32 = 8f32;
const BUMP_AMOUNT: f32 = 0.1f32;
const SCROLL_ZOOM_MULTIPLIER: f32 = 0.1f32;
const INITIAL_ZOOM: f32 = 0.2f32;

fundsp::lazy_static::lazy_static! {
    static ref CURRENT_ACTIVE_ID_MEMORY_ID: egui::Id = egui::Id::new((file!(), 0));
}

pub struct Envelope<'a> {
    param: &'a RwLock<Vec<(f32, f32)>>,
    pub size: Vec2,
    pub node_size: f32,
    pub stroke_width: f32,
    pub theme: Option<Rc<Theme>>,
    // Initial zoom
    pub initial_zoom: f32,
    pub zoom_range: RangeInclusive<f32>,
    /// A unique identifier used for UI purposes
    pub name: &'a str,
    id: egui::Id,
}

impl<'a> Widget for Envelope<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let theme = match self.theme.as_ref() {
                Some(theme) => *theme.clone(),
                None => Theme::default(),
            };
            let zoom = ui
                .memory()
                .data
                .get_temp::<f32>(self.id)
                .unwrap_or(self.initial_zoom);
            let current_node_id: Option<usize> =
                ui.memory().data.get_temp(*CURRENT_ACTIVE_ID_MEMORY_ID);
            let paint_node = |pos, painter: &Painter, color| {
                let r = Rect::from_center_size(pos, Vec2::splat(self.node_size));
                painter.rect_filled(r, 0f32, color);
            };

            // Convert param point coordinates to absolute UI coordinates for use in egui
            let to_screen_point = |(x, y): &(f32, f32), rect: Rect| -> Pos2 {
                let x = ((x * zoom) * rect.width()) + rect.left();
                let y = (-y * rect.height()) + rect.bottom();
                Pos2::new(x, y)
            };

            // Convert absolute egui coordinates into param point coordinates
            let from_screen_point = |pos: Pos2, rect: Rect| {
                let relative = pos - rect.left_top();
                let x = (relative.x / zoom) / rect.width();
                let y = (-relative.y / rect.height()) + 1f32;
                (x, y)
            };

            let (response, paint) =
                ui.allocate_painter(self.size - Vec2::new(0f32, 16f32), Sense::click_and_drag());

            let rect = response.rect;

            // Get the on-screen coordinates of every point
            let points: Vec<Pos2> = if let Ok(param) = self.param.read() {
                param.iter().map(|pos| to_screen_point(pos, rect)).collect()
            } else {
                Vec::default()
            };

            let hovered_point: Option<(usize, Pos2)> =
                if let Some(pos) = ui.input().pointer.interact_pos() {
                    let mut closest: Vec<(usize, Pos2)> = points
                        .iter()
                        .enumerate()
                        .filter(|(_, p)| pos.distance_sq(**p) <= f32::powi(HINT_SIZE, 2))
                        .map(|x| (x.0, *x.1))
                        .collect::<Vec<(usize, Pos2)>>();
                    closest.sort_by(|(_, a), (_, b)| {
                        pos.distance_sq(*a)
                            .partial_cmp(&pos.distance_sq(*b))
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                    closest.first().cloned()
                } else {
                    None
                };

            // Paint background
            paint.rect_filled(rect, 0f32, theme.colors.background_light);

            // TODO: Paint tickmarks

            // Paint crosshairs
            if let Some(pos) = response.hover_pos() {
                let stroke = Stroke::new(1f32, theme.colors.border);
                paint.line_segment(
                    [
                        Pos2::new(rect.left(), pos.y),
                        Pos2::new(rect.right(), pos.y),
                    ],
                    stroke,
                );
                paint.line_segment(
                    [
                        Pos2::new(pos.x, rect.top()),
                        Pos2::new(pos.x, rect.bottom()),
                    ],
                    stroke,
                )
            }

            // Loop through points
            let mut last_point = rect.left_bottom();
            for point in &points {
                paint.line_segment(
                    [last_point, *point],
                    Stroke::new(2f32, theme.colors.primary),
                );
                last_point = *point;
            }

            for (i, point) in points.iter().enumerate() {
                let hovered = current_node_id
                    .map(|x| i == x)
                    .unwrap_or_else(|| hovered_point.map(|x| i == x.0).unwrap_or_default());

                let color = match hovered {
                    true => Color32::RED,
                    false => theme.colors.primary,
                };

                if response.drag_started() && hovered {
                    ui.memory()
                        .data
                        .insert_temp(*CURRENT_ACTIVE_ID_MEMORY_ID, i);
                }
                paint_node(*point, &paint, color);
            }

            // Perform a drag on the node
            if let Some(saved_id) = current_node_id {
                // First point always has coordinates of 0,0
                if saved_id != 0 {
                    // First, we'll need the coordinates of the previous and next node so we can ensure we do not generate an invalid envelope
                    let (prev, next) = if let Ok(param) = self.param.read() {
                        let prev = param.get(saved_id - 1).cloned();
                        let next = param.get(saved_id + 1).cloned();
                        (prev, next)
                    } else {
                        (None, None)
                    };

                    if let Ok(mut param) = self.param.try_write() {
                        if let Some((x, y)) = param.get_mut(saved_id) {
                            let dt = response.drag_delta() * Vec2::new(1.0 / zoom, -1.0);
                            *x += dt.x / rect.width();
                            *y += dt.y / rect.height();

                            // if dragging past the x of a previous or next node... don't!
                            if let Some(prev) = prev {
                                if *x <= (prev.0 + BUMP_AMOUNT) {
                                    *x = prev.0 + BUMP_AMOUNT;
                                }
                            }
                            if let Some(next) = next {
                                if *x >= (next.0 - BUMP_AMOUNT) {
                                    *x = next.0 - BUMP_AMOUNT;
                                }
                            }

                            // If the last node, ensure Y is 0
                            if saved_id == points.len() - 1 {
                                *y = 0f32;
                            }

                            *y = y.clamp(0f32, 1f32);
                        }
                    }
                }
            } else if hovered_point.is_none() {
                // Hover style
                if let Some(pos) = response.hover_pos() {
                    // TODO: snap close to the line
                    paint_node(pos, &paint, theme.colors.border);
                }

                // Click to add a point
                if let Some(pos) = response.interact_pointer_pos() {
                    if response.drag_started() && !ui.input().pointer.secondary_down() {
                        // add node when clicked
                        let click_x = pos.x;

                        // get the point we are in-between
                        let left_egui_point: Option<(usize, f32)> =
                            points.iter().enumerate().find_map(|(i, pos)| {
                                if pos.x > click_x {
                                    Some((i, pos.x))
                                } else {
                                    None
                                }
                            });
                        let right_egui_point = points.iter().find_map(|pos| {
                            if pos.x < click_x {
                                Some(pos.x)
                            } else {
                                None
                            }
                        });

                        // Added node must be in-between others
                        if let (Some(left_point), Some(_right_point)) =
                            (left_egui_point, right_egui_point)
                        {
                            if let Ok(mut param) = self.param.try_write() {
                                param.insert(left_point.0, from_screen_point(pos, response.rect));
                                ui.memory().data.insert_temp(self.id, left_point.0);
                            }
                        }
                    }
                }
            }

            if response.drag_released() {
                ui.memory()
                    .data
                    .remove::<usize>(*CURRENT_ACTIVE_ID_MEMORY_ID);
            }

            // Respond to removing nodes
            if response.secondary_clicked() {
                if let Some(current_node_id) = current_node_id {
                    if current_node_id != 0 && current_node_id != points.len() - 1 {
                        if let Ok(mut param) = self.param.try_write() {
                            param.remove(current_node_id);
                        }
                    }
                }
            }

            // Respond to zooming
            if response.hovered() {
                // Scroll zooming
                let zoom_dt = (ui.input().zoom_delta() - 1f32) * SCROLL_ZOOM_MULTIPLIER;
                if zoom_dt != 0f32 {
                    let new_value =
                        (zoom + zoom_dt).clamp(*self.zoom_range.start(), *self.zoom_range.end());
                    ui.memory().data.insert_temp(self.id, new_value);
                }
            }

            // Zoom bar interface
            let (z_resp, z_paint) =
                ui.allocate_painter(egui::Vec2::new(self.size.x, 16f32), Sense::click_and_drag());

            // zoom bar bg
            z_paint.rect_filled(z_resp.rect, 0f32, theme.colors.background_light);

            // zoom bar fg
            let normalized_zoom = (zoom + self.zoom_range.start())
                / (self.zoom_range.end() + self.zoom_range.start());
            let mut bar_rect = z_resp.rect.shrink(z_resp.rect.height() * 0.1);

            // click to zoom
            if let Some(click_pos) = z_resp.interact_pointer_pos() {
                let ratio = 1f32 - (click_pos.x - bar_rect.left()) / bar_rect.width();
                ui.memory().data.insert_temp(self.id, ratio);
            }

            bar_rect.set_width(bar_rect.width() * (1f32 - normalized_zoom));
            z_paint.rect_filled(bar_rect, 0f32, theme.colors.primary);

            response
        })
        .inner
    }
}

impl<'a> Envelope<'a> {
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn theme(mut self, theme: Rc<Theme>) -> Self {
        self.theme = Some(theme);
        self
    }
    pub fn zoom(mut self, zoom: f32) -> Self {
        self.initial_zoom = zoom;
        self
    }
}

impl<'a> Envelope<'a> {
    pub fn from_param(param: &'a RwLock<Vec<(f32, f32)>>, name: &'a str) -> Self {
        Self {
            param,
            size: Vec2::new(100f32, 60f32),
            theme: None,
            initial_zoom: INITIAL_ZOOM,
            node_size: 6f32,
            stroke_width: 2f32,
            name,
            id: egui::Id::new(name),
            zoom_range: 0.05..=1f32,
        }
    }
}
