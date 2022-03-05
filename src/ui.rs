use crate::SynthyParams;
use nih_plug::prelude::*;
use std::{pin::Pin, sync::Arc};
use vizia::Context;

pub(crate) fn ui(cx: &mut Context, params: Pin<Arc<SynthyParams>>, context: Arc<dyn GuiContext>) {}
