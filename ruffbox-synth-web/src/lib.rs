use once_cell::sync::Lazy;
use parking_lot::Mutex;
use ruffbox_synth::building_blocks::{SynthParameterLabel, SynthParameterValue};
use ruffbox_synth::ruffbox::{
    init_ruffbox, PreparedInstance, ReverbMode, RuffboxControls, RuffboxPlayhead,
};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern "C" fn alloc(size: usize) -> *mut f32 {
    let vec: Vec<f32> = vec![0.0; size];
    Box::into_raw(vec.into_boxed_slice()) as *mut f32
}

static RUFF: Lazy<(
    Mutex<RuffboxControls<128, 2>>,
    Mutex<RuffboxPlayhead<128, 2>>,
)> = Lazy::new(|| {
    let (ctrl, play) = init_ruffbox::<128, 2>(0, 2.0, &ReverbMode::FreeVerb, 44100.0, 3000, 10);
    (Mutex::new(ctrl), Mutex::new(play))
});

#[no_mangle]
pub extern "C" fn process(out_ptr_l: *mut f32, out_ptr_r: *mut f32, size: usize, stream_time: f64) {
    let mut ruff = RUFF.1.lock();

    let out_buf_l: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(out_ptr_l, size) };
    let out_buf_r: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(out_ptr_r, size) };

    // mono for now ...
    let out = ruff.process(stream_time, false);
    for i in 0..128 {
        out_buf_l[i] = out[0][i];
        out_buf_r[i] = out[1][i];
    }
}

#[no_mangle]
pub extern "C" fn prepare(
    src_type: ruffbox_synth::synths::SynthType,
    timestamp: f64,
    sample_buf: usize,
) -> Box<PreparedInstance<128, 2>> {
    let ruff = RUFF.0.lock();
    let inst = ruff
        .prepare_instance(src_type, timestamp, sample_buf)
        .unwrap();
    Box::new(inst)
}

#[no_mangle]
pub extern "C" fn set_instance_parameter(
    mut instance: Box<PreparedInstance<128, 2>>,
    par: SynthParameterLabel,
    val: f32,
) -> Box<PreparedInstance<128, 2>> {
    instance.set_instance_parameter(par, &SynthParameterValue::ScalarF32(val));
    instance
}

#[no_mangle]
pub extern "C" fn set_master_parameter(par: SynthParameterLabel, val: f32) {
    let ruff = RUFF.0.lock();
    ruff.set_master_parameter(par, SynthParameterValue::ScalarF32(val));
}

#[no_mangle]
pub extern "C" fn trigger(instance: Box<PreparedInstance<128, 2>>) {
    let ruff = RUFF.0.lock();
    ruff.trigger(*instance);
}

#[no_mangle]
pub extern "C" fn load(sample_ptr: *mut f32, size: usize) -> usize {
    let ruff = RUFF.0.lock();
    let in_buf: &mut [f32] = unsafe { std::slice::from_raw_parts_mut(sample_ptr, size) };
    ruff.load_sample(&mut in_buf.to_vec(), true, 44100.0)
}
