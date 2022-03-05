#![feature(trait_alias)]
pub mod editor;
pub mod ui;

use editor::create_vizia_editor;
use fundsp::hacker::*;
use nih_plug::{nih_export_vst3, prelude::*, util::midi_note_to_freq};
use num_derive::FromPrimitive;
use std::{
    pin::Pin,
    sync::{Arc, RwLock},
    time::Duration,
};

type Note = u8;
type Velocity = u8;
type Stage = usize;

struct Synthy {
    audio: Box<dyn AudioUnit64 + Send + Sync>,
    sample_rate: f32,
    params: Pin<Arc<SynthyParams>>,
    time: Duration,
    note: Option<NoteInfo>,
    enabled: bool,
}

struct NoteInfo {
    note: Note,
    velocity: Velocity,
    on: Duration,
    stage: usize,
}

pub struct SynthyEditor {}

#[derive(Params)]
pub struct SynthyParams {
    #[id = "a_mod"]
    pub a_mod: FloatParam,
    #[id = "a_ratio"]
    pub a_ratio: FloatParam,
    #[persist = "a_env"]
    pub a_env: RwLock<Vec<(f32, f32)>>,
    #[persist = "b_env"]
    pub b_env: RwLock<Vec<(f32, f32)>>,
    #[persist = "noise_env"]
    pub noise_env: RwLock<Vec<(f32, f32)>>,
    #[persist = "env"]
    pub env: RwLock<Vec<(f32, f32)>>,
    #[id = "b_mod"]
    pub b_mod: FloatParam,
    #[id = "b_ratio"]
    pub b_ratio: FloatParam,
    #[id = "a_b_mod"]
    pub a_mod_b: FloatParam,
    #[id = "noise_amp"]
    pub noise_amp: FloatParam,
    #[id = "filter_freq"]
    pub filter_freq: FloatParam,
    #[id = "filter_q"]
    pub filter_q: FloatParam,
}

impl Default for SynthyParams {
    fn default() -> Self {
        Self {
            a_mod: FloatParam::new(
                "op a mod",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_value_to_string(formatters::f32_rounded(2)),
            a_ratio: FloatParam::new("op a ratio", 1.0, FloatRange::Linear { min: 0.0, max: 8.0 })
                .with_value_to_string(formatters::f32_rounded(2)),
            b_mod: FloatParam::new(
                "op b mod",
                0.5,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_value_to_string(formatters::f32_rounded(2)),
            b_ratio: FloatParam::new("op b ratio", 2.0, FloatRange::Linear { min: 0.0, max: 8.0 })
                .with_value_to_string(formatters::f32_rounded(2)),
            a_mod_b: FloatParam::new(
                "op ab mod",
                0.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 10.0,
                },
            )
            .with_value_to_string(formatters::f32_rounded(2)),
            noise_amp: FloatParam::new("noise amp", 0.0, FloatRange::Linear { min: 0.0, max: 0.5 })
                .with_value_to_string(formatters::f32_rounded(2)),
            filter_freq: FloatParam::new(
                "cutoff",
                25_000.0,
                FloatRange::Linear {
                    min: 0.0,
                    max: 25_000.0,
                },
            )
            .with_value_to_string(formatters::f32_rounded(2)),
            filter_q: FloatParam::new("resonance", 0.2, FloatRange::Linear { min: 0.0, max: 1.0 })
                .with_value_to_string(formatters::f32_rounded(2)),
            a_env: RwLock::new(vec![
                (0f32, 0f32),
                (0.5f32, 1.0f32),
                (1.0f32, 0.7f32),
                (2.0f32, 0.5f32),
                (3.0f32, 0.0f32),
            ]),
            b_env: RwLock::new(vec![
                (0f32, 0f32),
                (0.5f32, 1.0f32),
                (1.0f32, 0.7f32),
                (2.0f32, 0.5f32),
                (3.0f32, 0.0f32),
            ]),
            noise_env: RwLock::new(vec![
                (0f32, 0f32),
                (0.5f32, 1.0f32),
                (1.0f32, 0.7f32),
                (2.0f32, 0.5f32),
                (3.0f32, 0.0f32),
            ]),
            env: RwLock::new(vec![
                (0f32, 0f32),
                (0.5f32, 1.0f32),
                (1.0f32, 0.7f32),
                (2.0f32, 0.5f32),
                (3.0f32, 0.0f32),
            ]),
        }
    }
}

impl Default for Synthy {
    #[allow(clippy::precedence)]
    fn default() -> Self {
        let params = Arc::pin(SynthyParams::default());

        let freq_tag = || tag(Tag::Freq as i64, 0.);
        let cutoff_tag = || tag(Tag::FilterFreq as i64, 0.);
        let q_tag = || tag(Tag::FilterQ as i64, 0.);
        let wet_tag = || tag(Tag::Wet as i64, 0.);
        let time_tag = || tag(Tag::Time as i64, 0.);
        let noise_amp_tag = || tag(Tag::NoiseAmp as i64, 0.);
        let a_ratio_tag = || tag(Tag::OpARatio as i64, 0.);
        let b_ratio_tag = || tag(Tag::OpBRatio as i64, 0.);
        let a_mod_tag = || tag(Tag::OpAMod as i64, 0.);
        let a_env_tag = || tag(Tag::OpAEnv as i64, 0.);
        let b_env_tag = || tag(Tag::OpBEnv as i64, 0.);
        let noise_env_tag = || tag(Tag::NoiseEnv as i64, 0.);
        let env_tag = || tag(Tag::Env as i64, 0.) >> !declick();
        let b_mod_tag = || tag(Tag::OpBMod as i64, 0.);
        let a_b_mod_tag = || tag(Tag::OpAModB as i64, 0.);

        let op = |ratio, modulation, envelope| {
            freq_tag() * ratio >> envelope * sine() * freq_tag() * modulation + freq_tag()
        };

        // Operators
        let a = || op(a_ratio_tag(), a_mod_tag(), a_env_tag());
        let b = || op(b_ratio_tag(), b_mod_tag(), b_env_tag());
        let n = || noise() >> bandpass_hz(2000., 0.75) * noise_amp_tag() * noise_env_tag();
        // let ab = || a() >> b();

        let gen = ((a() & b()) >> (sine() * env_tag())) & n();
        let mix = // = (saw_hz(500.) ^ cutoff_tag() ^ q_tag()) >> lowpass();
         gen >> declick() >> split::<U2>();
        // >> reverb_stereo(wet(), time());

        Self {
            audio: Box::new(mix) as Box<dyn AudioUnit64 + Send + Sync>,
            sample_rate: Default::default(),
            time: Duration::default(),
            note: None,
            enabled: false,
            params,
        }
    }
}

impl Plugin for Synthy {
    const NAME: &'static str = "synthy";
    const VENDOR: &'static str = "rust audio";
    const URL: &'static str = "https://vaporsoft.net";
    const EMAIL: &'static str = "myemail@example.com";
    const VERSION: &'static str = "0.0.1";
    const DEFAULT_NUM_INPUTS: u32 = 0;
    const DEFAULT_NUM_OUTPUTS: u32 = 2;
    const ACCEPTS_MIDI: bool = true;

