use crate::ruffbox::synth::Effect;
use crate::ruffbox::synth::SynthParameter;

/**
 * Three-pole, 18dB/octave filter with tanh distortion
 * Adapted from CSound via Soundpipe
 *
 * My all-time favourite lowpass :D
 */
pub struct Lpf18<const BUFSIZE:usize> {
    // user parameters
    cutoff: f32,
    res: f32,
    dist: f32,

    // internal parameters    
    ay1: f32,
    ay2: f32,
    ay11: f32,
    ay31: f32,
    ax1: f32,
    kfcn: f32,
    kp: f32,
    kp1: f32,
    kp1h: f32,
    kres: f32,
    value: f32,
    aout: f32,
    lastin: f32,
    samplerate: f32,
}

impl <const BUFSIZE:usize> Lpf18<BUFSIZE> {
    pub fn new(freq: f32, res: f32, dist: f32, sr: f32) -> Self {
        let kfcn = 2.0 * freq * (1.0 / sr);
        let kp = ((-2.7528 * kfcn + 3.0429) * kfcn + 1.718) * kfcn - 0.9984;
        let kp1 = kp + 1.0;
        let kp1h = 0.5 * kp1;
        let kres = res * (((-2.7079 * kp1 + 10.963) * kp1 - 14.934) * kp1 + 8.4974);
        let value = 1.0 + (dist * (1.5 + 2.0 * res * (1.0 - kfcn)));
        Lpf18 {
            cutoff: freq,
            res: res,
            dist: dist,
            ay1: 0.0,
            ay2: 0.0,
            ax1: 0.0,
            ay11: 0.0,
            ay31: 0.0,            
            kfcn: kfcn,
            kp: kp,
            kp1: kp1,
            kp1h: kp1h,
            kres: kres,
            value: value,
            aout: 0.0,
            lastin: 0.0,
            samplerate: sr,
        }
    }

    pub fn process_sample(&mut self, sample: f32) -> f32 {
        self.ax1  = self.lastin;
        self.ay11 = self.ay1;
        self.ay31 = self.ay2;
        
        self.lastin = sample - (self.kres * self.aout).tanh();
        self.ay1 = self.kp1h * (self.lastin + self.ax1) - self.kp * self.ay1;
        self.ay2 = self.kp1h * (self.ay1 + self.ay11) - self.kp * self.ay2;
        self.aout = self.kp1h * (self.ay2 + self.ay31) - self.kp * self.aout;
        
        (self.aout * self.value).tanh()           
    }
}

impl <const BUFSIZE:usize> Effect<BUFSIZE> for Lpf18<BUFSIZE> {
    // some parameter limits might be nice ... 
    fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        match par {
            SynthParameter::LowpassCutoffFrequency => self.cutoff = value, 
            SynthParameter::LowpassQFactor => self.res = value,
            SynthParameter::LowpassFilterDistortion => self.dist = value,
            _ => (),
        };

        self.kfcn = 2.0 * self.cutoff * (1.0 / self.samplerate);
        self.kp = ((-2.7528 * self.kfcn + 3.0429) * self.kfcn + 1.718) * self.kfcn - 0.9984;
        self.kp1 = self.kp + 1.0;
        self.kp1h = 0.5 * self.kp1;
        self.kres = self.res * (((-2.7079 * self.kp1 + 10.963) * self.kp1 - 14.934) * self.kp1 + 8.4974);
        self.value = 1.0 + (self.dist * (1.5 + 2.0 * self.res * (1.0 - self.kfcn)));
    }
    
    fn finish(&mut self) {} // this effect is stateless
    fn is_finished(&self) -> bool { false } // it's never finished ..

    // start sample isn't really needed either ... 
    fn process_block(&mut self, block: [f32; BUFSIZE], _start_sample: usize) -> [f32; BUFSIZE] {
        let mut out_buf: [f32; BUFSIZE] = [0.0; BUFSIZE];

        for i in 0..BUFSIZE {

            self.ax1  = self.lastin;
            self.ay11 = self.ay1;
            self.ay31 = self.ay2;

            self.lastin = block[i] - (self.kres * self.aout).tanh();
            self.ay1 = self.kp1h * (self.lastin + self.ax1) - self.kp * self.ay1;
            self.ay2 = self.kp1h * (self.ay1 + self.ay11) - self.kp * self.ay2;
            self.aout = self.kp1h * (self.ay2 + self.ay31) - self.kp * self.aout;

            out_buf[i] = (self.aout * self.value).tanh();            
        }

        out_buf
    }
}
