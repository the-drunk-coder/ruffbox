use crate::ruffbox::synth::*;
use crate::ruffbox::synth::oscillators::*;
use crate::ruffbox::synth::envelopes::*;
use crate::ruffbox::synth::filters::*;
use crate::ruffbox::synth::routing::Balance2;
use crate::ruffbox::synth::sampler::Sampler;
use crate::ruffbox::synth::StereoSynth;
use crate::ruffbox::synth::SynthParameter;


use std::sync::Arc;

/// a sinusoidal synth with envelope etc.
pub struct SineSynth {
    oscillator: SineOsc,
    envelope: ASREnvelope,
    balance: Balance2,
}

impl SineSynth {
    pub fn new(sr: f32) -> Self {
        SineSynth {
            oscillator: SineOsc::new(440.0, 0.5, sr),
            envelope: ASREnvelope::new(sr, 0.3, 0.05, 0.1, 0.05),
            balance: Balance2::new(),
        }
    }
}

impl StereoSynth for SineSynth {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize) -> [[f32; 128]; 2] {
        let mut out: [f32; 128] = self.oscillator.get_next_block(start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }
}

/// a low-frequency sawtooth synth with envelope and lpf18 filter
pub struct LFSawSynth {
    oscillator: LFSaw,
    filter: Lpf18,
    envelope: ASREnvelope,
    balance: Balance2,
}

impl LFSawSynth {
    pub fn new(sr: f32) -> Self {
        LFSawSynth {
            oscillator: LFSaw::new(100.0, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: ASREnvelope::new(sr, 1.0, 0.002, 0.02, 0.08),
            balance: Balance2::new(),
        }
    }
}

impl StereoSynth for LFSawSynth {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize) -> [[f32; 128]; 2] {
        let mut out: [f32; 128] = self.oscillator.get_next_block(start_sample);
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }
}

/// a sinusoidal synth with envelope etc.
pub struct StereoSampler {
    sampler: Sampler,
    balance: Balance2,
}

impl StereoSampler {
    pub fn with_buffer_ref(buf: &Arc<Vec<f32>>) -> StereoSampler {        
        StereoSampler {
            sampler: Sampler::with_buffer_ref(buf),
            balance: Balance2::new(),
        }
    }
}

impl StereoSynth for StereoSampler {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.sampler.set_parameter(par, val);        
        self.balance.set_parameter(par, val);        
    }

    fn finish(&mut self) {
        self.sampler.finish();
    }

    fn is_finished(&self) -> bool {
        self.sampler.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize) -> [[f32; 128]; 2] {
        let out: [f32; 128] = self.sampler.get_next_block(start_sample);
        self.balance.process_block(out)
    }
}
