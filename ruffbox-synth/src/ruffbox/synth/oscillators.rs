use crate::ruffbox::synth::Source;
use crate::ruffbox::synth::SourceState;
use crate::ruffbox::synth::SourceParameter;

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
    state: SourceState,
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
            state: SourceState::Fresh,
        }
    }
}

impl Source for SineOsc {

    // some parameter limits might be nice ... 
    fn set_parameter(&mut self, par: SourceParameter, value: f32) {
        match par {
            SourceParameter::PitchFrequency => self.pi_slice = 2.0 * PI * value,
            SourceParameter::Duration => self.dur = value,
            SourceParameter::Level => self.lvl = value, 
            _ => (),
        };
    }
    
    fn finish(&mut self) {
        self.state = SourceState::Finished;
    }

    fn is_finished(&self) -> bool {
        match self.state {
            SourceState::Finished => true,
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