    fn params(&self) -> Pin<&dyn Params> {
        self.params.as_ref()
    }

    fn process(&mut self, buffer: &mut Buffer, context: &mut impl ProcessContext) -> ProcessStatus {
        for (_offset, mut block) in buffer.iter_blocks(MAX_BUFFER_SIZE) {
            self.audio
                .set(Tag::OpAMod as i64, self.params.a_mod.value as f64);
            self.audio
                .set(Tag::OpBMod as i64, self.params.b_mod.value as f64);
            self.audio
                .set(Tag::OpARatio as i64, self.params.a_ratio.value as f64);
            self.audio
                .set(Tag::OpBRatio as i64, self.params.b_ratio.value as f64);
            self.audio
                .set(Tag::OpAModB as i64, self.params.a_mod_b.value as f64);
            self.audio
                .set(Tag::NoiseAmp as i64, self.params.noise_amp.value as f64);
            self.audio
                .set(Tag::FilterFreq as i64, self.params.filter_freq.value as f64);
            self.audio
                .set(Tag::FilterQ as i64, self.params.filter_q.value as f64);

            let midi = context.next_midi_event();
            if let Some(event) = midi {
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.enabled = true;
                        self.audio
                            .set(Tag::Freq as i64, midi_note_to_freq(note) as f64);

                        self.note = Some(NoteInfo {
                            note,
                            velocity,
                            on: self.time,
                            stage: 0,
                        });
                    }
                    NoteEvent::NoteOff { note, velocity, .. } => {
                        if let Some(current_note) = &mut self.note {
                            let params = self.params.env.read().unwrap();
                            if current_note.note == note {
                                current_note.velocity = velocity;
                                current_note.stage = params.len() - 2;
                                if let Ok(params) = self.params.env.read() {
                                    // TODO: figure out how to offset the time here
                                    // current_note.on = Duration::from_secs_f32(

                                    //     // current_note.on.as_secs_f32()
                                    //     //     - (params.last().unwrap().0
                                    //     //         - params
                                    //     //             .get(params.len() - 2)
                                    //     //             .unwrap()
                                    //     //             .0
                                    //     //             .min(0f32))),
                                    // );
                                }
                            }
                        }
                        if Some(note) == self.note.as_ref().map(|x| x.note) {}
                    }
                }
            }

            // Calculate main env notes on and off
            if let Ok(envelope) = self.params.env.read() {
                if let Some(note) = &mut self.note {
                    let relative_time = self.time - note.on;
                    // increase the point counter if more than the next point
                    if let Some(next_point) = envelope.get(note.stage + 1) {
                        if relative_time.as_secs_f32() >= next_point.0 {
                            note.stage += 1;
                        }
                    }
                    if note.stage == envelope.len() {
                        // We have reached the end of the envelope. Trigger a note off
                        self.note = None;
                    }
                }
            }

