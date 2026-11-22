mod rng;
pub(crate) use rng::Mt19937;

use crate::ThumbnailData;
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
    thumbnail_images: Vec<ThumbnailData>,
    pub(crate) max_triangle_count: i32,
    pub(crate) no_check: bool,
    pub(crate) cli_path_conflict_check: bool,
    pub(crate) enable_timelapse: bool,
    pub(crate) custom_gcodes: Option<(i32, crate::CustomGCodeInfo)>,
}

impl<'a> SlicerContext<'a> {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            rng: Mt19937::new(seed),
            status: None,
            thumbnail_images: Vec::new(),
            max_triangle_count: 0,
            no_check: false,
            cli_path_conflict_check: false,
            enable_timelapse: false,
            custom_gcodes: None,
        }
    }

    /// Sets the print's action-local `no_check` flag (OrcaSlicer.cpp:6988).
    #[must_use]
    pub fn with_no_check(mut self, no_check: bool) -> Self {
        self.no_check = no_check;
        self
    }

    /// Enables the direct CLI's `Print::process` conflict check without a wipe tower
    /// (`Print.cpp:3946–3964`) and reports a typed conflict to the CLI consumer.
    #[must_use]
    pub fn with_cli_path_conflict_check(mut self) -> Self {
        self.cli_path_conflict_check = true;
        self
    }

    /// Sets the CLI's runtime timelapse flag for the saved-project preset overlay
    /// (`OrcaSlicer.cpp:1593-1595,4303-4320`). This does not select a timelapse mode.
    #[must_use]
    pub fn with_enable_timelapse(mut self, enable_timelapse: bool) -> Self {
        self.enable_timelapse = enable_timelapse;
        self
    }

    /// Replaces model layer events with the supplied events for one zero-based plate.
    #[must_use]
    pub fn with_custom_gcodes(mut self, plate_index: i32, info: crate::CustomGCodeInfo) -> Self {
        self.custom_gcodes = Some((plate_index, info));
        self
    }

    /// Supplies the direct STL plate limit (`OrcaSlicer.cpp:6637`); zero is unlimited.
    #[must_use]
    pub fn with_max_triangle_count(mut self, count: i32) -> Self {
        self.max_triangle_count = count;
        self
    }

    /// Supplies the image payload of `ThumbnailsGeneratorCallback` for this slice.
    #[must_use]
    pub fn with_thumbnail_images(mut self, images: Vec<ThumbnailData>) -> Self {
        self.thumbnail_images = images;
        self
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

    pub(crate) fn thumbnail_images(&self) -> &[ThumbnailData] {
        &self.thumbnail_images
    }

    pub(crate) fn seed(&self) -> u32 {
        self.seed
    }

    pub(crate) fn rng(&mut self) -> &mut Mt19937 {
        &mut self.rng
    }
}
