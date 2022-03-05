use std::rc::Rc;

use super::{drag::ParamDragWidget, theme::Theme, FloatParamControl, ParamControl};
use egui::{epaint::PathShape, *};
use lyon_geom::{vector, Angle, Arc, Point};
use nih_plug::prelude::*;

pub struct Knob<'a, P: Param> {
    param: &'a P,
    setter: &'a ParamSetter<'a>,
    pub width: f32,
    pub track_width: f32,
    pub track_offset: f32,
    pub theme: Option<Rc<Theme>>,
    pub show_label: bool,
    pub show_value: bool,
    pub show_value_normalized: bool,
}

impl<'a, P> Knob<'a, P>
where
    P: Param,
{
    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
    pub fn track_width(mut self, width: f32) -> Self {
        self.track_width = width;
        self
    }
    pub fn theme(mut self, theme: Rc<Theme>) -> Self {
        self.theme = Some(theme.clone());
        self
    }
    pub fn show(mut self, label: bool, value: bool) -> Self {
        self.show_label = label;
        self.show_value = value;
        self
    }
    pub fn show_value_normalized(mut self, normalized: bool) -> Self {
        self.show_value_normalized = normalized;
        self
    }
}

impl<'a, P> Widget for Knob<'a, P>
where
    P: Param,
{
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = match self.theme.as_ref() {
            Some(theme) => *theme.clone(),
            None => Theme::default(),
        };

        let (response, paint) =
            ui.allocate_painter(egui::vec2(self.width, self.width), Sense::click_and_drag());
        let center = response.rect.center();
        let radii = (vector(self.width, self.width) * 0.5)
            - (vector(self.track_width, self.track_width) * 0.5);
        let center = Point::new(center.x, center.y);
        let offset = Angle::radians(self.track_offset);
        let start_angle = Angle::frac_pi_2() + offset;

        let track_arc: Vec<_> = Arc {
            center,
            radii,
            start_angle,
            sweep_angle: Angle::two_pi() - (offset * 2f32),
            x_rotation: Angle::zero(),
        }
        .flattened(0.01)
        .map(|p| egui::Pos2::new(p.x as f32, p.y as f32))
        .collect();

        let mut offset_angle =
            Angle::radians(std::f32::consts::TAU * self.param.normalized_value()) - (offset * 2f32);
        if offset_angle < Angle::zero() {
            offset_angle = Angle::zero()
        }
        let control_arc: Vec<_> = Arc {
            center,
            radii,
            start_angle,
            sweep_angle: offset_angle,
            x_rotation: Angle::radians(0.),
        }
        .flattened(0.01)
        .map(|p| egui::Pos2::new(p.x as f32, p.y as f32))
        .collect();

        paint.add(PathShape::line(
            track_arc,
            Stroke::new(self.track_width, theme.colors.background_light),
        ));
        paint.add(PathShape::line(
            control_arc,
            Stroke::new(self.track_width * 0.9, theme.colors.primary),
        ));

        ui.allocate_ui(Vec2::new(self.width, 0f32), |ui| {
            ui.vertical_centered(|ui| {
                if self.show_value {
                    ui.label(self.param.to_string());
                }
                if self.show_label {
                    ui.small(self.param.name());
                }
            });
        });

        self.respond_to_drags(ui, response, None)
    }
}

impl<'a, P> ParamControl<'a, P> for Knob<'a, P>
where
    P: Param,
{
    fn from_param(param: &'a P, setter: &'a ParamSetter<'a>) -> Self {
        Self {
            param,
            setter,
            width: 48f32,
            track_width: 8f32,
            track_offset: 35f32.to_radians(),
            theme: None,
            show_value: true,
            show_label: true,
            show_value_normalized: false,
        }
    }

    fn param(&self) -> &P {
        self.param
    }
    fn setter(&self) -> &'a ParamSetter {
        self.setter
    }
    fn show_label(&self) -> bool {
        self.show_label
    }
}

impl<'a, P> FloatParamControl<'a, P> for Knob<'a, P>
where
    P: Param<Plain = f32>,
{
    fn show_value(&self) -> bool {
        self.show_value
    }
    fn show_value_normalized(&self) -> bool {
        self.show_value_normalized
    }
}

impl<'a, P> ParamDragWidget<'a, P> for Knob<'a, P> where P: Param {}