            // lerp between the two points based on note stage
            let mut set_env = |param: &RwLock<Vec<(f32, f32)>>, tag| {
                if let Some(note) = &self.note {
                    let relative_time = self.time - note.on;
                    if let Ok(envelope) = param.read() {
                        if let (Some(left), Some(right)) =
                            (envelope.get(note.stage), envelope.get(note.stage + 1))
                        {
                            let normalized =
                                (relative_time.as_secs_f32() - left.0) / (right.0 - left.0);
                            let val = lerp(left.1, right.1, normalized);
                            self.audio.set(tag as i64, val as f64);
                        }
                    }
                }
            };

            set_env(&self.params.a_env, Tag::OpAEnv);
            set_env(&self.params.b_env, Tag::OpBEnv);
            set_env(&self.params.noise_env, Tag::NoiseEnv);
            set_env(&self.params.env, Tag::Env);

            // if let Some(note) = &midi {
            //     if let NoteEvent::NoteOn { note, velocity, .. } = note {
            //         self.audio
            //             .set(Tag::Freq as i64, midi_note_to_freq(*note) as f64);
            //         self.enabled = true;
            //         self.note = Some(NoteInfo {
            //             note: *note,
            //             velocity: *velocity,
            //             on: self.time,
            //             stage: 0,
            //         });
            //     }
            // }

            // // get the envelope amplitude at this position in time
            // let mut set_env = |param: &RwLock<Vec<(f32, f32)>>, tag| {
            //     if let Ok(env_amp) = param.read() {
            //         if let Some(note) = &mut self.note {
            //             // check if we need to bump the note index
            //             let relative_time = self.time - note.on;
            //             let next_point = env_amp
            //                 .get(note.stage + 1)
            //                 .unwrap_or_else(|| env_amp.last().unwrap());
            //             if relative_time.as_secs_f32() >= next_point.0 {
            //                 // Increment the stage
            //                 note.stage += 1;
            //             }

            //             // set to release envelope
            //             if let Some(NoteEvent::NoteOff {
            //                 timing,
            //                 channel,
            //                 note: current_note,
            //                 velocity,
            //             }) = midi
            //             {
            //                 if current_note == note.note {
            //                     // set to second to last for release
            //                     note.stage = env_amp.len() - 2;
            //                 }
            //             }

            //             let left = env_amp
            //                 .get(note.stage)
            //                 .unwrap_or_else(|| env_amp.last().unwrap());
            //             let right = env_amp
            //                 .get(note.stage + 1)
            //                 .unwrap_or_else(|| env_amp.last().unwrap());

            //             let normalized =
            //                 (relative_time.as_secs_f32() - left.0) / (right.0 - left.0);

            //             let val = lerp(left.1, right.1, normalized);
            //             self.audio.set(tag as i64, val as f64);
            //         }
            //     }
            // };

            //
            //
            //
            //

            let mut left_tmp = [0f64; MAX_BUFFER_SIZE];
            let mut right_tmp = [0f64; MAX_BUFFER_SIZE];

            if self.enabled {
                self.time += Duration::from_secs_f32(MAX_BUFFER_SIZE as f32 / self.sample_rate);
                self.audio
                    .process(MAX_BUFFER_SIZE, &[], &mut [&mut left_tmp, &mut right_tmp]);
            }

            for (index, channel) in block.iter_mut().enumerate() {
                let new_channel = match index {
                    0 => left_tmp,
                    1 => right_tmp,
                    _ => return ProcessStatus::Error("unexpected number of channels"),
                };
                for (sample_index, sample) in channel.iter_mut().enumerate() {
                    *sample = new_channel[sample_index] as f32;
                }
            }
        }

        ProcessStatus::Normal
    }

    fn initialize(
        &mut self,
        _bus_config: &BusConfig,
        buffer_config: &BufferConfig,
        _context: &mut impl ProcessContext,
    ) -> bool {
        // Set up logs, adapted from code from DGriffin91
        // MIT: https://github.com/DGriffin91/egui_baseview_test_vst2/blob/main/LICENSE
        let home = dirs::home_dir().unwrap().join("tmp");
        let id_string = format!("{}-{}-log.txt", Self::NAME, Self::VERSION);
        let log_file = std::fs::File::create(home.join(id_string)).unwrap();
        let log_config = ::simplelog::ConfigBuilder::new()
            .set_time_to_local(true)
            .build();
        simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file).ok();
        log_panics::init();
        log::info!("init");
        self.sample_rate = buffer_config.sample_rate;
        true
    }

    fn editor(&self) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        create_vizia_editor(move |cx, context| {
            ui::ui(cx, params.clone(), context.clone());
        })
    }
}

#[derive(FromPrimitive, Clone, Copy)]
pub enum Tag {
    Freq,
    OpAMod,
    OpBMod,
    OpAModB,
    Env,
    OpAEnv,
    OpBEnv,
    NoiseEnv,
    OpARatio,
    OpBRatio,
    Wet,
    Time,
    FilterFreq,
    FilterQ,
    NoiseAmp,
}

impl Vst3Plugin for Synthy {
    const VST3_CLASS_ID: [u8; 16] = *b"1234567891234567";
    const VST3_CATEGORIES: &'static str = "Instrument|Synth";
}

nih_export_vst3!(Synthy);
