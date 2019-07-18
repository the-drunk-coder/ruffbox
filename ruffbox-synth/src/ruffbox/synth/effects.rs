use crate::ruffbox::synth::Effect;

/// simple attack-sustain-release envelope
pub struct ASREnvelope {
    
}

impl ASREnvelope {
    pub fn new(atk: f64, sus: f64, rel: f64) {
        
    }
    
}

impl Effect for ASREnvelope {
    fn set_parameter(&mut self, par: SourceParameter, value: f32) {
        
    }

    fn process_block(&mut self, block: [f32; 128]) -> [f32; 128] {
        let mut out = [f32, 128];

        out
    }
}
