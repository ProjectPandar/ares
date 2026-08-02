mod bounds;
mod path;

use super::{Clipper, ClipperError, PathRole};
use crate::geometry::{Polygon, Polyline};

#[cfg(test)]
use super::types::{InputSnapshot, LocalMinimumSnapshot};

impl Clipper {
    pub(crate) fn add_closed_path(
        &mut self,
        path: &Polygon,
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        self.add_path(path.points(), role, true)
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

    pub(crate) fn add_open_path(
        &mut self,
        path: &Polyline,
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        if role == PathRole::Clip {
            return Err(ClipperError::OpenPathMustBeSubject);
        }
        self.add_path(path.points(), role, false)
    }

    pub(crate) fn add_open_paths(
        &mut self,
        paths: &[Polyline],
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        if role == PathRole::Clip {
            return Err(ClipperError::OpenPathMustBeSubject);
        }
        let mut accepted_any = false;
        for path in paths {
            accepted_any |= self.add_path(path.points(), role, false)?;
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
                    left: minimum.left.map(|id| id.0),
                    right: minimum.right.map(|id| id.0),
                })
                .collect(),
        }
    }
}
