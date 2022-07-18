//use js_sys::Math;
pub mod parser;
pub mod seqgen;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use wasm_bindgen::prelude::*;

use crate::seqgen::*;
use serde::{Serialize, Deserialize};

use decorum::N32;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into())
    }
}

type EventHash = u64;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

struct MainEvent {
    name: String,
    params: HashMap<String, N32>,
}

impl Hash for MainEvent {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        for (par, val) in self.params.iter() {
            par.hash(state);
            (*val).hash(state);
        }
    }
}

impl MainEvent {
    fn from_parsed_input(input_name: String, input_params: &Vec<(&str, f32)>) -> Self {
        let mut param_map = HashMap::new();

        for param_tuple in input_params {
            param_map.insert(param_tuple.0.to_string(), param_tuple.1.into());
        }

        MainEvent {
            name: input_name,
            params: param_map,
        }
    }

    fn get_raw_params(&self) -> HashMap<String, f32> {
        let mut map = HashMap::new();

        for (k, v) in self.params.iter() {
            map.insert(k.clone(), v.into_inner());
        }

        map
    }
}

impl PartialEq for MainEvent {
    fn eq(&self, other: &Self) -> bool {
        for (param, value) in self.params.iter() {
            if !other.params.contains_key(param) {
                return false;
            } else if *value != other.params[param] {
                return false;
            }
        }
        self.name == other.name
    }
}

/// A simple event sequence represented by a vector of strings and params
struct EventSequence {
    event_refs: HashMap<EventHash, MainEvent>,
    events: Box<dyn SequenceGenerator<EventHash, usize>>,
    param_generators: HashMap<String, Box<dyn SequenceGenerator<N32, usize>>>,
}

impl EventSequence {
    /// Create an event sequence from a string.    
    pub fn from_parsed_line_ast(
        input_line: (
            (&str, Vec<(&str, Vec<(&str, f32)>)>),
            Vec<((&str, &str), Vec<f32>)>,
        ),
    ) -> Self {
        let pattern_ast = input_line.0;
        let param_asts = input_line.1;

        let mut main_events = HashMap::new();
        let mut event_hashes = Vec::new();

        for parsed_event in pattern_ast.1.iter() {
            let main_event =
                MainEvent::from_parsed_input(parsed_event.0.to_string(), &parsed_event.1);
            let main_event_hash = calculate_hash::<MainEvent>(&main_event);
            main_events.insert(main_event_hash, main_event);
            event_hashes.push(main_event_hash);
        }

        let mut param_row_map: HashMap<String, Box<dyn SequenceGenerator<N32, usize>>> =
            HashMap::new();

        for parsed_param_seq in param_asts.iter() {
            let mut param_conv: Vec<N32> = Vec::new();
            for raw_float in &parsed_param_seq.1 {
                param_conv.push((*raw_float).into())
            }

            param_row_map.insert(
                (parsed_param_seq.0).0.to_string(),
                match (parsed_param_seq.0).1 {
                    "rnd" => Box::new(RandomSequenceGenerator::from_seq(&param_conv)),
                    "cyc" => Box::new(CycleSequenceGenerator::from_seq(&param_conv)),
                    "learn" => Box::new(PfaSequenceGenerator::from_seq(&param_conv)),
                    "bounce" => Box::new(BounceSequenceGenerator::from_params(
                        param_conv[0],
                        param_conv[1],
                        param_conv[2],
                    )),
                    "ramp" => Box::new(RampSequenceGenerator::from_params(
                        param_conv[0],
                        param_conv[1],
                        param_conv[2],
                    )),
                    //"brownian" => Box::new(BounceSequenceGenerator::from_params(param_conv[0], param_conv[1], param_conv[2])),
                    _ => Box::new(CycleSequenceGenerator::from_seq(&param_conv)),
                },
            );
        }

        EventSequence {
            event_refs: main_events,
            events: match pattern_ast.0 {
                "rnd" => Box::new(RandomSequenceGenerator::from_seq(&event_hashes)),
                "cyc" => Box::new(CycleSequenceGenerator::from_seq(&event_hashes)),
                "learn" => Box::new(PfaSequenceGenerator::from_seq(&event_hashes)),
                _ => Box::new(CycleSequenceGenerator::from_seq(&event_hashes)),
            },

            param_generators: param_row_map,
        }
    }

