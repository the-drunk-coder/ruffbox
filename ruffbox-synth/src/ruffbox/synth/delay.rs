use crate::ruffbox::synth::filters::*;
use crate::ruffbox::synth::Effect;
use crate::ruffbox::synth::SynthParameter;

pub struct MonoDelay {
    buffer: Vec<f32>,
    buffer_idx: usize,
    max_buffer_idx: usize,
    feedback: f32,
    dampening_filter: Lpf18,
    samplerate: f32,
}

impl MonoDelay {
    pub fn with_max_capacity_sec(capacity_sec: f32, sr: f32) -> Self {
        MonoDelay {
            buffer: vec![0.0; (sr * capacity_sec) as usize],
            buffer_idx: 0,
            max_buffer_idx: (sr * 0.256) as usize, // 512ms default time
            feedback: 0.5,
            dampening_filter: Lpf18::new(3000.0, 0.4, 0.3, sr),
            samplerate: sr,
        }
    }
}

impl Effect for MonoDelay {
    // some parameter limits might be nice ...
    fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        match par {
            SynthParameter::DelayDampeningFrequency => self
                .dampening_filter
                .set_parameter(SynthParameter::LowpassCutoffFrequency, val),
            SynthParameter::DelayFeedback => self.feedback = val,
            SynthParameter::DelayTime => self.max_buffer_idx = (self.samplerate * val) as usize,
            _ => (),
        };
    }

    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool {
        false
    } // it's never finished ..

    // start sample isn't really needed either ...
    fn process_block(&mut self, block: [f32; 128], _start_sample: usize) -> [f32; 128] {
        let mut out_buf: [f32; 128] = [0.0; 128];

        for i in 0..128 {
            let buf_out = self.buffer[self.buffer_idx];

            self.buffer[self.buffer_idx] =
                (self.dampening_filter.process_sample(buf_out) * self.feedback) + block[i];

            out_buf[i] = self.buffer[self.buffer_idx];

            // increment delay idx
            self.buffer_idx += 1;
            if self.buffer_idx >= self.max_buffer_idx {
                self.buffer_idx = 0;
            }
        }

        out_buf
    }
}

pub struct StereoDelay {
    delay_l: MonoDelay,
    delay_r: MonoDelay,
}

impl StereoDelay {
    pub fn with_max_capacity_sec(capacity_sec: f32, sr: f32) -> Self {
        StereoDelay {
            delay_l: MonoDelay::with_max_capacity_sec(capacity_sec, sr),
            delay_r: MonoDelay::with_max_capacity_sec(capacity_sec, sr),
        }
    }

    pub fn set_parameter(&mut self, par: SynthParameter, val: f32) {
        self.delay_l.set_parameter(par, val);
        self.delay_r.set_parameter(par, val);
    }

    pub fn process(&mut self, block: [[f32; 128]; 2]) -> [[f32; 128]; 2] {
        let mut out_buf = [[0.0; 128]; 2];

        out_buf[0] = self.delay_l.process_block(block[0], 0);
        out_buf[1] = self.delay_r.process_block(block[1], 0);

        out_buf
    }
}
