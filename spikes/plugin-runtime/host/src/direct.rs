use crate::{Guest, api::Plugin, samples::Sample, wire};
use anyhow::{Result, bail};

pub fn sample(subdivision: bool) -> Sample {
    if subdivision {
        Sample::Subdivided { lower_scale: 1.0 }
    } else {
        Sample::Staggered {
            offset_adjustment: 0.0,
        }
    }
}

pub struct Rust(pub Sample);

impl Guest for Rust {
    fn has_source(&self) -> bool {
        false
    }
    fn set_source(&mut self, _: &[u8]) -> Result<()> {
        bail!("compiled-in Rust cannot reload")
    }
    fn perimeters(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
        let edits = wire::with_layer(bytes, |layer| self.0.perimeters(layer))?;
        Ok(wire::encode_edits(&edits))
    }
    fn gcode(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
        Ok(self.0.gcode_post_process(bytes)?)
    }
}

#[cfg(any(feature = "direct-boa", feature = "direct-quickjs"))]
macro_rules! js_engine {
    ($name:ident, $engine:ident) => {
        pub struct $name(plugin_spike_js::$engine::Script);

        impl $name {
            pub fn new(bytes: &[u8]) -> Self {
                Self(plugin_spike_js::$engine::Script::new(
                    &plugin_spike_js::source(bytes),
                ))
            }
        }

        impl Guest for $name {
            fn has_source(&self) -> bool {
                true
            }
            fn set_source(&mut self, bytes: &[u8]) -> Result<()> {
                *self = Self::new(bytes);
                Ok(())
            }
            fn perimeters(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
                Ok(self.0.run("perimeters", bytes))
            }
            fn gcode(&mut self, bytes: &[u8]) -> Result<Vec<u8>> {
                Ok(self.0.run("gcodePostProcess", bytes))
            }
        }
    };
}

#[cfg(feature = "direct-quickjs")]
js_engine!(QuickJs, quickjs);
#[cfg(feature = "direct-boa")]
js_engine!(Boa, boa);

#[cfg(feature = "direct-boa")]
pub use plugin_spike_js::boa::probe as probe_boa;
#[cfg(feature = "direct-quickjs")]
pub use plugin_spike_js::quickjs::probe as probe_quickjs;
