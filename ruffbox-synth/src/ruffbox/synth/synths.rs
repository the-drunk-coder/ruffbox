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
pub struct SineSynth<const BUFSIZE:usize> {
    oscillator: SineOsc<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: Balance2<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl <const BUFSIZE:usize> SineSynth<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        SineSynth {
            oscillator: SineOsc::new(440.0, 0.5, sr),
            envelope: ASREnvelope::new(sr, 0.3, 0.05, 0.1, 0.05),
            balance: Balance2::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl <const BUFSIZE:usize> StereoSynth<BUFSIZE> for SineSynth<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);        
        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };        
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize) -> [[f32; BUFSIZE]; 2] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}

/// a low-frequency sawtooth synth with envelope and lpf18 filter
pub struct LFSawSynth<const BUFSIZE:usize> {
    oscillator: LFSaw<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: Balance2<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl <const BUFSIZE:usize> LFSawSynth<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        LFSawSynth {
            oscillator: LFSaw::new(100.0, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: ASREnvelope::new(sr, 1.0, 0.002, 0.02, 0.08),
            balance: Balance2::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl <const BUFSIZE:usize> StereoSynth<BUFSIZE> for LFSawSynth<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize) -> [[f32; BUFSIZE]; 2] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample);
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 { 
        self.reverb
    }

    fn delay_level(&self) -> f32 { 
        self.delay
    }
}

/// a low-frequency (non-bandlimited) squarewave synth with envelope and lpf18 filter
pub struct LFSquareSynth<const BUFSIZE:usize> {
    oscillator: LFSquare<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    balance: Balance2<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl <const BUFSIZE:usize> LFSquareSynth<BUFSIZE> {
    pub fn new(sr: f32) -> Self {
        LFSquareSynth {
            oscillator: LFSquare::new(100.0, 0.4, 0.8, sr),
            filter: Lpf18::new(1500.0, 0.5, 0.1, sr),
            envelope: ASREnvelope::new(sr, 1.0, 0.002, 0.02, 0.08),
            balance: Balance2::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl <const BUFSIZE:usize> StereoSynth<BUFSIZE> for LFSquareSynth<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        match par {
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize) -> [[f32; BUFSIZE]; 2] {
        let mut out: [f32; BUFSIZE] = self.oscillator.get_next_block(start_sample);
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 { 
        self.reverb
    }

    fn delay_level(&self) -> f32 { 
        self.delay
    }
}

/// a sampler with envelope etc.
pub struct StereoSampler<const BUFSIZE:usize> {
    sampler: Sampler<BUFSIZE>,
    envelope: ASREnvelope<BUFSIZE>,
    filter: Lpf18<BUFSIZE>,
    balance: Balance2<BUFSIZE>,
    reverb: f32,
    delay: f32,
}

impl <const BUFSIZE:usize> StereoSampler<BUFSIZE> {
    pub fn with_buffer_ref(buf: &Arc<Vec<f32>>, sr: f32) -> StereoSampler<BUFSIZE> {
        let dur = (buf.len() as f32 / sr) - 0.0002;
        
        StereoSampler {
            sampler: Sampler::with_buffer_ref(buf, true),
            envelope: ASREnvelope::new(sr, 1.0, 0.0001, dur, 0.0001),
            filter: Lpf18::new(19500.0, 0.01, 0.01, sr),
            balance: Balance2::new(),
            reverb: 0.0,
            delay: 0.0,
        }
    }
}

impl <const BUFSIZE:usize> StereoSynth<BUFSIZE> for StereoSampler<BUFSIZE> {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.sampler.set_parameter(par, val);        
        self.filter.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
        self.balance.set_parameter(par, val);

        match par {            
            SynthParameter::ReverbMix => self.reverb = val,
            SynthParameter::DelayMix => self.delay = val,
            _ => (),
        };
    }

    fn finish(&mut self) {
        self.envelope.finish();
    }

    fn is_finished(&self) -> bool {
        self.envelope.is_finished()   
    }

    fn get_next_block(&mut self, start_sample: usize) -> [[f32; BUFSIZE]; 2] {
        let mut out: [f32; BUFSIZE] = self.sampler.get_next_block(start_sample);
        out = self.filter.process_block(out, start_sample);
        out = self.envelope.process_block(out, start_sample);
        self.balance.process_block(out)
    }

    fn reverb_level(&self) -> f32 {
        self.reverb
    }

    fn delay_level(&self) -> f32 {
        self.delay
    }
}
