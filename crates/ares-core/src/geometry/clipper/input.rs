mod bounds;
mod path;

use super::round_policy::{Clipper1Round, RoundPolicy};
use super::{Clipper, ClipperError, PathRole};
use crate::geometry::{Polygon, Polyline};

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
        self.add_closed_paths_with_policy::<Clipper1Round>(paths, role)
    }

    pub(crate) fn add_closed_paths_with_policy<R: RoundPolicy>(
        &mut self,
        paths: &[Polygon],
        role: PathRole,
    ) -> Result<bool, ClipperError> {
        let mut accepted_any = false;
        for path in paths {
            accepted_any |= self.add_path_with_policy::<_, R>(path.points(), role, true)?;
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
}
