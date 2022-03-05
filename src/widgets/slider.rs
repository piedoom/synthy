use std::rc::Rc;

use super::{drag::ParamDragWidget, theme::Theme, FloatParamControl, Orientation, ParamControl};
use egui::*;
use nih_plug::prelude::*;

const SLIDER_SIZE: Vec2 = Vec2::new(100f32, 20f32);

pub struct Slider<'a, P: Param> {
    param: &'a P,
    setter: &'a ParamSetter<'a>,
    pub size: Vec2,
    pub track_width: f32,
    pub show_label: bool,
    pub show_value: bool,
    pub show_value_normalized: bool,
    pub theme: Option<Rc<Theme>>,
}

impl<'a, P> Widget for Slider<'a, P>
where
    P: Param,
{
    fn ui(self, ui: &mut Ui) -> Response {
        let theme = match self.theme.as_ref() {
            Some(theme) => *theme.clone(),
            None => Theme::default(),
        };

        let (response, paint) = ui.allocate_painter(self.size, Sense::click_and_drag());

        let rect = response.rect;

        // Paint background track
        paint.rect_filled(rect, 0f32, theme.colors.background_light);

        // Begin painting foreground track
        let mut track_rect = rect.shrink(2.0);

        // Determine orientation from size
        let orientation = match self.size.x > self.size.y {
            true => Orientation::Horizontal,
            false => Orientation::Vertical,
        };

        match orientation {
            Orientation::Horizontal => {
                track_rect.set_width(track_rect.width() * self.param().normalized_value())
            }
            Orientation::Vertical => track_rect.set_top(
                track_rect.bottom() - (track_rect.height() * self.param().normalized_value()),
            ),
        }

        paint.rect_filled(track_rect, 0f32, theme.colors.primary);

        ui.allocate_ui(Vec2::new(self.size.x, 0f32), |ui| {
            ui.vertical_centered(|ui| {
                if self.show_value {
                    ui.label(self.param().to_string());
                }
                if self.show_label {
                    ui.small(self.param().name());
                }
            });
        });

        self.respond_to_drags(ui, response, Some(orientation))
    }
}

impl<'a, P> Slider<'a, P>
where
    P: Param,
{
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }
    pub fn track_width(mut self, width: f32) -> Self {
        self.track_width = width;
        self
    }
    pub fn theme(mut self, theme: Rc<Theme>) -> Self {
        self.theme = Some(theme);
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

    /// Gets a vertical slider with the default size
    pub fn vertical(self) -> Self {
        self.size(Vec2::new(SLIDER_SIZE.y, SLIDER_SIZE.x))
    }
}

impl<'a, P> ParamControl<'a, P> for Slider<'a, P>
where
    P: Param,
{
    fn from_param(param: &'a P, setter: &'a ParamSetter<'a>) -> Self {
        Self {
            param,
            setter,
            size: SLIDER_SIZE,
            track_width: 8f32,
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

impl<'a, P> FloatParamControl<'a, P> for Slider<'a, P>
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

impl<'a, P> ParamDragWidget<'a, P> for Slider<'a, P> where P: Param {}