    /// Update an existing sequence from a string.
    pub fn update_sequence(
        &mut self,
        input_line: (
            (&str, Vec<(&str, Vec<(&str, f32)>)>),
            Vec<((&str, &str), Vec<f32>)>,
        ),
    ) {
        self.event_refs.clear();
        self.param_generators.clear();

        let pattern_ast = input_line.0;
        let param_asts = input_line.1;

        let mut main_events = HashMap::new();
        let mut event_hashes = Vec::new();

        //let mut param_row_map: HashMap<String, Box<dyn SequenceGenerator<N32>>> = HashMap::new();

        for parsed_param_seq in param_asts.iter() {
            let mut param_conv: Vec<N32> = Vec::new();
            for raw_float in &parsed_param_seq.1 {
                param_conv.push((*raw_float).into())
            }

            let key = (parsed_param_seq.0).0.to_string();
            let mut state = 0;
            if self.param_generators.contains_key(&key) {
                state = self.param_generators[&key].get_state();
            }

            self.param_generators.insert(
                key,
                match (parsed_param_seq.0).1 {
                    "rnd" => Box::new(RandomSequenceGenerator::from_seq(&param_conv)),
                    "cyc" => Box::new(CycleSequenceGenerator::from_seq_with_index(
                        &param_conv,
                        state,
                    )),
                    "learn" => Box::new(PfaSequenceGenerator::from_seq(&param_conv)),
                    "bounce" => Box::new(BounceSequenceGenerator::from_params(
                        param_conv[0],
                        param_conv[1],
                        param_conv[2],
                    )),
                    "ramp" => Box::new(RampSequenceGenerator::from_params(
                        param_conv[0],
                        param_conv[1],
                        param_conv[2],
                    )),
                    //"brownian" => Box::new(BounceSequenceGenerator::from_params(param_conv[0], param_conv[1], param_conv[2])),
                    _ => Box::new(CycleSequenceGenerator::from_seq(&param_conv)),
                },
            );
        }

        for parsed_event in pattern_ast.1.iter() {
            let main_event =
                MainEvent::from_parsed_input(parsed_event.0.to_string(), &parsed_event.1);
            let main_event_hash = calculate_hash::<MainEvent>(&main_event);
            main_events.insert(main_event_hash, main_event);
            event_hashes.push(main_event_hash);
        }

        self.event_refs = main_events;

        let cycle_state = self.events.get_state();

        self.events = match pattern_ast.0 {
            "rnd" => Box::new(RandomSequenceGenerator::from_seq(&event_hashes)),
            "cyc" => Box::new(CycleSequenceGenerator::from_seq_with_index(
                &event_hashes,
                cycle_state,
            )),
            "learn" => Box::new(PfaSequenceGenerator::from_seq(&event_hashes)),
            _ => Box::new(CycleSequenceGenerator::from_seq(&event_hashes)),
        };
    }

    /// get the next event in the sequence
    pub fn get_next_event(&mut self) -> (String, HashMap<String, f32>) {
        let mut final_param_map: HashMap<String, f32> = HashMap::new();
        match self.events.get_next() {
            Some(ev_hash) => {
                let ev = &self.event_refs[&ev_hash];
                if ev.name == "~" {
                    return ("~".to_string(), final_param_map);
                }
                // pref for dyn params, so insert fixed pars first (might be overwritten)
                for (par, val) in ev.params.iter() {
                    final_param_map.insert(par.to_string(), (*val).into());
                }

                // pref for dyn params, so insert fixed pars first (might be overwritten)
                for (par, gen) in self.param_generators.iter_mut() {
                    match gen.get_next() {
                        Some(val) => final_param_map.insert(par.to_string(), val.into()),
                        None => None,
                    };
                }

                (ev.name.clone(), final_param_map)
            }
            None => ("~".to_string(), final_param_map),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TriggerData {
    pub params: HashMap<String, f32>,
    pub timestamp: f64,
    pub sample_id: String,
    pub source_type: String,
}

/// A simple time-recursion event scheduler running at a fixed time interval.
#[wasm_bindgen]
pub struct Scheduler {
    /// time this scheduler was started (AudioContext.currentTime)
    audio_start_time: f64,
    /// time this scheduler was started (performance.now())
    browser_start_time: f64,
    audio_logical_time: f64,
    browser_logical_time: f64,
    lookahead: f64, // in seconds
    tempo: f64, // currently just the duration of a 16th note ...
    event_sequences: Vec<EventSequence>,
    event_variables: HashMap<String, MainEvent>,
}

#[wasm_bindgen]
impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            audio_start_time: 0.0,
            browser_start_time: 0.0,
            audio_logical_time: 0.0,
            browser_logical_time: 0.0,
            lookahead: 0.100,
            tempo: 128.0,
            event_sequences: Vec::new(),
            event_variables: HashMap::new(),
        }
    }

