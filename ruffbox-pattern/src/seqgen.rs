use rand::seq::SliceRandom;
use std::hash::Hash;
use vom_rs::safe_pfa::Pfa;

pub trait SequenceGenerator<T> {
    fn get_next(&mut self) -> Option<T>;
}

////////////
// RANDOM //
////////////

pub struct RandomSequenceGenerator<T> {
    items: Vec<T>
}

impl <T: Copy> RandomSequenceGenerator<T> {
    pub fn from_seq(seq: &Vec<T>) -> Self {
        RandomSequenceGenerator {
            items: seq.to_vec(),
        }
    }
}

impl <T: Copy> SequenceGenerator<T> for RandomSequenceGenerator<T> {    
    fn get_next(&mut self) -> Option<T> {
        match self.items.choose(&mut rand::thread_rng()) {
            Some(thing) => Some(*thing),
            None => None                
        }
    }
}

////////////
// CYCLE  //
////////////

pub struct CycleSequenceGenerator<T> {
    items: Vec<T>,
    index: usize,        
}

impl <T: Copy> CycleSequenceGenerator<T> {
    pub fn from_seq(seq: &Vec<T>) -> Self {
        CycleSequenceGenerator {
            items: seq.to_vec(),
            index: 0,
        }
    }
}

impl <T: Copy> SequenceGenerator<T> for CycleSequenceGenerator<T> {    
    fn get_next(&mut self) -> Option<T> {
        let item = self.items[self.index];

        self.index += 1;
        
        if self.index >= self.items.len() {
            self.index = 0;
        } 

        Some(item)        
    }
}

/////////
// PFA //
/////////

pub struct PfaSequenceGenerator<T: Eq + Copy + Hash> {
    pfa: Pfa<T>,
}

impl <T: Eq + Copy + Hash> PfaSequenceGenerator<T> {
    pub fn from_seq(seq: &Vec<T>) -> Self {        
        PfaSequenceGenerator {            
            pfa: Pfa::learn(&seq, 3, 0.01, 30),
        }
    }
}

// fixed to second order, for now 
impl <T: Eq + Copy + Hash> SequenceGenerator<T> for PfaSequenceGenerator<T> {    
    fn get_next(&mut self) -> Option<T> {
        self.pfa.next_symbol()
    }
}

//////////
// RAMP //
//////////

pub struct RampSequenceGenerator {
    min: f32,
    inc: f32,
    steps: usize,
    step_count: usize,
}

impl RampSequenceGenerator {
    pub fn from_params(min: f32, max: f32, steps: usize) -> Self {        
        RampSequenceGenerator {            
            min: min,
            inc: (max - min) / (steps as f32),
            steps: steps,
            step_count: 0,
        }
    }
}

// fixed to second order, for now
impl SequenceGenerator<f32> for RampSequenceGenerator {    
    fn get_next(&mut self) -> Option<f32> {
        let cur = self.min + self.step_count as f32 * self.inc;
        self.step_count += 1;
        if self.step_count > self.steps {
            self.step_count = 0;
        }
        Some(cur)
    }
}

////////////
// BOUNCE //
////////////

// sinusoidal bounce

pub struct BounceSequenceGenerator {
    min: f32,
    degree_inc: f32,
    range: f32,
    steps: usize,
    step_count: usize,
}

impl BounceSequenceGenerator {
    pub fn from_params(min: f32, max: f32, steps: usize) -> Self {        
        BounceSequenceGenerator {                        
            min: min,
            range: max - min,
            degree_inc: 360.0 / steps as f32,            
            steps: steps,
            step_count: 0,
        }
    }
}

// fixed to second order, for now
impl SequenceGenerator<f32> for BounceSequenceGenerator {    
    fn get_next(&mut self) -> Option<f32> {
        let degree = (self.degree_inc * (self.step_count as f32 % self.steps as f32)) % 360.0;
        let abs_sin = degree.to_radians().sin().abs();
        let cur = self.min + (abs_sin * self.range);
        self.step_count += 1;
        Some(cur)
    }
}

