mod rng;
pub(crate) use rng::Mt19937;

use crate::{SliceError, ThumbnailData};
use std::{
    fmt,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// `PrintBase::canceled()` / `PrintTryCancel` (`PrintBase.hpp:367-377,508-514,550-555`):
/// lightweight, thread-safe cooperative cancellation token.
#[derive(Clone, Debug, Default)]
pub struct CancelToken(Arc<AtomicBool>);

impl CancelToken {
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks the computation canceled (`PrintBase::cancel`, `PrintBase.hpp:510`).
    #[inline]
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    /// Returns `true` if cancellation was requested (`PrintBase::canceled`, `PrintBase.hpp:508`).
    #[inline]
    pub fn is_canceled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }

    /// Returns `Err(SliceError::Canceled)` when canceled (`PrintBase::throw_if_canceled`,
    /// `PrintBase.hpp:552`).
    #[inline]
    pub fn check(&self) -> Result<(), SliceError> {
        if self.is_canceled() {
            Err(SliceError::Canceled)
        } else {
            Ok(())
        }
    }
}

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
    cancel: CancelToken,
    status: Option<&'a mut dyn FnMut(&SlicingStatus<'_>)>,
    thumbnail_images: Vec<ThumbnailData>,
    pub(crate) max_triangle_count: i32,
    pub(crate) no_check: bool,
    pub(crate) cli_path_conflict_check: bool,
    pub(crate) enable_timelapse: bool,
    pub(crate) custom_gcodes: Option<(i32, crate::CustomGCodeInfo)>,
    pub(crate) downward: Option<crate::downward::ProjectCheck<'a>>,
    normative_check: bool,
    skip_modified_gcodes: bool,
    allow_mix_temp: bool,
}

impl<'a> SlicerContext<'a> {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            rng: Mt19937::new(seed),
            cancel: CancelToken::new(),
            status: None,
            thumbnail_images: Vec::new(),
            max_triangle_count: 0,
            no_check: false,
            cli_path_conflict_check: false,
            enable_timelapse: false,
            custom_gcodes: None,
            downward: None,
            normative_check: true,
            skip_modified_gcodes: false,
            allow_mix_temp: false,
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

    /// Reports explicit target machines compatible with the imported saved plates.
    #[must_use]
    pub fn with_downward_check<'b>(
        self,
        targets: &'b [crate::DownwardMachine],
        report: &'b mut Vec<String>,
    ) -> SlicerContext<'b>
    where
        'a: 'b,
    {
        report.clear();
        let status = self
            .status
            .map(|callback| callback as &mut dyn FnMut(&SlicingStatus<'_>));
        SlicerContext {
            status,
            downward: (!targets.is_empty()).then_some((targets, report)),
            ..self
        }
    }

    /// Supplies the direct STL plate limit (`OrcaSlicer.cpp:6637`); zero is unlimited.
    #[must_use]
    pub fn with_max_triangle_count(mut self, count: i32) -> Self {
        self.max_triangle_count = count;
        self
    }

    /// Controls admission of embedded post-processing settings on CLI slice paths.
    #[must_use]
    pub fn with_normative_check(mut self, enabled: bool) -> Self {
        self.normative_check = enabled;
        self
    }

    pub(crate) fn normative_check(&self) -> bool {
        self.normative_check
    }

    /// Applies the CLI `--skip-modified-gcodes` policy to this slice.
    #[must_use]
    pub fn with_skip_modified_gcodes(mut self, enabled: bool) -> Self {
        self.skip_modified_gcodes = enabled;
        self
    }

    pub(crate) fn skip_modified_gcodes(&self) -> bool {
        self.skip_modified_gcodes
    }

    /// Supplies the image payload of `ThumbnailsGeneratorCallback` for this slice.
    #[must_use]
    pub fn with_thumbnail_images(mut self, images: Vec<ThumbnailData>) -> Self {
        self.thumbnail_images = images;
        self
    }

    /// `Print::set_check_multi_filaments_compatibility(!allow)` (`Print.hpp:1360`).
    #[must_use]
    pub fn with_allow_mix_temp(mut self, allow: bool) -> Self {
        self.allow_mix_temp = allow;
        self
    }

    pub(crate) fn allow_mix_temp(&self) -> bool {
        self.allow_mix_temp
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

    /// Attaches a caller-owned [`CancelToken`] to this slice (`PrintBase::cancel` /
    /// `PrintBase::throw_if_canceled`, `PrintBase.hpp:510,552`).
    #[must_use]
    pub fn with_cancel_token(mut self, token: CancelToken) -> Self {
        self.cancel = token;
        self
    }

    /// Returns a cloned [`CancelToken`] handle sharing this context's cancellation state
    /// (`PrintBase::make_try_cancel`, `PrintBase.hpp:554`).
    pub fn cancel_token(&self) -> CancelToken {
        self.cancel.clone()
    }

    pub(crate) fn cancel_ref(&self) -> &CancelToken {
        &self.cancel
    }

    /// Returns `Err(SliceError::Canceled)` if cancellation was requested
    /// (`PrintBase::throw_if_canceled`, `PrintBase.hpp:552`).
    #[inline]
    pub fn check_canceled(&self) -> Result<(), SliceError> {
        self.cancel.check()
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
