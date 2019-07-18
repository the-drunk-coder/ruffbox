pub mod sampler;
pub mod oscillators;

pub enum SourceState {    
    Fresh,
    Finished
}

/// a collection of common parameters
#[allow(dead_code)]
pub enum SourceParameter {
    Attack,    
    Decay,
    Duration,    
    PitchFrequency,
    PitchNote,
    HighpassCutoffFrequency,
    HighpassQFactor,
    Level,
    LowpassCutoffFrequency,
    LowpassQFactor,
    LowpassFilterDistortion,
    PeakFrequency,
    PeakGain,
    PeakQFactor,
    Pulsewidth,
    PlaybackRate,
    PlaybackStart,
    PlaybackLoop,
    Release,
    ReverbMix,
    SampleBufferNumber,
    Samplerate,
    StereoPosition,
    Sustain,
}

pub enum SourceType {
    Sampler,
    SinOsc,
}

pub trait Source {
    fn set_parameter(&mut self, par: SourceParameter, value: f32);
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(&mut self, start_sample: usize) -> [f32; 128];
}

pub trait Effect {
    fn set_parameter(&mut self, par: SourceParameter, value: f32);    
    fn process_block(&mut self, block: [f32; 128]) -> [f32; 128];
}
