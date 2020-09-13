use rand::seq::SliceRandom;
use std::hash::Hash;
use vom_rs::pfa::Pfa;

use decorum::N32;

pub trait SequenceGenerator<T, S> {
    fn get_next(&mut self) -> Option<T>;
    fn get_state(&self) -> S;
}

////////////
// RANDOM //
////////////

pub struct RandomSequenceGenerator<T> {
    items: Vec<T>
}

impl <T: Copy + Ord> RandomSequenceGenerator<T> {
    pub fn from_seq(seq: &Vec<T>) -> Self {
        RandomSequenceGenerator {
            items: seq.to_vec(),
        }
    }
}

impl <T: Copy> SequenceGenerator<T, usize> for RandomSequenceGenerator<T> {    
    fn get_next(&mut self) -> Option<T> {
        match self.items.choose(&mut rand::thread_rng()) {
            Some(thing) => Some(*thing),
            None => None                
        }
    }

    fn get_state(&self) -> usize {
        0
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

    pub fn from_seq_with_index(seq: &Vec<T>, idx: usize) -> Self {
        let mut idx_clamp = idx;
        if seq.len() <= idx {
            idx_clamp = seq.len() - 1;
        }
        CycleSequenceGenerator {
            items: seq.to_vec(),
            index: idx_clamp,
        }
    }    
}

impl <T: Copy> SequenceGenerator<T, usize> for CycleSequenceGenerator<T> {    
    fn get_next(&mut self) -> Option<T> {
        let item = self.items[self.index];

        self.index += 1;
        
        if self.index >= self.items.len() {
            self.index = 0;
        }
                
        Some(item)        
    }

    fn get_state(&self) -> usize {
        self.index
    }
}

/////////
// PFA //
/////////

pub struct PfaSequenceGenerator<T: Eq + Copy + Hash + Ord> {
    pfa: Pfa<T>,
}

impl <T: Eq + Copy + Hash + Ord> PfaSequenceGenerator<T> {
    pub fn from_seq(seq: &Vec<T>) -> Self {        
        PfaSequenceGenerator {            
            pfa: Pfa::learn(&seq, 3, 0.01, 30),
        }
    }
}

impl <T: Eq + Copy + Hash + Ord> SequenceGenerator<T, usize> for PfaSequenceGenerator<T> {    
    fn get_next(&mut self) -> Option<T> {
        self.pfa.next_symbol()
    }

    fn get_state(&self) -> usize {
        0
    }
}

//////////
// RAMP //
//////////

pub struct RampSequenceGenerator {
    min: N32,
    inc: N32,
    steps: N32,
    step_count: N32,
}

impl RampSequenceGenerator {
    pub fn from_params(min: N32, max: N32, steps: N32) -> Self {        
        RampSequenceGenerator {            
            min: min,
            inc: (max - min) / steps,
            steps: steps,
            step_count: (0.0).into(),
        }
    }
}

impl SequenceGenerator<N32, usize> for RampSequenceGenerator {    
    fn get_next(&mut self) -> Option<N32> {
        let cur = self.min + self.step_count * self.inc;
        self.step_count = self.step_count + 1.0;
        if self.step_count > self.steps {
            self.step_count = (0.0).into();
        }
        Some(cur)
    }

    fn get_state(&self) -> usize {
        let state_raw:f32 = self.step_count.into();
        state_raw as usize
    }
}

////////////
// BOUNCE //
////////////

// sinusoidal bounce
pub struct BounceSequenceGenerator {
    min: N32,
    degree_inc: N32,
    range: N32,
    steps: N32,
    step_count: N32,
}

impl BounceSequenceGenerator {
    pub fn from_params(min: N32, max: N32, steps: N32) -> Self {
        let mut dec_inc:N32 = (360.0).into();
        dec_inc = dec_inc / steps;
        BounceSequenceGenerator {                        
            min: min,
            range: max - min,
            degree_inc: dec_inc,            
            steps: steps,
            step_count: (0.0).into(),
        }
    }
}

impl SequenceGenerator<N32, usize> for BounceSequenceGenerator {    
    fn get_next(&mut self) -> Option<N32> {
        // why doesn't rust has a hashable float ?????
        let deg_inc_raw:f32 = self.degree_inc.into();
        let mut step_count_raw:f32 = self.step_count.into();
        let steps_raw:f32 = self.steps.into();
        let min_raw:f32 = self.min.into();
        let range_raw:f32 = self.range.into();
                
        let degree:f32 = (deg_inc_raw * (step_count_raw % steps_raw)) % 360.0;
        let abs_sin:f32 = degree.to_radians().sin().abs().into();
        
        let cur:f32 = min_raw + (abs_sin * range_raw);

        step_count_raw += 1.0;
        self.step_count = step_count_raw.into(); 
        
        Some(cur.into())
    }

    fn get_state(&self) -> usize {
        let state_raw:f32 = self.step_count.into();
        state_raw as usize
    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
        
    #[test]
    fn test_bounce_gen() {
        let mut bounce_gen = BounceSequenceGenerator::from_params((20.0).into(), (200.0).into(), (10.0).into());
        let mut results = Vec::new();
        for _ in 0..10 {
            results.push(bounce_gen.get_next());
        }
        println!("Result: {:?}", results);
    }

    #[test]
    fn test_ramp_gen() {
        let mut ramp_gen = RampSequenceGenerator::from_params((20.0).into(), (200.0).into(), (10.0).into());
        let mut results = Vec::new();
        for _ in 0..10 {
            results.push(ramp_gen.get_next());
        }
        println!("Result: {:?}", results);
    }

    #[test]
    fn test_pfa_gen() {
        let in_vec:Vec<N32> = vec![(20.0).into(), (200.0).into(), (10.0).into(), (20.0).into(), (200.0).into(), (10.0).into(),
                          (200.0).into(), (200.0).into(), (10.0).into(), (10.0).into(), (200.0).into(), (10.0).into(),
                          (200.0).into(), (20.0).into(), (10.0).into(), (20.0).into(), (20.0).into(), (10.0).into()];
        let mut pfa_gen = PfaSequenceGenerator::from_seq(&in_vec);
        let results:Vec<N32> = Vec::new();
        for _ in 0..10 {
            println!("Result: {:?}", pfa_gen.get_next());
            
        }
        println!("Result: {:?}", results);
    }

}
