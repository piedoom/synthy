#![feature(trait_alias)]
pub mod editor;
pub mod ui;
pub mod util;
pub mod widgets;

use editor::create_vizia_editor;
use fundsp::hacker::*;
use nih_plug::{nih_export_vst3, prelude::*, util::midi_note_to_freq};
use num_derive::FromPrimitive;
use std::{
    pin::Pin,
    sync::{Arc, RwLock},
    time::Duration,
};
use util::{CurvePoint, CurvePoints};

type Note = u8;
type Velocity = u8;
type Stage = usize;

struct Synthy {
    audio: Box<dyn AudioUnit64 + Send + Sync>,
    sample_rate: f32,
    params: Pin<Arc<SynthyParams>>,
    /// `f32` seconds since the plugin started
    time: f32,
    note: Option<NoteInfo>,
    enabled: bool,
}

struct NoteInfo {
    note: Note,
    velocity: Velocity,
    /// Duration of seconds, as `f32` since the plugin timer has started
    /// This is stored as an `f32` and not a `Duration` to allow for
    /// negative values, which we use for envelope generation
    on: f32,
    stage: usize,
}

pub struct SynthyEditor {}

#[derive(Copy, Clone)]
pub enum SynthyFloatParam {
    AMod,
    ARatio,
    BMod,
    BRatio,
    ABMod,
    NoiseAmp,
    FilterFreq,
    FilterQ,
}

#[derive(Copy, Clone)]
pub enum SynthyEnvParam {
    AEnv,
    BEnv,
    Env,
    NoiseEnv,
}

#[derive(Params)]
pub struct SynthyParams {
    #[id = "a_mod"]
    pub a_mod: FloatParam,
    #[id = "a_ratio"]
    pub a_ratio: FloatParam,
    #[persist = "a_env"]
    pub a_env: RwLock<CurvePoints>,
    #[persist = "b_env"]
    pub b_env: RwLock<CurvePoints>,
    #[persist = "noise_env"]
    pub noise_env: RwLock<CurvePoints>,
    #[persist = "env"]
    pub env: RwLock<CurvePoints>,
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
            a_env: RwLock::new(CurvePoints::new(
                vec![
                    (0f32, 0f32),
                    (0.5f32, 1.0f32),
                    (1.0f32, 0.7f32),
                    (2.0f32, 0.5f32),
                    (3.0f32, 0.0f32),
                ]
                .iter()
                .cloned()
                .map(CurvePoint::from)
                .collect(),
            )),
            b_env: RwLock::new(CurvePoints::new(
                vec![
                    (0f32, 0f32),
                    (0.5f32, 1.0f32),
                    (1.0f32, 0.7f32),
                    (2.0f32, 0.5f32),
                    (3.0f32, 0.0f32),
                ]
                .iter()
                .cloned()
                .map(CurvePoint::from)
                .collect(),
            )),
            noise_env: RwLock::new(CurvePoints::new(
                vec![
                    (0f32, 0f32),
                    (0.5f32, 1.0f32),
                    (1.0f32, 0.7f32),
                    (2.0f32, 0.5f32),
                    (3.0f32, 0.0f32),
                ]
                .iter()
                .cloned()
                .map(CurvePoint::from)
                .collect(),
            )),
            env: RwLock::new(CurvePoints::new(
                vec![
                    (0f32, 0f32),
                    (0.5f32, 1.0f32),
                    (1.0f32, 0.7f32),
                    (2.0f32, 0.5f32),
                    (3.0f32, 0.0f32),
                ]
                .iter()
                .cloned()
                .map(CurvePoint::from)
                .collect(),
            )),
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
            time: 0f32,
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
                                let index = params.len() - 2;
                                if current_note.stage < index {
                                    // Jump to new stage
                                    current_note.stage = index;
                                    if let Ok(params) = self.params.env.read() {
                                        // TODO: figure out how to offset the time here
                                        // Get relative offset in seconds of the jump
                                        let relative_time = current_note.on - self.time;
                                        let relative_offset = match params.get(index) {
                                            Some(point) => point.x + relative_time,
                                            None => 0f32,
                                        };
                                        // Apply offset to the current note
                                        // We subtract time to simulate pressing the note earlier, enough
                                        // to be in the right stage of the envelope.
                                        current_note.on -= relative_offset
                                    }
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
                        if relative_time >= next_point.x {
                            note.stage += 1;
                        }
                    } else {
                        // We have reached the end of the envelope. Trigger a note off
                        self.note = None;
                    }
                }
            }

            // lerp between the two points based on note stage
            let mut set_env = |param: &RwLock<CurvePoints>, tag| {
                if let Some(note) = &self.note {
                    let relative_time = self.time - note.on;
                    if let Ok(envelope) = param.read() {
                        if let (Some(left), Some(right)) =
                            (envelope.get(note.stage), envelope.get(note.stage + 1))
                        {
                            let normalized = (relative_time - left.x) / (right.x - left.x);
                            let val = lerp(left.y, right.y, normalized);
                            self.audio.set(tag as i64, val as f64);
                        }
                    }
                }
            };

            set_env(&self.params.a_env, Tag::OpAEnv);
            set_env(&self.params.b_env, Tag::OpBEnv);
            set_env(&self.params.noise_env, Tag::NoiseEnv);
            set_env(&self.params.env, Tag::Env);

            let mut left_tmp = [0f64; MAX_BUFFER_SIZE];
            let mut right_tmp = [0f64; MAX_BUFFER_SIZE];

            if self.enabled {
                self.time += MAX_BUFFER_SIZE as f32 / self.sample_rate;
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
