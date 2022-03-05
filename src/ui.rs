use crate::{widgets::*, SynthyParams};
use egui::{style::Margin, Context};
use nih_plug::prelude::*;
use std::{pin::Pin, sync::Arc};

#[inline]
pub(crate) fn ui(egui_ctx: &Context, params: Pin<Arc<SynthyParams>>, setter: &ParamSetter) {
    let margin = 16f32;
    egui::CentralPanel::default()
        .frame(
            egui::Frame::default()
                .fill(crate::widgets::Theme::default().colors.background)
                .margin(Margin::symmetric(margin, margin)),
        )
        .show(egui_ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(Knob::from_param(&params.a_mod, setter));
                        ui.add_space(margin);
                        ui.add(Knob::from_param(&params.a_ratio, setter));
                    });
                    ui.add_space(margin);
                    ui.add(
                        Envelope::from_param(&params.a_env, "op a envelope")
                            .size(ui.available_size()),
                    );
                });

                ui.add_space(margin);
                ui.separator();
                ui.add_space(margin);

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(Knob::from_param(&params.b_mod, setter));
                        ui.add_space(margin);
                        ui.add(Knob::from_param(&params.b_ratio, setter));
                    });
                    ui.add_space(margin);
                    ui.add(
                        Envelope::from_param(&params.b_env, "op b envelope")
                            .size(ui.available_size()),
                    );
                });

                ui.add_space(margin);
                ui.separator();
                ui.add_space(margin);

                ui.horizontal(|ui| {
                    ui.add(Slider::from_param(&params.noise_amp, setter));
                    ui.add_space(margin);
                    ui.add(
                        Envelope::from_param(&params.noise_env, "noise envelope")
                            .size(ui.available_size()),
                    );
                });

                ui.add_space(margin);
                ui.separator();
                ui.add_space(margin);

                ui.add(Envelope::from_param(&params.env, "envelope").size(ui.available_size()));
            });
        });
}
