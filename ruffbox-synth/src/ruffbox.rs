pub mod synth;

// crossbeam for the event queue
use crossbeam::channel::Sender;
use crossbeam::channel::Receiver;
use crossbeam::atomic::AtomicCell;

use std::collections::HashMap;

use std::cmp::Ordering;
use std::sync::Arc;

use crate::ruffbox::synth::StereoSynth;
use crate::ruffbox::synth::SynthParameter;
use crate::ruffbox::synth::SourceType;
use crate::ruffbox::synth::freeverb::StereoFreeverb;
use crate::ruffbox::synth::delay::StereoDelay;
use crate::ruffbox::synth::synths::*;

/// timed event, to be created in the trigger method, then 
/// sent to the event queue to be either dispatched directly
/// or pushed to the pending queue ...
struct ScheduledEvent<const BUFSIZE:usize> {
    timestamp: f64,
    source: Box<dyn StereoSynth<BUFSIZE> + Send>,
}

impl <const BUFSIZE:usize> Ord for ScheduledEvent <BUFSIZE> {
    /// ScheduledEvent implements Ord so the pending events queue
    /// can be ordered by the timestamps ...
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.partial_cmp(&other.timestamp).unwrap()
    }
}

impl <const BUFSIZE:usize> PartialOrd for ScheduledEvent <BUFSIZE> {
    /// ScheduledEvent implements PartialOrd so the pending events queue
    /// can be ordered by the timestamps ...
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl <const BUFSIZE:usize> PartialEq for ScheduledEvent <BUFSIZE> {
    /// ScheduledEvent implements PartialEq so the pending events queue
    /// can be ordered by the timestamps ...
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl <const BUFSIZE:usize> Eq for ScheduledEvent <BUFSIZE> {}

// constructor implementation
impl <const BUFSIZE:usize> ScheduledEvent <BUFSIZE> {
    pub fn new(ts: f64, src: Box<dyn StereoSynth<BUFSIZE> + Send>) -> Self {
        ScheduledEvent {
            timestamp: ts,
            source: src,
        }
    }

    pub fn set_parameter(&mut self, par: SynthParameter, value: f32) {
        self.source.set_parameter(par, value);
    }
}

/// the main synth instance
pub struct Ruffbox<const BUFSIZE:usize> {
    running_instances: Vec<Box<dyn StereoSynth<BUFSIZE> + Send>>,
    pending_events: Vec<ScheduledEvent<BUFSIZE>>,
    buffers: Vec<Arc<Vec<f32>>>,
    prepared_instance_map: HashMap<usize, ScheduledEvent<BUFSIZE>>,
    instance_counter: AtomicCell<usize>,
    new_instances_q_send: crossbeam::channel::Sender<ScheduledEvent<BUFSIZE>>,
    new_instances_q_rec: crossbeam::channel::Receiver<ScheduledEvent<BUFSIZE>>,
    block_duration: f64,
    sec_per_sample: f64,
    now: f64,
    master_reverb: StereoFreeverb<BUFSIZE>,
    master_delay: StereoDelay<BUFSIZE>,
}

impl <const BUFSIZE: usize> Ruffbox<BUFSIZE> {
    pub fn new() -> Ruffbox<BUFSIZE> {
        let (tx, rx): (Sender<ScheduledEvent<BUFSIZE>>, Receiver<ScheduledEvent<BUFSIZE>>) = crossbeam::channel::bounded(1000);

        // tweak some reverb values ... 
        let mut rev = StereoFreeverb::new();
        rev.set_roomsize(0.65);
        rev.set_damp(0.43);
        rev.set_wet(1.0);
	
        let del = StereoDelay::with_max_capacity_sec(2.0, 44100.0);
        
        Ruffbox {            
            running_instances: Vec::with_capacity(600),
            pending_events: Vec::with_capacity(600),
            buffers: Vec::with_capacity(20),
            prepared_instance_map: HashMap::with_capacity(600),
            instance_counter: AtomicCell::new(0),
            new_instances_q_send: tx,
            new_instances_q_rec: rx,
            // timing stuff
            block_duration: BUFSIZE as f64 / 44100.0,
            sec_per_sample: 1.0 / 44100.0,
            now: 0.0,
            master_reverb: rev,
            master_delay: del,
        }
    }
           
    pub fn process(&mut self, stream_time: f64) -> [[f32; BUFSIZE]; 2] {        
        let mut out_buf: [[f32; BUFSIZE]; 2] = [[0.0; BUFSIZE]; 2];

        let mut master_delay_in: [[f32; BUFSIZE]; 2] = [[0.0; BUFSIZE]; 2];        
        let mut master_reverb_in: [f32; BUFSIZE] = [0.0; BUFSIZE];

        self.now = stream_time;
        
        // remove finished instances ...
        self.running_instances.retain( |instance| !&instance.is_finished());

        // add new instances
        for new_event in self.new_instances_q_rec.try_iter() {
            if new_event.timestamp == 0.0 || new_event.timestamp == self.now {
                self.running_instances.push(new_event.source);                
            } else if new_event.timestamp < self.now { // late events 
                self.running_instances.push(new_event.source);
                // how to send out a late message ??
                // some lock-free message queue to a printer thread or something .... 
                // println!("late");
            } else {
                self.pending_events.push(new_event);
            }            
        }

        // handle already running instances
        for running_inst in self.running_instances.iter_mut() {
            let block = running_inst.get_next_block(0);
            for s in 0..BUFSIZE {
                out_buf[0][s] += block[0][s];
                out_buf[1][s] += block[1][s];

                master_reverb_in[s] += (block[0][s] + block[1][s]) * running_inst.reverb_level();
                master_delay_in[0][s] += block[0][s] * running_inst.delay_level();
                master_delay_in[1][s] += block[1][s] * running_inst.delay_level();
            }
        }
        
        // sort new events by timestamp, order of already sorted elements doesn't matter
        self.pending_events.sort_unstable_by(|a, b| b.cmp(a));
        let block_end = stream_time + self.block_duration;
        
        // fetch event if it belongs to this block, if any ...
        while !self.pending_events.is_empty() && self.pending_events.last().unwrap().timestamp < block_end {

            let mut current_event = self.pending_events.pop().unwrap();

            // calculate precise timing
            let sample_offset = (current_event.timestamp - stream_time) / self.sec_per_sample;           

            let block = current_event.source.get_next_block(sample_offset.round() as usize);

            for s in 0..BUFSIZE {
                out_buf[0][s] += block[0][s];
                out_buf[1][s] += block[1][s];
                
                master_reverb_in[s] += (block[0][s] + block[1][s]) * current_event.source.reverb_level();
                master_delay_in[0][s] += block[0][s] * current_event.source.delay_level();
                master_delay_in[1][s] += block[1][s] * current_event.source.delay_level();
            }
            
            // if length of sample event is longer than the rest of the block,
            // add to running instances
            if !current_event.source.is_finished() {
                self.running_instances.push(current_event.source);
            }
        }

        let reverb_out = self.master_reverb.process(master_reverb_in);
        let delay_out = self.master_delay.process(master_delay_in);
        
        for s in 0..BUFSIZE {
            out_buf[0][s] += reverb_out[0][s] + delay_out[0][s];
            out_buf[1][s] += reverb_out[1][s] + delay_out[1][s];
        }
                              
        out_buf
    }

    /// prepare a sound source instance, return instance id 
    pub fn prepare_instance(&mut self, src_type: SourceType, timestamp: f64, sample_buf: usize) -> usize {
        let instance_id = self.instance_counter.fetch_add(1);

        let scheduled_event = match src_type {
            SourceType::SineOsc => ScheduledEvent::new(timestamp, Box::new(SineSynth::new(44100.0))),
            SourceType::SineSynth => ScheduledEvent::new(timestamp, Box::new(SineSynth::new(44100.0))),
            SourceType::Sampler => ScheduledEvent::new(timestamp, Box::new(StereoSampler::with_buffer_ref(&self.buffers[sample_buf], 44100.0))),
            SourceType::LFSawSynth => ScheduledEvent::new(timestamp, Box::new(LFSawSynth::new(44100.0))),
            SourceType::LFSquareSynth => ScheduledEvent::new(timestamp, Box::new(LFSquareSynth::new(44100.0))),
        };

        self.prepared_instance_map.insert(instance_id, scheduled_event);
        
        instance_id
    }

    pub fn set_instance_parameter(&mut self, instance_id: usize, par: SynthParameter, val: f32) {
        self.prepared_instance_map.get_mut(&instance_id).unwrap().set_parameter(par, val);
    }

    pub fn set_master_parameter(&mut self, par: SynthParameter, val: f32) {
        self.master_reverb.set_parameter(par, val);
        self.master_delay.set_parameter(par, val);
    }
    
    /// triggers a synth for buffer reference or a synth
    pub fn trigger(&mut self, instance_id: usize) {
        // add check if it actually exists !
        let scheduled_event = self.prepared_instance_map.remove(&instance_id).unwrap();        
        self.new_instances_q_send.send(scheduled_event).unwrap();
    }

    /// loads a sample and returns the assigned buffer number
    pub fn load_sample(&mut self, samples:&[f32]) -> usize {
        self.buffers.push(Arc::new(samples.to_vec()));
        self.buffers.len() - 1
    }     
}


// TEST TEST TEST 
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use std::f32::consts::PI;
    
    #[test]
    fn test_sine_synth_at_block_start() {
        let mut ruff = Ruffbox::<128>::new();

        let inst = ruff.prepare_instance(SourceType::SineSynth, 0.0, 0);
        ruff.set_instance_parameter(inst, SynthParameter::PitchFrequency, 440.0);
        ruff.set_instance_parameter(inst, SynthParameter::StereoPosition, -1.0);
        ruff.set_instance_parameter(inst, SynthParameter::Level, 1.0);
        ruff.set_instance_parameter(inst, SynthParameter::Attack, 0.0);
        ruff.set_instance_parameter(inst, SynthParameter::Sustain, 1.0);
        ruff.set_instance_parameter(inst, SynthParameter::Release, 0.0);
        
        ruff.trigger(inst);
        
        let out_1 = ruff.process(0.0);
        let mut comp_1 = [0.0; 128];

        for i in 0..128 {
            comp_1[i] = (2.0 * PI * 440.0 * (i as f32 * (1.0 / 44100.0))).sin()
        }
        
        for i in 0..128 {
            //println!("{} {} {}; ", i, out_1[0][i], comp_1[i]);
            assert_approx_eq::assert_approx_eq!(out_1[0][i], comp_1[i], 0.00001);
        }
    }
    
    #[test]
    fn test_basic_playback() {
        
        let mut ruff = Ruffbox::<128>::new();

        // first point and last two points are for eventual interpolation
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);
        
        ruff.process(0.0);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum2);

