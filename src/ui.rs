use crate::{
    widgets::mseg::{Mseg, MsegHandle},
    SynthyEnvParam, SynthyFloatParam, SynthyParams,
};
use glam::Vec2;
use nih_plug::prelude::*;
use std::{ops::RangeInclusive, pin::Pin, sync::Arc};
use vizia::*;

const STYLE: &str = include_str!("style.css");

#[derive(Lens)]
pub struct AppData {
    pub params: Pin<Arc<SynthyParams>>,
    #[lens(ignore)]
    pub context: Arc<dyn GuiContext>,
    env_zoom_view: RangeInclusive<f32>,
    a_env_zoom_view: RangeInclusive<f32>,
    b_env_zoom_view: RangeInclusive<f32>,
    noise_env_zoom_view: RangeInclusive<f32>,
}

#[derive(Clone, Copy)]
pub enum SynthyEvent {
    SetFloatParam {
        param: SynthyFloatParam,
        value: f32,
    },
    SetEnvStart {
        param: SynthyEnvParam,
        value: f32,
    },
    SetEnvEnd {
        param: SynthyEnvParam,
        value: f32,
    },
    SetEnvPoint {
        param: SynthyEnvParam,
        index: usize,
        point: Vec2,
    },
    InsertPoint {
        param: SynthyEnvParam,
        index: usize,
        point: Vec2,
    },
    RemovePoint {
        param: SynthyEnvParam,
        index: usize,
    },
}

impl Model for AppData {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
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
                    param,
                } => {
                    let params = self.params.clone();
                    let writer = {
                        match param {
                            SynthyEnvParam::AEnv => params.a_env.try_write(),
                            SynthyEnvParam::BEnv => params.b_env.try_write(),
                            SynthyEnvParam::Env => params.env.try_write(),
                            SynthyEnvParam::NoiseEnv => params.noise_env.try_write(),
                        }
                    };
                    if let Ok(mut points) = writer {
                        if let Some(point) = points.get_mut(index) {
                            point.x = new_point.x;
                            point.y = new_point.y;
                        }
                    }
                }
                SynthyEvent::SetEnvStart { value, param } => {
                    let writer = {
                        match param {
                            SynthyEnvParam::AEnv => &mut self.a_env_zoom_view,
                            SynthyEnvParam::BEnv => &mut self.b_env_zoom_view,
                            SynthyEnvParam::Env => &mut self.env_zoom_view,
                            SynthyEnvParam::NoiseEnv => &mut self.noise_env_zoom_view,
                        }
                    };
                    let end = *writer.end();
                    *writer = value..=end;
                }
                SynthyEvent::SetEnvEnd { value, param } => {
                    let writer = {
                        match param {
                            SynthyEnvParam::AEnv => &mut self.a_env_zoom_view,
                            SynthyEnvParam::BEnv => &mut self.b_env_zoom_view,
                            SynthyEnvParam::Env => &mut self.env_zoom_view,
                            SynthyEnvParam::NoiseEnv => &mut self.noise_env_zoom_view,
                        }
                    };
                    let start = *writer.start();
                    *writer = start..=value;
                }
                SynthyEvent::InsertPoint {
                    index,
                    point,
                    param,
                } => {
                    todo!();
                }
                SynthyEvent::RemovePoint { index, param } => {
                    let params = self.params.clone();
                    let writer = {
                        match param {
                            SynthyEnvParam::AEnv => params.a_env.try_write(),
                            SynthyEnvParam::BEnv => params.b_env.try_write(),
                            SynthyEnvParam::Env => params.env.try_write(),
                            SynthyEnvParam::NoiseEnv => params.noise_env.try_write(),
                        }
                    };
                    if let Ok(mut param) = writer {
                        param.remove(index);
                    }
                }
            }
        }
    }
}

pub fn knob(
    cx: &mut Context,
    name: impl AsRef<str>,
    param: SynthyFloatParam,
    value: impl Lens<Target = (f32, String)>,
    width: Units,
) {
    VStack::new(cx, move |cx| {
        Label::new(cx, name.as_ref()).width(width);
        Knob::new(cx, 0.0, value.clone().map(|x| x.0), false)
            .on_changing(move |cx, value| cx.emit(SynthyEvent::SetFloatParam { param, value }))
            .left(Stretch(1f32))
            .right(Stretch(1f32));
        Label::new(cx, value.map(|x| x.1.clone())).width(width);
    })
    .width(width);
}