    /// Evaluate an input string, turn it into a series of event sequences.
    pub fn evaluate(&mut self, input: Option<String>) {
        match input {
            Some(all_lines) => {
                let mut seq_idx = 0;

                for line in all_lines.lines() {
                    let trimmed_line = line.trim();

                    if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') {
                        // only these two for now ... could probably be solved more elegantly in the
                        // parser itself ...
                        match parser::variable_definiton(trimmed_line) {
                            Ok(var_ast) => {
                                let res = var_ast.1;
                                let var_def = res.0;
                                let ev_def = res.1;
                                let name = var_def.1;
                                let event =
                                    MainEvent::from_parsed_input(ev_def.0.to_string(), &ev_def.1);
                                self.event_variables.insert(name.to_string(), event);
                            }
                            Err(_) => {
                                match parser::pattern_line(trimmed_line) {
                                    Ok(pat_ast) => {
                                        if self.event_sequences.len() > seq_idx {
                                            self.event_sequences[seq_idx]
                                                .update_sequence(pat_ast.1);
                                        } else {
                                            self.event_sequences.push(
                                                EventSequence::from_parsed_line_ast(pat_ast.1),
                                            );
                                        }
                                    }
                                    Err(pat_err) => {
                                        log!("invalid line! {:?}, {}", pat_err, trimmed_line)
                                    } // ??
                                };
                                seq_idx += 1;
                            }
                        };
                    }
                }

                // check if we need to remove some sequnces because the number of lines got reduced ...
                if seq_idx < self.event_sequences.len() {
                    self.event_sequences.truncate(seq_idx);
                }
            }

            None => log!("no input!"),
        }
    }

    /// Fetch all events from the event sequences, pass then to the JS scheduler 
    pub fn generate_events(&mut self) -> Vec<JsValue> {
	let mut triggers = Vec::new();
		
        let trigger_time = self.audio_logical_time + self.lookahead;
		
        for seq in self.event_sequences.iter_mut() {
            let (mut next_event, mut next_params) = seq.get_next_event();

            // overwrite with variable if there is one ...
            if self.event_variables.contains_key(&next_event) {
                // first get params, then overwrite key
                let var_params = self.event_variables[&next_event].get_raw_params();
                for (k, v) in var_params {
                    next_params.entry(k).or_insert(v);
                }
                next_event = self.event_variables[&next_event].name.clone();
            }

            let next_source_type = match next_event.as_str() {
                "sine" => "SineSynth",
                "saw" => "LFSawSynth",
                "sqr" => "LFSquareSynth",
                _ => "Sampler",
            };
	    
	    // might not be the most efficient way to do this but I doubt that the old stdweb
	    // implementation was any faster ... 
	    if next_event != "~" {
		triggers.push(JsValue::from_serde(&TriggerData {
		    params: next_params,
		    timestamp: trigger_time,
		    sample_id: next_event.to_string(),
		    source_type: next_source_type.to_string(),
		}).unwrap());
            }
        }

	triggers
    }

    /// The main scheduler recursion.
    pub fn compensate_time(&mut self, browser_timestamp: f64) -> f64 {
                       
        // Calculate drift, correct timing.
        // The time at which this is called is most likely later, but never earlier,
        // than the time it SHOULD have been called at (self.browser_logical_time).
        // To compensate for the delay, we schedule the next call a bit earlier
        // than the actual interval.
        let next_schedule_time = self.tempo - (browser_timestamp - self.browser_logical_time);

        // Advance timestamps!
        // audio time in seconds
        self.audio_logical_time += self.tempo / 1000.0;

        // browser time in milliseconds
        self.browser_logical_time += self.tempo;
	        
	next_schedule_time   	
    }

    /// Start this scheduler.
    pub fn start(&mut self, audio_timestamp: f64, browser_timestamp: f64) {
        self.audio_start_time = audio_timestamp;
        self.browser_start_time = browser_timestamp;
        self.audio_logical_time = self.audio_start_time;
        self.browser_logical_time = self.browser_start_time;                
    }
    
    /// Set tick duration.
    pub fn set_tempo(&mut self, tempo: f64) {
        self.tempo = tempo;
    }
}
