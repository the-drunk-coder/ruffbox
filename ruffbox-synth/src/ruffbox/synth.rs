pub mod sampler;
pub mod envelopes;
pub mod routing;
pub mod oscillators;
pub mod synths;
pub mod filters;
pub mod freeverb;
pub mod delay;


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
    DelayDampeningFrequency,
    DelayFeedback,
    DelayMix,
    DelayTime,
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
    ReverbDampening,
    ReverbMix,
    ReverbRoomsize,
    SampleBufferNumber,
    Samplerate,
    StereoPosition,
    Sustain,
}

pub enum SourceType {
    Sampler,
    SineOsc,
    SineSynth,
    LFSawSynth,
    LFSquareSynth,
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

pub trait StereoSynth {
    fn set_parameter(&mut self, par: SynthParameter, value: f32);
    fn finish(&mut self);
    fn is_finished(&self) -> bool;
    fn get_next_block(&mut self, start_sample: usize) -> [[f32; 128]; 2];
    fn reverb_level(&self) -> f32;
    fn delay_level(&self) -> f32;
}
