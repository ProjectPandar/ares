mod rng;
pub(crate) use rng::Mt19937;

use std::fmt;

/// `PrintBase::SlicingStatus` (`PrintBase.hpp:443-481`): the slice's progress, its current
/// stage text and the scene flags. The warning fields are not ported.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlicingStatus<'a> {
    pub percent: i32,
    pub text: &'a str,
    pub flags: u32,
}

impl SlicingStatus<'_> {
    pub const DEFAULT: u32 = 0;
    /// The slices changed; a viewer reloads the object shells.
    pub const RELOAD_SCENE: u32 = 1 << 1;
}

/// Execution-local state for one project slice.
pub struct SlicerContext<'a> {
    seed: u32,
    rng: Mt19937,
    status: Option<&'a mut dyn FnMut(&SlicingStatus<'_>)>,
}

impl<'a> SlicerContext<'a> {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            rng: Mt19937::new(seed),
            status: None,
        }
    }

    /// `PrintBase::set_status_callback` (`PrintBase.hpp:489`): `callback` receives every status
    /// of the slice, in the order `Print::process` reports them.
    pub fn set_status_callback(&mut self, callback: &'a mut dyn FnMut(&SlicingStatus<'_>)) {
        self.status = Some(callback);
    }

    /// `PrintBase::set_status` (`PrintBase.cpp:123-128`). The text is formatted only when a
    /// callback is registered.
    pub(crate) fn set_status(&mut self, percent: i32, text: fmt::Arguments<'_>, flags: u32) {
        let Some(callback) = self.status.as_mut() else {
            return;
        };
        match text.as_str() {
            Some(text) => callback(&SlicingStatus {
                percent,
                text,
                flags,
            }),
            None => callback(&SlicingStatus {
                percent,
                text: &text.to_string(),
                flags,
            }),
        }
    }

    pub(crate) fn seed(&self) -> u32 {
        self.seed
    }

    pub(crate) fn rng(&mut self) -> &mut Mt19937 {
        &mut self.rng
    }
}