pub fn ui(cx: &mut Context, params: Pin<Arc<SynthyParams>>, context: Arc<dyn GuiContext>) {
    cx.add_theme(STYLE);

    AppData {
        params,
        context: context.clone(),
        env_zoom_view: 0f32..=1f32,
        a_env_zoom_view: 0f32..=1f32,
        b_env_zoom_view: 0f32..=1f32,
        noise_env_zoom_view: 0f32..=1f32,
    }
    .build(cx);

    VStack::new(cx, |cx| {
        HStack::new(cx, |cx| {
            knob(
                cx,
                "a mod",
                SynthyFloatParam::AMod,
                AppData::params
                    .map(|params| (params.a_mod.normalized_value(), params.a_mod.to_string())),
                Pixels(72f32),
            );
            knob(
                cx,
                "a ratio",
                SynthyFloatParam::ARatio,
                AppData::params.map(|params| {
                    (
                        params.a_ratio.normalized_value(),
                        params.a_ratio.to_string(),
                    )
                }),
                Pixels(72f32),
            );
            Mseg::new(
                cx,
                AppData::params.map(|params| params.clone().a_env.read().unwrap().clone()),
                AppData::a_env_zoom_view,
                8f32,
            )
            .on_changing_range_start(|cx, x| {
                cx.emit(SynthyEvent::SetEnvStart {
                    param: SynthyEnvParam::AEnv,
                    value: x,
                })
            })
            .on_changing_range_end(|cx, x| {
                cx.emit(SynthyEvent::SetEnvEnd {
                    param: SynthyEnvParam::AEnv,
                    value: x,
                })
            })
            .on_changing_point(|cx, index, point| {
                cx.emit(SynthyEvent::SetEnvPoint {
                    param: SynthyEnvParam::AEnv,
                    index,
                    point,
                });
            })
            .on_insert_point(|cx, index, point| {
                cx.emit(SynthyEvent::InsertPoint {
                    param: SynthyEnvParam::AEnv,
                    index,
                    point,
                })
            })
            .on_remove_point(|cx, index| {
                cx.emit(SynthyEvent::RemovePoint {
                    param: SynthyEnvParam::AEnv,
                    index,
                })
            });
        });

        HStack::new(cx, |cx| {
            knob(
                cx,
                "b mod",
                SynthyFloatParam::BMod,
                AppData::params
                    .map(|params| (params.b_mod.normalized_value(), params.b_mod.to_string())),
                Pixels(72f32),
            );
            knob(
                cx,
                "b ratio",
                SynthyFloatParam::BRatio,
                AppData::params.map(|params| {
                    (
                        params.b_ratio.normalized_value(),
                        params.b_ratio.to_string(),
                    )
                }),
                Pixels(72f32),
            );
            Mseg::new(
                cx,
                AppData::params.map(|params| params.clone().b_env.read().unwrap().clone()),
                AppData::b_env_zoom_view,
                8f32,
            )
            .on_changing_range_start(|cx, x| {
                cx.emit(SynthyEvent::SetEnvStart {
                    param: SynthyEnvParam::BEnv,
                    value: x,
                })
            })
            .on_changing_range_end(|cx, x| {
                cx.emit(SynthyEvent::SetEnvEnd {
                    param: SynthyEnvParam::BEnv,
                    value: x,
                })
            })
            .on_changing_point(|cx, index, point| {
                cx.emit(SynthyEvent::SetEnvPoint {
                    param: SynthyEnvParam::BEnv,
                    index,
                    point,
                });
            })
            .on_insert_point(|cx, index, point| {
                cx.emit(SynthyEvent::InsertPoint {
                    param: SynthyEnvParam::BEnv,
                    index,
                    point,
                })
            })
            .on_remove_point(|cx, index| {
                cx.emit(SynthyEvent::RemovePoint {
                    param: SynthyEnvParam::BEnv,
                    index,
                })
            });
        });

        Mseg::new(
            cx,
            AppData::params.map(|params| params.clone().env.read().unwrap().clone()),
            AppData::env_zoom_view,
            8f32,
        )
        .on_changing_range_start(|cx, x| {
            cx.emit(SynthyEvent::SetEnvStart {
                param: SynthyEnvParam::Env,
                value: x,
            })
        })
        .on_changing_range_end(|cx, x| {
            cx.emit(SynthyEvent::SetEnvEnd {
                param: SynthyEnvParam::Env,
                value: x,
            })
        })
        .on_changing_point(|cx, index, point| {
            cx.emit(SynthyEvent::SetEnvPoint {
                param: SynthyEnvParam::Env,
                index,
                point,
            });
        })
        .on_insert_point(|cx, index, point| {
            cx.emit(SynthyEvent::InsertPoint {
                param: SynthyEnvParam::Env,
                index,
                point,
            })
        })
        .on_remove_point(|cx, index| {
            cx.emit(SynthyEvent::RemovePoint {
                param: SynthyEnvParam::Env,
                index,
            })
        });

        //crate::widgets::zoomer::Zoomer::new(cx, AppData::zoom_view);
    });
}
