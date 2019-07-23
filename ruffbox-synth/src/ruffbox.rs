pub mod synth;

// crossbeam for the event queue
use crossbeam::channel::Sender;
use crossbeam::channel::Receiver;
use crossbeam::atomic::AtomicCell;

use std::collections::HashMap;

use std::cmp::Ordering;
use std::sync::Arc;

use crate::ruffbox::synth::Source;
use crate::ruffbox::synth::SynthParameter;
use crate::ruffbox::synth::SourceType;
use crate::ruffbox::synth::sampler::Sampler;
use crate::ruffbox::synth::oscillators::SineOsc;
use crate::ruffbox::synth::synths::*;

/// timed event, to be created in the trigger method, then 
/// sent to the event queue to be either dispatched directly
/// or pushed to the pending queue ...
struct ScheduledEvent {
    timestamp: f64,
    source: Box<dyn Source + Send>,
}

impl Ord for ScheduledEvent {
    /// ScheduledEvent implements Ord so the pending events queue
    /// can be ordered by the timestamps ...
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.partial_cmp(&other.timestamp).unwrap()
    }
}

impl PartialOrd for ScheduledEvent {
    /// ScheduledEvent implements PartialOrd so the pending events queue
    /// can be ordered by the timestamps ...
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledEvent {
    /// ScheduledEvent implements PartialEq so the pending events queue
    /// can be ordered by the timestamps ...
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Eq for ScheduledEvent {}

// constructor implementation
impl ScheduledEvent {
    pub fn new(ts: f64, src: Box<dyn Source + Send>) -> Self {
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
pub struct Ruffbox {
    running_instances: Vec<Box<dyn Source + Send>>,
    pending_events: Vec<ScheduledEvent>,
    buffers: Vec<Arc<Vec<f32>>>,
    prepared_instance_map: HashMap<usize, ScheduledEvent>,
    instance_counter: AtomicCell<usize>,
    new_instances_q_send: crossbeam::channel::Sender<ScheduledEvent>,
    new_instances_q_rec: crossbeam::channel::Receiver<ScheduledEvent>,
    block_duration: f64,
    sec_per_sample: f64,
    now: f64,
}

impl Ruffbox {
    pub fn new() -> Ruffbox {
        let (tx, rx): (Sender<ScheduledEvent>, Receiver<ScheduledEvent>) = crossbeam::channel::bounded(1000);
        Ruffbox {            
            running_instances: Vec::with_capacity(600),
            pending_events: Vec::with_capacity(600),
            buffers: Vec::with_capacity(20),
            prepared_instance_map: HashMap::with_capacity(600),
            instance_counter: AtomicCell::new(0),
            new_instances_q_send: tx,
            new_instances_q_rec: rx,
            // timing stuff
            block_duration: 128.0 / 44100.0,
            sec_per_sample: 1.0 / 44100.0,
            now: 0.0,
        }
    }
           
    pub fn process(&mut self, stream_time: f64) -> [f32; 128] {        
        let mut out_buf: [f32; 128] = [0.0; 128];
        self.now = stream_time;
        
        // remove finished instances ...
        self.running_instances.retain( |instance| !&instance.is_finished());

        // add new instances
        for new_event in self.new_instances_q_rec.try_iter() {
            if new_event.timestamp == 0.0 {
                self.running_instances.push(new_event.source);                
            } else if new_event.timestamp <= self.now { // late events 
                self.running_instances.push(new_event.source);
                // how to send out a late message ??
            } else  {
                self.pending_events.push(new_event);
            }            
        }

        // sort new events by timestamp, order of already sorted elements doesn't matter
        self.pending_events.sort_unstable_by(|a, b| b.cmp(a));
        let block_end = stream_time + self.block_duration;
        
        // fetch event if it belongs to this block, if any ...
        while !self.pending_events.is_empty() && self.pending_events.last().unwrap().timestamp <= block_end {

            let mut current_event = self.pending_events.pop().unwrap();

            // calculate precise timing
            let sample_offset = (current_event.timestamp - stream_time) / self.sec_per_sample;           
            let block = current_event.source.get_next_block(sample_offset.round() as usize);
            for s in 0..128 {
                out_buf[s] += block[s];
            }

            // if length of sample event is longer than the rest of the block,
            // add to running instances
            if !current_event.source.is_finished() {
                self.running_instances.push(current_event.source);
            }
        }
        
        // handle already running instances
        for running_inst in self.running_instances.iter_mut() {
            let block = running_inst.get_next_block(0);
            for s in 0..128 {
                out_buf[s] += block[s];
            }
        }
        
        out_buf
    }

    /// prepare a sound source instance, return instance id 
    pub fn prepare_instance(&mut self, src_type: SourceType, timestamp: f64, sample_buf: usize) -> usize {
        let instance_id = self.instance_counter.fetch_add(1);

        let scheduled_event = match src_type {
            SourceType::SineOsc => ScheduledEvent::new(timestamp, Box::new(SineOsc::new(440.0, 0.2, 0.3, 44100.0))),
            SourceType::SineSynth => ScheduledEvent::new(timestamp, Box::new(SineSynth::new(44100.0))),
            SourceType::Sampler => ScheduledEvent::new(timestamp, Box::new(Sampler::with_buffer_ref(&self.buffers[sample_buf]))),
            SourceType::LFSawSynth => ScheduledEvent::new(timestamp, Box::new(LFSawSynth::new(44100.0))),
        };

        self.prepared_instance_map.insert(instance_id, scheduled_event);
        
        instance_id
    }

    pub fn set_instance_parameter(&mut self, instance_id: usize, par: SynthParameter, val: f32) {
        self.prepared_instance_map.get_mut(&instance_id).unwrap().set_parameter(par, val);
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

    #[test]
    fn test_basic_playback() {
        
        let mut ruff = Ruffbox::new();

        let sample1 = [0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let sample2 = [0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);
        
        ruff.process(0.0);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.0, bnum2);
        
        ruff.trigger(inst_1);
        ruff.trigger(inst_2);

        let out_buf = ruff.process(0.0);
        
        for i in 0..9 {
            assert_eq!(out_buf[i], sample1[i] + sample2[i]);
        }        
    }

    #[test]
    fn test_scheduled_playback() {
        let mut ruff = Ruffbox::new();

        // block duration in seconds
        let block_duration = 0.00290249433;

        let sample1 = [0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let sample2 = [0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum2);
        
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
            assert_eq!(out_buf[33 + i], sample1[i] + sample2[i]);
        }        
    }

    #[test]
    fn test_overlap_playback() {
        let mut ruff = Ruffbox::new();

        // block duration in seconds
        let block_duration = 0.00290249433;
        let sec_per_sample = 0.00002267573;
        
        let sample1 = [0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let sample2 = [0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, 0.291 + (4.0 * sec_per_sample), bnum2);
        
        ruff.trigger(inst_1);
        ruff.trigger(inst_2);
        
        let mut stream_time = 0.0;
        
        // calculate a few blocks
        for _ in 0..100 {
            ruff.process(stream_time);
            stream_time += block_duration;
        }
         
        let out_buf = ruff.process(stream_time);
               
        for i in 0..4 {
            assert_eq!(out_buf[33 + i], sample1[i]);
        }

        for i in 0..5 {
            assert_eq!(out_buf[37 + i], sample1[i + 4] + sample2[i]);
        }

        for i in 0..4 {
            assert_eq!(out_buf[42 + i], sample2[i + 5]);
        }
    }

    #[test]
    fn test_disjunct_playback() {
        let mut ruff = Ruffbox::new();

        // block duration in seconds
        let block_duration = 0.00290249433;
        
        let sample1 = [0.0, 0.1, 0.2, 0.3, 0.4, 0.3, 0.2, 0.1, 0.0];
        let sample2 = [0.0, 0.01, 0.02, 0.03, 0.04, 0.03, 0.02, 0.01, 0.0];
        
        let bnum1 = ruff.load_sample(&sample1);
        let bnum2 = ruff.load_sample(&sample2);

        // schedule two samples ahead, so they should  occur in different blocks
        // first sample should appear in block 100
        
        // second sample should appear ten blocks later
        let second_sample_timestamp = 0.291 + (10.0 * block_duration);

        let inst_1 = ruff.prepare_instance(SourceType::Sampler, 0.291, bnum1);
        let inst_2 = ruff.prepare_instance(SourceType::Sampler, second_sample_timestamp, bnum2);

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
            assert_eq!(out_buf[33 + i], sample1[i]);
        }
        
        // calculate a few blocks more
        for _ in 0..9 {
            ruff.process(stream_time);            
            stream_time += block_duration;
        }
        
        let out_buf = ruff.process(stream_time);
        
        for i in 0..9 {
            assert_eq!(out_buf[33 + i], sample2[i]);
        }
    }
}
