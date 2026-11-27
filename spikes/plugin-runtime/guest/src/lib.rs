#[cfg(not(any(feature = "boa", feature = "quickjs")))]
use {plugin_spike_api::Plugin, plugin_spike_samples::Sample};

#[cfg(not(any(feature = "boa", feature = "quickjs")))]
fn sample() -> Sample {
    if cfg!(feature = "subdivide") {
        Sample::Subdivided {
            lower_scale: if cfg!(feature = "alternate") {
                0.5
            } else {
                1.0
            },
        }
    } else {
        Sample::Staggered {
            offset_adjustment: if cfg!(feature = "alternate") {
                0.25
            } else {
                0.0
            },
        }
    }
}

#[cfg(feature = "boa")]
use plugin_spike_js::boa::Script;
#[cfg(feature = "quickjs")]
use plugin_spike_js::quickjs::Script;
use std::cell::RefCell;

thread_local! {
    static INPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static OUTPUT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    #[cfg(any(feature = "boa", feature = "quickjs"))]
    static SCRIPT: RefCell<Option<Script>> = const { RefCell::new(None) };
}

#[unsafe(no_mangle)]
pub extern "C" fn reserve_input(len: u32) -> *mut u8 {
    INPUT.with_borrow_mut(|input| {
        input.resize(len as usize, 0);
        input.as_mut_ptr()
    })
}

#[cfg(any(feature = "boa", feature = "quickjs"))]
#[unsafe(no_mangle)]
pub extern "C" fn set_source() {
    let source = INPUT.with_borrow(|input| plugin_spike_js::source(input));
    SCRIPT.with_borrow_mut(|script| *script = Some(Script::new(&source)));
}

fn call(f: impl FnOnce(&[u8]) -> Vec<u8>) -> u64 {
    let bytes = INPUT.with_borrow(|input| f(input));
    OUTPUT.with_borrow_mut(|output| {
        *output = bytes;
        ((output.len() as u64) << 32) | output.as_ptr() as u32 as u64
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn perimeters() -> u64 {
    call(|input| {
        #[cfg(not(any(feature = "boa", feature = "quickjs")))]
        {
            let edits =
                plugin_spike_wire::with_layer(input, |layer| sample().perimeters(layer).unwrap());
            plugin_spike_wire::encode_edits(&edits)
        }
        #[cfg(any(feature = "boa", feature = "quickjs"))]
        SCRIPT.with_borrow_mut(|script| script.as_mut().unwrap().run("perimeters", input))
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn gcode_post_process() -> u64 {
    call(|input| {
        #[cfg(not(any(feature = "boa", feature = "quickjs")))]
        {
            sample().gcode_post_process(input).unwrap()
        }
        #[cfg(any(feature = "boa", feature = "quickjs"))]
        SCRIPT.with_borrow_mut(|script| script.as_mut().unwrap().run("gcodePostProcess", input))
    })
}
