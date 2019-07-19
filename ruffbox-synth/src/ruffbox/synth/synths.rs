use crate::ruffbox::synth::*;
use crate::ruffbox::synth::oscillators::*;
use crate::ruffbox::synth::envelopes::*;
use crate::ruffbox::synth::Source;
use crate::ruffbox::synth::SynthParameter;

/// a sinusoidal synth with envelope etc.
pub struct SineSynth {
    oscillator: SineOsc,
    envelope: ASREnvelope,    
}

impl SineSynth {
    pub fn new(sr: f32) -> Self {
        SineSynth {
            oscillator: SineOsc::new(440.0, 0.3, 0.3, sr),
            envelope: ASREnvelope::new(sr, 0.3, 0.05, 0.1, 0.05),
        }
    }
}

impl Source for SineSynth {
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.oscillator.set_parameter(par, val);
        self.envelope.set_parameter(par, val);
    }

    fn finish(&mut self) {
        self.oscillator.finish();
    }

    fn is_finished(&self) -> bool {
        self.oscillator.is_finished()
    }

    fn get_next_block(&mut self, start_sample: usize) -> [f32; 128] {
        let mut out: [f32; 128] = self.oscillator.get_next_block(start_sample);
        out = self.envelope.process_block(out, start_sample);
        out
    }
}
