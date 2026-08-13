use crate::{
    ProcessInfillPattern, RegionOptions,
    geometry::{
        ClipperError, ExPolygon, Polygon, difference_polygons_ex_with_safety_offset,
        union_safety_offset_ex,
    },
    project_slice::region_slices::{RegionSurface, RegionSurfaceKind},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Pattern {
    Monotonic,
    MonotonicLine,
    CrossHatch,
}

impl Pattern {
    const fn rank(self) -> u8 {
        match self {
            Self::Monotonic => 0,
            Self::MonotonicLine => 1,
            Self::CrossHatch => 20,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct GroupKey {
    bridge_angle: f32,
    pattern: Pattern,
}

impl GroupKey {
    fn precedes(self, other: Self) -> bool {
        if self.bridge_angle > other.bridge_angle {
            return true;
        }
        if self.bridge_angle < other.bridge_angle {
            return false;
        }
        self.pattern.rank() < other.pattern.rank()
    }

    fn equivalent(self, other: Self) -> bool {
        !self.precedes(other) && !other.precedes(self)
    }
}

pub(super) struct SurfaceFill {
    pub(super) representative_kind: RegionSurfaceKind,
    pub(super) pattern: Pattern,
    key: GroupKey,
    pub(super) expolygons: Vec<ExPolygon>,
}

pub(super) fn group_and_prioritize(
    surfaces: &[RegionSurface],
    options: &RegionOptions,
) -> Result<Vec<SurfaceFill>, ClipperError> {
    let mut groups = Vec::new();
    for surface in surfaces {
        let (kind, expolygon, thickness, thickness_layers, bridge_angle, _) = surface.as_parts();
        debug_assert_eq!(thickness, -1.0);
        debug_assert_eq!(thickness_layers, 1);
        let pattern = projected_pattern(kind, options);
        let key = GroupKey {
            bridge_angle: bridge_angle as f32,
            pattern,
        };
        insert_surface(&mut groups, kind, key, expolygon.clone());
    }
    apply_priority(&mut groups)?;
    Ok(groups)
}

fn projected_pattern(kind: RegionSurfaceKind, options: &RegionOptions) -> Pattern {
    match kind {
        RegionSurfaceKind::BottomBridge => {
            debug_assert_eq!(
                options.top_surface_pattern,
                ProcessInfillPattern::MonotonicLine
            );
            Pattern::Monotonic
        }
        RegionSurfaceKind::InternalSolid => {
            debug_assert_eq!(
                options.internal_solid_infill_pattern,
                ProcessInfillPattern::Monotonic
            );
            Pattern::Monotonic
        }
        RegionSurfaceKind::Top => {
            debug_assert_eq!(
                options.top_surface_pattern,
                ProcessInfillPattern::MonotonicLine
            );
            Pattern::MonotonicLine
        }
        RegionSurfaceKind::Internal => {
            debug_assert_eq!(
                options.sparse_infill_pattern,
                ProcessInfillPattern::CrossHatch
            );
            Pattern::CrossHatch
        }
        RegionSurfaceKind::Bottom
        | RegionSurfaceKind::InternalBridge
        | RegionSurfaceKind::InternalVoid => {
            unreachable!("trusted sparse anchoring layer contains only the four KSR kinds")
        }
    }
}

fn insert_surface(
    groups: &mut Vec<SurfaceFill>,
    kind: RegionSurfaceKind,
    key: GroupKey,
    expolygon: ExPolygon,
) {
    for index in 0..groups.len() {
        if key.equivalent(groups[index].key) {
            groups[index].expolygons.push(expolygon);
            return;
        }
        if key.precedes(groups[index].key) {
            groups.insert(
                index,
                SurfaceFill {
                    representative_kind: kind,
                    pattern: key.pattern,
                    key,
                    expolygons: vec![expolygon],
                },
            );
            return;
        }
    }
    groups.push(SurfaceFill {
        representative_kind: kind,
        pattern: key.pattern,
        key,
        expolygons: vec![expolygon],
    });
}

fn apply_priority(groups: &mut [SurfaceFill]) -> Result<(), ClipperError> {
    let mut preceding = Vec::new();
    let group_count = groups.len();
    for (index, group) in groups.iter_mut().enumerate() {
        if group.expolygons.is_empty() {
            continue;
        }
        let subjects = flatten(&group.expolygons);
        if group.expolygons.len() > 1 || !preceding.is_empty() {
            group.expolygons = if preceding.is_empty() {
                union_safety_offset_ex(&subjects)?
            } else {
                difference_polygons_ex_with_safety_offset(&subjects, &preceding)?
            };
            preceding.extend(subjects);
        } else if index + 1 < group_count {
            preceding.extend(subjects);
        }
    }
    Ok(())
}

fn flatten(expolygons: &[ExPolygon]) -> Vec<Polygon> {
    let mut polygons = Vec::new();
    for expolygon in expolygons {
        polygons.push(expolygon.contour().clone());
        polygons.extend(expolygon.holes().iter().cloned());
    }
    polygons
}
