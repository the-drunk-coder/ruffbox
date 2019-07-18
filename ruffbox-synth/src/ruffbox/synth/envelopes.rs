use crate::ruffbox::synth::Effect;
use crate::ruffbox::synth::SourceParameter;

/// simple attack-sustain-release envelope
pub struct ASREnvelope {
    samplerate: f32,
    atk: f32,
    sus: f32,
    rel: f32,
    atk_samples: usize,
    sus_samples: usize,
    rel_samples: usize,
    sample_count: usize,
    lvl: f32,
    atk_lvl_increment: f32,
    rel_lvl_decrement: f32,    
}

impl ASREnvelope {
    pub fn new(samplerate: f32, gain: f32, atk: f32, sus: f32, rel: f32) -> Self {
        let atk_samples = samplerate / atk;
        let sus_samples = atk_samples + samplerate / sus;
        let rel_samples = sus_samples + samplerate / rel;
        
        ASREnvelope {
            samplerate: samplerate,
            atk: atk,
            sus: sus,
            rel: rel,
            atk_samples: atk_samples as usize,
            sus_samples: sus_samples as usize,
            rel_samples: rel_samples as usize,
            sample_count: 0,
            lvl: 0.0,
            atk_lvl_increment: gain / atk_samples,
            rel_lvl_decrement: gain / (rel_samples - sus_samples),            
        }
    }    
}

impl Effect for ASREnvelope {

    fn set_parameter(&mut self, par: SourceParameter, value: f32) {
        match par {
            SourceParameter::Attack => {
                self.atk_samples = (self.samplerate / value) as usize;
                self.sus_samples = self.atk_samples + (self.samplerate / self.sus) as usize;
                self.rel_samples = self.sus_samples + (self.samplerate / self.rel) as usize;
            },
            SourceParameter::Sustain => {
                self.sus_samples = self.atk_samples + (self.samplerate / value) as usize;
                self.rel_samples = self.sus_samples + (self.samplerate / self.rel) as usize;  
            },
            SourceParameter::Release => {
                self.rel_samples = self.sus_samples + (self.samplerate / value) as usize;
            },
            SourceParameter::Samplerate => {
                self.atk_samples = (self.samplerate / self.sus) as usize;
                self.sus_samples = self.atk_samples + (self.samplerate / self.sus) as usize;
                self.rel_samples = self.sus_samples + (self.samplerate / self.rel) as usize;
            }
            _ => ()
        };
    }
    
    fn process_block(&mut self, block: [f32; 128], start_sample: usize) -> [f32; 128] {        
        let mut out: [f32; 128] = [0.0; 128];

        for i in start_sample..128 {
            out[i] = block[i] * self.lvl;

            self.sample_count += 1;

            if self.sample_count < self.atk_samples {
                self.lvl += self.atk_lvl_increment;
            } else if self.sample_count >= self.sus_samples && self.sample_count <= self.rel_samples {
                self.lvl -= self.rel_lvl_decrement;
            } else {
                self.lvl = 0.0;
            }            
        }

        out
    }
}