        // pan to left, neutralize
        ruff.set_instance_parameter(inst_1, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);
	
	ruff.set_instance_parameter(inst_2, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);
		               
        ruff.trigger(inst_1);
        ruff.trigger(inst_2);

        let out_buf = ruff.process(0.0);
        
        for i in 0..9 {
            println!("{} {} ", out_buf[0][i], sample1[i + 1] + sample2[i + 1]);
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 1] + sample2[i + 1], 0.03);
        }        
    }

    #[test]
    fn reverb_smoke_test() {
        
        let mut ruff = Ruffbox::<128>::new();

        // first point and last two points are for eventual interpolation
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
                
        let bnum1 = ruff.load_sample(&sample1);
                
        ruff.process(0.0);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum1);        

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::StereoPosition, -1.0);
        ruff.set_instance_parameter(inst_1, SynthParameter::ReverbMix, 1.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);
                       
        ruff.trigger(inst_1);
        
        let out_buf = ruff.process(0.0);
        
        for i in 0..9 {
            println!("{} {} ", out_buf[0][i], sample1[i + 1]);
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 1], 0.03);
        }        
    }

    #[test]
    fn test_scheduled_playback() {
        let mut ruff = Ruffbox::<128>::new();

        // block duration in seconds
        let block_duration = 0.00290249433;

        // first point and last two points are for eventual interpolation
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum2);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.set_instance_parameter(inst_2, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);
        
        ruff.trigger(inst_1);
        ruff.trigger(inst_2);
        
        let mut stream_time = 0.0;
        // calculate a few blocks
        for _ in 0..100 {
            ruff.process(stream_time);
            stream_time += block_duration;
        }
                        
        let out_buf = ruff.process(stream_time);
                       
        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample1[i + 1] + sample2[i + 1], 0.03);
        }        
    }

    #[test]
    fn test_overlap_playback() {
        let mut ruff = Ruffbox::<128>::new();

        // block duration in seconds
        let block_duration = 0.00290249433;
        let sec_per_sample = 0.00002267573;
        
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.291 + (4.0 * sec_per_sample), bnum2);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.set_instance_parameter(inst_2, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);
        
        ruff.trigger(inst_1);
        ruff.trigger(inst_2);
        
        let mut stream_time = 0.0;
        
        // calculate a few blocks
        for _ in 0..100 {
            ruff.process(stream_time);
            stream_time += block_duration;
        }
         
        let out_buf = ruff.process(stream_time);

        // offsets to account for interpolation
        for i in 0..4 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample1[i + 1], 0.03);
        }

        for i in 0..5 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][37 + i], sample1[i + 4 + 1] + sample2[i + 1], 0.03);
        }

        for i in 0..4 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][42 + i], sample2[i + 5 + 1], 0.03);
        }
    }

    #[test]
    fn test_disjunct_playback() {
        let mut ruff = Ruffbox::<128>::new();

        // block duration in seconds
        let block_duration = 0.00290249433;
        
        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
        let sample2 = [0.0, 0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0, 0.0, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        // schedule two samples ahead, so they should  occur in different blocks
        // first sample should appear in block 100
        
        // second sample should appear ten blocks later
        let second_sample_timestamp = 0.291 + (10.0 * block_duration);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, second_sample_timestamp, bnum2);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.set_instance_parameter(inst_2, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_2, SynthParameter::Sustain, 9.0 / 44100.0);
	
        ruff.trigger(inst_1);
        ruff.trigger(inst_2);
        
        let mut stream_time = 0.0;
        
        // calculate a few blocks
        for _ in 0..100 {
            ruff.process(stream_time);
            stream_time += block_duration;
        }
         
        let out_buf = ruff.process(stream_time);
        stream_time += block_duration;
                
        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample1[i + 1], 0.03);
        }
        
        // calculate a few blocks more
        for _ in 0..9 {
            ruff.process(stream_time);            
            stream_time += block_duration;
        }
        
        let out_buf = ruff.process(stream_time);
        
        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][33 + i], sample2[i + 1], 0.03);
        }
    }

    #[test]
    fn test_late_playback() {
        
        let mut ruff = Ruffbox::<128>::new();

        let sample1 = [0.0, 0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0, 0.0, 0.0];
                
        let bnum1 = ruff.load_sample(&sample1);        
        
        ruff.process(0.0);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.1, bnum1);

        // pan to left
        ruff.set_instance_parameter(inst_1, SynthParameter::StereoPosition, -1.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassCutoffFrequency, 22050.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassFilterDistortion, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::LowpassQFactor, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Attack, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Release, 0.0);
	ruff.set_instance_parameter(inst_1, SynthParameter::Sustain, 9.0 / 44100.0);

        ruff.trigger(inst_1);

        // process after the instance's trigger time
        let out_buf = ruff.process(0.101);
        
        for i in 0..9 {
            assert_approx_eq::assert_approx_eq!(out_buf[0][i], sample1[i + 1], 0.03);
        }        
    }
}
