use crate::ruffbox::synth::Source;
use crate::ruffbox::synth::SynthState;
use crate::ruffbox::synth::SynthParameter;

use std::f32::consts::PI;

/**
 * A simple sine oscillator
 */
pub struct SineOsc {
    //freq: f32,
    lvl: f32,
    //samplerate: f32,
    dur: f32,
    //dur_samples: f32,
    sin_time: f32,
    sin_delta_time: f32,
    pi_slice: f32,
    state: SynthState,
}

impl SineOsc {    
    pub fn new(freq: f32, lvl: f32, dur: f32, sr: f32) -> Self {
        SineOsc {
            //freq: freq,
            lvl: lvl,
            //samplerate: sr,
            dur: dur,
            //dur_samples: dur * sr,
            sin_time: 0.0,
            sin_delta_time: 1.0 / sr,
            pi_slice: 2.0 * PI * freq,
            state: SynthState::Fresh,
        }
    }
}

impl Source for SineOsc {

    // some parameter limits might be nice ... 
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::PitchFrequency => self.pi_slice = 2.0 * PI * value,
            SynthParameter::Duration => self.dur = value,
            SynthParameter::Level => self.lvl = value, 
            _ => (),
        };
    }
    
    fn finish(&mut self) {
        self.state = SynthState::Finished;
    }

    fn is_finished(&self) -> bool {
        match self.state {
            SynthState::Finished => true,
            _ => false,
        }
    }

    fn get_next_block(&mut self, start_sample: usize) -> [f32; 128] {
        let mut out_buf: [f32; 128] = [0.0; 128];

        for i in start_sample..128 {
            out_buf[i] = (self.pi_slice * self.sin_time).sin() * self.lvl;
            self.sin_time += self.sin_delta_time;
            if self.sin_time >= self.dur {
                self.finish();
            }
        }

        out_buf
    }
}
