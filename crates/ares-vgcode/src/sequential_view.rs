// Ports the rendering-neutral part of AGPL-licensed OrcaSlicer
// `src/slic3r/GUI/GCodeViewer.cpp` `GCodeViewer::SequentialView`: the tool
// marker latch of `GCodeViewer::render` (`:1594-1601`) and the vertex
// `Marker::render_position_window` describes (`:345-352`).

use crate::{MoveType, ViewType, ViewerImpl};

/// The tool marker of a shown `SequentialView`: the current vertex it is drawn
/// at (`GCodeViewer.cpp:1597-1598`) and the vertex its position window shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToolMarker {
    pub vertex: usize,
    pub window_vertex: usize,
}

/// `GCodeViewer::SequentialView::m_show_marker`, false for a new load
/// (`GCodeViewer.cpp:1388`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SequentialView {
    show_marker: bool,
}

impl SequentialView {
    /// `GCodeViewer::render` (`GCodeViewer.cpp:1594-1601`): the marker shows
    /// once the visible range ends before the full range and stays shown;
    /// `m_no_render_path` is never set. Outside the FeatureType view a seam
    /// current vertex shows the vertex before it in the position window
    /// (`GCodeViewer.cpp:345-352`).
    pub fn update(&mut self, viewer: &ViewerImpl) -> Option<ToolMarker> {
        let visible = viewer.get_view_visible_range();
        self.show_marker |= visible[1] != viewer.get_view_full_range()[1];
        if !self.show_marker {
            return None;
        }
        let vertex = viewer.get_current_vertex_id();
        let seam = viewer.get_view_type() != ViewType::FeatureType
            && viewer
                .vertices()
                .get(vertex)
                .is_some_and(|v| v.move_type == MoveType::Seam);
        let window_vertex = match visible[1] {
            last if seam && last > 0 => last - 1,
            _ => vertex,
        };
        Some(ToolMarker {
            vertex,
            window_vertex,
        })
    }
}
