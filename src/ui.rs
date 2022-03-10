use crate::{
    util::CurvePoints,
    widgets::{mseg::{Mseg, MsegHandle}, zoomer::Zoomer},
    SynthyFloatParam, SynthyParams,
};
use fundsp::math::xerp;
use glam::Vec2;
use nih_plug::prelude::*;
use std::{ops::RangeInclusive, pin::Pin, sync::Arc};
use vizia::*;

use self::app_data_derived_lenses::params as ViziaParams;

const STYLE: &str = include_str!("style.css");

#[derive(Lens)]
pub struct AppData {
    pub params: Pin<Arc<SynthyParams>>,
    #[lens(ignore)]
    pub context: Arc<dyn GuiContext>,
    env_zoom_view: RangeInclusive<f32>,
}

#[derive(Clone, Copy)]
pub enum SynthyEvent {
    SetFloatParam { param: SynthyFloatParam, value: f32 },
    SetEnvStart { value: f32 },
    SetEnvEnd { value: f32 },
    SetEnvPoint { index: usize, point: Vec2 },
}

impl Model for AppData {
    fn event(&mut self, _cx: &mut Context, event: &mut Event) {
        let set = ParamSetter::new(self.context.as_ref());
        if let Some(app_event) = event.message.downcast().cloned() {
            match app_event {
                SynthyEvent::SetFloatParam { param, value } => match param {
                    SynthyFloatParam::AMod => {
                        set.set_parameter_normalized(&self.params.a_mod, value)
                    }
                    SynthyFloatParam::ARatio => {
                        set.set_parameter_normalized(&self.params.a_ratio, value)
                    }
                    SynthyFloatParam::BMod => {
                        set.set_parameter_normalized(&self.params.b_mod, value)
                    }
                    SynthyFloatParam::BRatio => {
                        set.set_parameter_normalized(&self.params.b_ratio, value)
                    }
                    SynthyFloatParam::ABMod => {
                        set.set_parameter_normalized(&self.params.a_mod_b, value)
                    }
                    SynthyFloatParam::NoiseAmp => {
                        set.set_parameter_normalized(&self.params.noise_amp, value)
                    }
                    SynthyFloatParam::FilterFreq => {
                        set.set_parameter_normalized(&self.params.filter_freq, value)
                    }
                    SynthyFloatParam::FilterQ => {
                        set.set_parameter_normalized(&self.params.filter_q, value)
                    }
                },
                SynthyEvent::SetEnvPoint {
                    index,
                    point: new_point,
                } => {
                    AppData::params.map(move |x| {
                        {
                            //
                            if let Ok(mut points) = x.env.try_write() {
                                if let Some(point) = points.get_mut(index) {
                                    point.x = new_point.x;
                                    point.y = new_point.y;
                                }
                            }
                        }
                    });
                }
                SynthyEvent::SetEnvStart { value } => {
                    self.env_zoom_view = value..=*self.env_zoom_view.end();
                }
                SynthyEvent::SetEnvEnd { value } => {
                    self.env_zoom_view = *self.env_zoom_view.start()..=value;
                }
            }
        }
    }
}

pub fn knob(
    cx: &mut Context,
    name: impl AsRef<str>,
    param: SynthyFloatParam,
    lens: Then<ViziaParams, Map<Pin<Arc<SynthyParams>>, f32>>,
) {
    VStack::new(cx, move |cx| {
        Label::new(cx, name.as_ref());
        Knob::new(cx, 0.0, lens.clone(), true)
            .on_changing(move |cx, value| cx.emit(SynthyEvent::SetFloatParam { param, value }));
        Label::new(cx, lens.clone());
    });
}

pub fn ui(cx: &mut Context, params: Pin<Arc<SynthyParams>>, context: Arc<dyn GuiContext>) {
    cx.add_theme(STYLE);

    AppData {
        params,
        context: context.clone(),
        env_zoom_view: 0f32..=1f32,
    }
    .build(cx);

    VStack::new(cx, |cx| {
        // knob(
        //     cx,
        //     "a mod",
        //     SynthyFloatParam::AMod,
        //     AppData::params.map(|params| params.a_mod.normalized_value()),
        // );

        // knob(
        //     cx,
        //     "b mod",
        //     SynthyFloatParam::BMod,
        //     AppData::params.map(|params| params.b_mod.normalized_value()),
        // );
        Mseg::new(
            cx,
            AppData::params.map(|params| params.env.read().unwrap().clone()),
            AppData::env_zoom_view,
        )
        .on_changing_range_start(|cx, x| {
            cx.emit(SynthyEvent::SetEnvStart { value: x })
        })
        .on_changing_range_end(|cx, x|{
            cx.emit(SynthyEvent::SetEnvEnd { value: x })
        })
        .on_changing_point(|cx, index, point| {
            cx.emit(SynthyEvent::SetEnvPoint { index, point });
        });

        //crate::widgets::zoomer::Zoomer::new(cx, AppData::zoom_view);
    })
    .left(Pixels(100f32))
    .top(Pixels(100f32))
    .width(Pixels(500f32));
}
