pub mod sampler;

use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;

use std::cmp::Ordering;

use std::sync::Arc;

use crate::ruffbox::sampler::Source;
use crate::ruffbox::sampler::Sampler;

/// timed event, to be created in the trigger method, then 
/// sent to the event queue to be either dispatched directly
/// or pushed to the pending queue ...
struct ScheduledEvent {
    timestamp: f64,
    sampler: Box<Source + Send>,
}

/// ScheduledEvent implements Ord so the pending events queue
/// can be ordered by the timestamps ...
impl Ord for ScheduledEvent {
    fn cmp(&self, other: &Self) -> Ordering {
        self.timestamp.partial_cmp(&other.timestamp).unwrap()
    }
}

impl PartialOrd for ScheduledEvent {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ScheduledEvent {
    fn eq(&self, other: &Self) -> bool {
        self.timestamp == other.timestamp
    }
}

impl Eq for ScheduledEvent {}

/// constructor implementation
impl ScheduledEvent {
    pub fn new(ts: f64, sam: Box<Source + Send>) -> Self {
        ScheduledEvent {
            timestamp: ts,
            sampler: sam,
        }
    }
}

/// the main synth instance
pub struct Ruffbox {
    running_instances: Vec<Box<Source + Send>>,
    pending_events: Vec<ScheduledEvent>,
    buffers: Vec<Arc<Vec<f32>>>,
    new_instances_q_send: Sender<ScheduledEvent>,
    new_instances_q_rec: Receiver<ScheduledEvent>,
    block_duration: f64,
    sec_per_sample: f64,
    now: f64,
}

impl Ruffbox {
    pub fn new() -> Ruffbox {
        let (tx, rx): (Sender<ScheduledEvent>, Receiver<ScheduledEvent>) = mpsc::channel();
        Ruffbox {            
            running_instances: Vec::with_capacity(600),
            pending_events: Vec::with_capacity(600),
            buffers: Vec::with_capacity(20),            
            new_instances_q_send: tx,
            new_instances_q_rec: rx,
            // timing stuff
            block_duration: 0.00290249433,
            sec_per_sample: 0.00002267573,
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
                self.running_instances.push(new_event.sampler);
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
            let block = current_event.sampler.get_next_block(sample_offset as usize);
            for s in 0..128 {
                out_buf[s] += block[s];
            }

            // if length of sample event is longer than the rest of the block,
            // add to running instances
            if !current_event.sampler.is_finished() {
                self.running_instances.push(current_event.sampler);
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

    /// triggers a sampler for buffer reference or a synth
    pub fn trigger(&mut self, temp: usize, timestamp: f64) {
        // add check if it actually exists !
        let scheduled_event = ScheduledEvent::new(timestamp, Box::new(Sampler::with_buffer_ref(&self.buffers[temp])));
        self.new_instances_q_send.send(scheduled_event).unwrap();
    }

    /// loads a sample and returns the assigned buffer number
    pub fn load(&mut self, samples:&[f32]) -> usize {
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
        
        let bnum1 = ruff.load(&sample1);
        let bnum2 = ruff.load(&sample2);
        
        ruff.process(0.0);
        
        ruff.trigger(bnum1, 0.0);
        ruff.trigger(bnum2, 0.0);

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
        
        let bnum1 = ruff.load(&sample1);
        let bnum2 = ruff.load(&sample2);

        // schedule two samples ahead, to the same point in time
        ruff.trigger(bnum1, 0.291);
        ruff.trigger(bnum2, 0.291);
        
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
        
        let bnum1 = ruff.load(&sample1);
        let bnum2 = ruff.load(&sample2);

        // schedule two samples ahead, so they should overlap by five ticks
        ruff.trigger(bnum1, 0.291);
        ruff.trigger(bnum2, 0.291 + (4.0 * sec_per_sample));
        
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
        
        let bnum1 = ruff.load(&sample1);
        let bnum2 = ruff.load(&sample2);

        // schedule two samples ahead, so they should  occur in different blocks
        // first sample should appear in block 100
        ruff.trigger(bnum1, 0.291);

        // second sample should appear ten blocks later
        let second_sample_timestamp = 0.291 + (10.0 * block_duration);
        
        ruff.trigger(bnum2, second_sample_timestamp);
        
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
