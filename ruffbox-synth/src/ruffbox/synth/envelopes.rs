use crate::ruffbox::synth::Effect;
use crate::ruffbox::synth::SourceParameter;
use crate::ruffbox::synth::SourceState;



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
    max_lvl: f32,
    atk_lvl_increment: f32,
    rel_lvl_decrement: f32,
    state: SourceState,
}

impl ASREnvelope {
    pub fn new(samplerate: f32, lvl: f32, atk: f32, sus: f32, rel: f32) -> Self {
        let atk_samples = (samplerate * atk).round();
        let sus_samples = atk_samples + (samplerate * sus).round();
        let rel_samples = sus_samples + (samplerate * rel).round();

        println!("atk sam: {} sus sam: {} rel sam: {}", atk_samples.round(),sus_samples.round(),rel_samples.round(), );
        
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
            max_lvl: lvl,
            atk_lvl_increment: lvl / atk_samples,
            rel_lvl_decrement: lvl / (rel_samples - sus_samples),
            state: SourceState::Fresh,
        }
    }    
}

impl Effect for ASREnvelope {
    fn finish(&mut self) {
        self.state = SourceState::Finished;
    }

    fn is_finished(&self) -> bool {
        match self.state {
            SourceState::Finished => true,
            _ => false,
        }
    }

    fn set_parameter(&mut self, par: SourceParameter, value: f32) {
        match par {
            SourceParameter::Attack => {
                self.atk_samples = (self.samplerate / value).round() as usize;
                self.sus_samples = self.atk_samples + (self.samplerate / self.sus).round() as usize;
                self.rel_samples = self.sus_samples + (self.samplerate / self.rel).round() as usize;                
            },
            SourceParameter::Sustain => {
                self.sus_samples = self.atk_samples + (self.samplerate / value).round() as usize;
                self.rel_samples = self.sus_samples + (self.samplerate / self.rel).round() as usize;  
            },
            SourceParameter::Release => {
                self.rel_samples = self.sus_samples + (self.samplerate / value).round() as usize;
            },
            SourceParameter::Samplerate => {
                self.atk_samples = (self.samplerate / self.atk).round() as usize;
                self.sus_samples = self.atk_samples + (self.samplerate / self.sus).round() as usize;
                self.rel_samples = self.sus_samples + (self.samplerate / self.rel).round() as usize;
            }
            _ => ()
        };
        
        self.atk_lvl_increment = self.max_lvl / self.atk_samples as f32;
        self.rel_lvl_decrement = self.max_lvl / (self.rel_samples - self.sus_samples) as f32;      
    }
    
    fn process_block(&mut self, block: [f32; 128], start_sample: usize) -> [f32; 128] {        
        let mut out: [f32; 128] = [0.0; 128];

        for i in start_sample..128 {
            out[i] = block[i] * self.lvl;

            self.sample_count += 1;

            if self.sample_count <= self.atk_samples {
                self.lvl += self.atk_lvl_increment;
            } else if self.sample_count >= self.sus_samples && self.sample_count <= self.rel_samples {
                self.lvl -= self.rel_lvl_decrement;
            } else if self.sample_count > self.rel_samples {
                self.lvl = 0.0;
                self.finish();
            }            
        }

        out
    }
}

// TEST TEST TEST 
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_asr_envelope() {
        let mut test_block: [f32; 128] = [1.0; 128];

        // half a block attack, one block sustain, half a block release ... 2 blocks total .
        let mut env = ASREnvelope::new(44100.0, 0.5, 0.0014512, 0.0029024, 0.0014512);

        let mut out_1: [f32; 128] = env.process_block(test_block, 0);
        let mut out_2: [f32; 128] = env.process_block(test_block, 0);

        // comparison
        let mut comp_block_1: [f32; 128] = [0.0; 128];
        let mut comp_block_2: [f32; 128] = [0.0; 128];


        let mut gain = 0.0;
        let mut gain_inc_dec = 0.5 / 64.0;
            
        // fill comp blocks
        for i in 0..64 {
            comp_block_1[i] = test_block[i] * gain;
            gain += gain_inc_dec;
        }

        for i in 0..64 {
            comp_block_1[64 + i] = test_block[64 + i] * gain;
        }

        for i in 0..64 {
            comp_block_2[i] = test_block[i] * gain;
        }

        for i in 0..64 {
            gain -= gain_inc_dec;
            comp_block_2[64 + i] = test_block[64 + i] * gain;            
        }

        for i in 0..128 {
            //println!("{} {}", out_1[i], comp_block_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[i], comp_block_1[i], 0.00001);
        }

        for i in 0..128 {
            //println!("{} {}", out_2[i], comp_block_2[i]);
            assert_approx_eq::assert_approx_eq!(out_2[i], comp_block_2[i], 0.00001);
        }
        
    }

}
