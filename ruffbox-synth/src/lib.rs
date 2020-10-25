#![feature(const_generics)]

#[macro_use]
extern crate lazy_static;

use parking_lot::Mutex;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut f32 {
    let vec: Vec<f32> = vec![0.0; size];
    Box::into_raw(vec.into_boxed_slice()) as *mut f32
}

pub mod ruffbox;

lazy_static! {
    static ref RUFF: Mutex<ruffbox::Ruffbox<128>> = Mutex::new(ruffbox::Ruffbox::new());
}

#[no_mangle]
pub extern "C" fn process(out_ptr_l: *mut f32, out_ptr_r: *mut f32, size: usize, stream_time: f64) {
    let mut ruff = RUFF.lock();

    let out_buf_l: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(out_ptr_l, size)};
    let out_buf_r: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(out_ptr_r, size)};

    // no more mono for now ... 
    let out = ruff.process(stream_time);
    for i in 0..128 {
        out_buf_l[i] = out[0][i];
        out_buf_r[i] = out[1][i];
    }    
}

#[no_mangle]
pub extern "C" fn prepare(src_type: ruffbox::synth::SourceType, timestamp: f64, sample_buf: usize) -> usize {
    let mut ruff = RUFF.lock();
    ruff.prepare_instance(src_type, timestamp, sample_buf)
}

#[no_mangle]
pub extern "C" fn set_instance_parameter(instance_id: usize, par: ruffbox::synth::SynthParameter, val: f32) {
    let mut ruff = RUFF.lock();
    ruff.set_instance_parameter(instance_id, par, val);
}

#[no_mangle]
pub extern "C" fn set_master_parameter(par: ruffbox::synth::SynthParameter, val: f32) {
    let mut ruff = RUFF.lock();
    ruff.set_master_parameter(par, val);
}

#[no_mangle]
pub extern "C" fn trigger(instance_id: usize) {
    let mut ruff = RUFF.lock();
    ruff.trigger(instance_id);
}

#[no_mangle]
pub extern "C" fn load(sample_ptr: *mut f32, size: usize) -> usize {
    let mut ruff = RUFF.lock();
    let in_buf: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(sample_ptr, size)};
    ruff.load_sample(in_buf)
}

