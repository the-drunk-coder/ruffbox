pub mod sampler;
pub mod envelopes;
pub mod oscillators;
pub mod synths;

pub enum SynthState { 
    Fresh,
    Finished
}

/// a collection of common parameters
#[allow(dead_code)]
#[derive(Clone,Copy)]
pub enum SynthParameter {
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
    SineOsc,
    SineSynth,
}

pub trait Source {
    fn set_parameter(&mut self, par: SynthParameter, value: f32);
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(&mut self, start_sample: usize) -> [f32; 128];
}

pub trait Effect {
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn set_parameter(&mut self, par: SynthParameter, value: f32);    
    fn process_block(&mut self, block: [f32; 128], start_sample: usize) -> [f32; 128];
}
