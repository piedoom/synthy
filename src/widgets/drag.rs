use egui::*;
use nih_plug::param::*;

use super::{Orientation, ParamControl};

const GRANULAR_DRAG_MULTIPLIER: f32 = 0.0015;

fundsp::lazy_static::lazy_static! {
    static ref DRAG_NORMALIZED_START_VALUE_MEMORY_ID: egui::Id = egui::Id::new((file!(), 0));
    static ref DRAG_AMOUNT_MEMORY_ID: egui::Id = egui::Id::new((file!(), 1));
}

pub trait ParamDragWidget<'a, P>: ParamControl<'a, P>
where
    P: Param,
{
    fn begin_drag(&'a self) {
        self.setter().begin_set_parameter(self.param());
    }
    fn normalized_value(&self) -> f32 {
        self.param().normalized_value()
    }

    fn set_normalized_value(&'a self, normalized: f32) {
        if normalized != self.param().normalized_value() {
            // This snaps to the nearest plain value if the parameter is stepped in some wayA
            let value = self.param().preview_plain(normalized);
            self.setter().set_parameter(self.param(), value);
        }
    }

    fn reset_param(&'a self) {
        let normalized_default = self.setter().default_normalized_param_value(self.param());
        self.setter()
            .set_parameter_normalized(self.param(), normalized_default);
    }

    fn end_drag(&'a self) {
        self.setter().end_set_parameter(self.param());
    }

    fn get_drag_normalized_start_value_memory(ui: &Ui) -> f32 {
        ui.memory()
            .data
            .get_temp(*DRAG_NORMALIZED_START_VALUE_MEMORY_ID)
            .unwrap_or(0.5)
    }

    fn set_drag_normalized_start_value_memory(ui: &Ui, amount: f32) {
        ui.memory()
            .data
            .insert_temp(*DRAG_NORMALIZED_START_VALUE_MEMORY_ID, amount);
    }

    fn get_drag_amount_memory(ui: &Ui) -> Vec2 {
        ui.memory()
            .data
            .get_temp(*DRAG_AMOUNT_MEMORY_ID)
            .unwrap_or(Vec2::ZERO)
    }

    fn set_drag_amount_memory(ui: &Ui, amount: Vec2) {
        ui.memory().data.insert_temp(*DRAG_AMOUNT_MEMORY_ID, amount);
    }

    fn granular_drag(
        &'a self,
        ui: &Ui,
        drag_delta: Vec2,
        speed: Option<f32>,
        orientation: Option<Orientation>,
    ) {
        // Remember the intial position when we started with the granular drag. This value gets
        // reset whenever we have a normal itneraction with the slider.
        let start_value = if Self::get_drag_amount_memory(ui) == Vec2::ZERO {
            Self::set_drag_normalized_start_value_memory(ui, self.normalized_value());
            self.normalized_value()
        } else {
            Self::get_drag_normalized_start_value_memory(ui)
        };

        let total_drag_distance = drag_delta + Self::get_drag_amount_memory(ui);
        Self::set_drag_amount_memory(ui, total_drag_distance);

        let mut x = total_drag_distance.x;
        let mut y = total_drag_distance.y;

        if let Some(orientation) = orientation {
            match orientation {
                Orientation::Vertical => x = 0f32,
                Orientation::Horizontal => y = 0f32,
            }
        }

        let delta = (x + -y) * GRANULAR_DRAG_MULTIPLIER * speed.unwrap_or(1.0);
        self.set_normalized_value((start_value + delta).clamp(0.0, 1.0));
    }

    fn respond_to_drags(
        &'a self,
        ui: &mut Ui,
        response: Response,
        orientation: Option<Orientation>,
    ) -> egui::Response {
        if response.drag_started() {
            // When beginning a drag or dragging normally, reset the memory used to keep track of
            // our granular drag
            self.begin_drag();
            Self::set_drag_amount_memory(ui, Vec2::ZERO)
        }

        if response.interact_pointer_pos().is_some() {
            if ui.input().modifiers.command {
                // Like double clicking, Ctrl+Click should reset the parameter
                self.reset_param();
            }

            let speed = if ui.input().modifiers.shift {
                // And shift dragging should switch to a more granulra input method
                1.0
            } else {
                5.0
            };

            self.granular_drag(ui, response.drag_delta(), Some(speed), orientation);

            if response.double_clicked() {
                self.reset_param();
            }
            if response.drag_released() {
                self.end_drag();
            }
        }

        response
    }
}
