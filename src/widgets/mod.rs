use egui::Widget;
use nih_plug::prelude::{Param, ParamSetter};

pub(crate) mod drag;
pub mod envelope;
pub mod knob;
pub mod slider;
pub mod theme;

pub use {envelope::Envelope, knob::Knob, slider::Slider, theme::*};

pub trait ParamControl<'a, P: Param>: Widget {
    fn from_param(param: &'a P, setter: &'a ParamSetter<'a>) -> Self;
    fn param(&self) -> &P;
    fn setter(&self) -> &'a ParamSetter;
    fn show_label(&self) -> bool;
}

pub trait FloatParamControl<'a, P: Param<Plain = f32>>: ParamControl<'a, P> {
    fn show_value(&self) -> bool;
    fn show_value_normalized(&self) -> bool;
}
pub enum Orientation {
    Vertical,
    Horizontal,
}
