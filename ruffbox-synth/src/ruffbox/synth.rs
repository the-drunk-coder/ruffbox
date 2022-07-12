pub mod delay;
pub mod envelopes;
pub mod filters;
pub mod freeverb;
pub mod oscillators;
pub mod routing;
pub mod sampler;
pub mod synths;

pub enum SynthState {
    Fresh,
    Finished,
}

/// a collection of common parameters
#[allow(dead_code)]
#[derive(Clone, Copy)]
#[repr(C)]
pub enum SynthParameter {
    Attack,                  // 0
    Decay,                   // 1
    DelayDampeningFrequency, // 2
    DelayFeedback,           // 3
    DelayMix,                // 4
    DelayTime,               // 5
    Duration,                // 6
    PitchFrequency,          // 7
    PitchNote,               // 8
    HighpassCutoffFrequency, // 9
    HighpassQFactor,         // 10
    Level,                   // 11
    LowpassCutoffFrequency,  // 12
    LowpassQFactor,          // 13
    LowpassFilterDistortion, // 14
    PeakFrequency,           // 15
    PeakGain,                // 16
    PeakQFactor,             // 17
    Pulsewidth,              // 18
    PlaybackRate,            // 19
    PlaybackStart,           // 20
    PlaybackLoop,            // 21
    Release,                 // 22
    ReverbDampening,         // 23
    ReverbMix,               // 24
    ReverbRoomsize,          // 25
    SampleBufferNumber,      // 26
    Samplerate,              // 27
    StereoPosition,          // 28
    Sustain,                 // 29
}

#[repr(C)]
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
