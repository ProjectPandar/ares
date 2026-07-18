mod bounds;
mod path;

use super::{ClipperError, ClosedClipper, PathRole};
use crate::geometry::Polygon;

#[cfg(test)]
use super::types::{InputSnapshot, LocalMinimumSnapshot};

impl ClosedClipper {
    pub(crate) fn add_closed_path(
        &mut self,
        path: &Polygon,
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        self.add_path(path.points(), role)
    }

    pub(crate) fn add_closed_paths(
        &mut self,
        paths: &[Polygon],
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        let mut accepted_any = false;
        for path in paths {
            accepted_any |= self.add_closed_path(path, role)?;
        }
        Ok(accepted_any)
    }

    #[cfg(test)]
    pub(crate) fn input_snapshot(&self) -> InputSnapshot {
        InputSnapshot {
            use_full_range: self.use_full_range,
            edges: self.edges.snapshot(),
            minima: self
                .minima
                .iter()
                .map(|minimum| LocalMinimumSnapshot {
                    y: minimum.y,
                    left: minimum.left.0,
                    right: minimum.right.0,
                })
                .collect(),
        }
    }
}
