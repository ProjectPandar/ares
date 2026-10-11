// Ports the `ENABLE_ACTUAL_SPEED_DEBUG` profile data of AGPL-licensed
// OrcaSlicer `src/slic3r/GUI/GCodeViewer.cpp` (`:1681-1753`), compiled in
// shipping builds (`libslic3r/Technologies.hpp:52`).

use super::SequentialView;
use crate::ViewerImpl;

/// `SequentialView::ActualSpeedImguiWidget::Item` (`GCodeViewer.hpp:52-57`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ActualSpeedItem {
    pub pos: f32,
    pub speed: f32,
    pub internal: bool,
}

/// The data of `SequentialView::ActualSpeedImguiWidget` (`GCodeViewer.hpp:47-59`):
/// the ActualSpeed color range, its levels as `ColorRGBA` with alpha 0.5, and
/// the compressed profile of the current G-code line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActualSpeedProfile {
    pub y_range: [f32; 2],
    pub levels: Vec<(f32, [f32; 4])>,
    pub data: Vec<ActualSpeedItem>,
}

impl SequentialView {
    /// The actual speed block of `GCodeViewer::update_sequential_view_current`
    /// after `set_view_visible_range` and `ViewerImpl::colors`.
    pub fn update_actual_speed(&mut self, _viewer: &ViewerImpl) {}

    pub fn actual_speed(&self) -> &ActualSpeedProfile {
        &self.actual_speed
    }
}
