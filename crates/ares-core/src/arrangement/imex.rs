//! OrcaSlicer's IDEX/IQEX arranger at v2.5.0-beta (`345efec64`): `src/libslic3r/IMEXArrange.hpp`
//! and `IMEXArrange.cpp`. See `docs/architecture/web-plater-arrange-imex.md`, "A2a".

use super::{ArrangeParams, ArrangePolygon, Point};

/// `BoundingBoxf`: a box in mm.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoundingBoxf {
    pub min: [f64; 2],
    pub max: [f64; 2],
}

/// `ImexArrangeZones` (`IMEXArrange.hpp:15-20`): a plate's IDEX/IQEX zones as Arrange sees them,
/// in plate-local mm.
#[derive(Clone, Debug, PartialEq)]
pub struct ImexArrangeZones {
    pub primary_zone: BoundingBoxf,
    pub collision_zones: Vec<BoundingBoxf>,
}

/// `ImexArrangeInput` (`IMEXArrange.hpp:25-40`): what an arrange needs to know about the plates it
/// packs into.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImexArrangeInput {
    /// Per arranger bed, in order: the zones of the plate it packs into, `None` for a plate whose
    /// parts may go anywhere on the bed.
    pub beds: Vec<Option<ImexArrangeZones>>,
    /// Whether the beds past `beds` become new plates, as when arranging every plate, and the
    /// zones such a plate gets.
    pub adds_plates: bool,
    pub added_plate: Option<ImexArrangeZones>,
    /// Room a strip keeps for the brim: brim width plus its gap to the part, in mm.
    pub brim: f64,
}

/// `ImexArranger` (`IMEXArrange.hpp:42-99`): keeps an arrange inside each IDEX/IQEX plate's
/// primary zone and out of that zone's carriage collision strips.
pub struct ImexArranger;

impl ImexArranger {
    /// `bed` is the arranger's bed shape, shrunk by `params.bed_shrink_x/y`.
    pub fn new(_input: &ImexArrangeInput, _params: &ArrangeParams, _bed: &[Point]) -> Self {
        todo!("ImexArranger::new")
    }

    /// Whether any bed has a zone.
    pub fn active(&self) -> bool {
        todo!("ImexArranger::active")
    }

    /// The bed shape to arrange in.
    pub fn bed_shape(&self) -> Vec<Point> {
        todo!("ImexArranger::bed_shape")
    }

    /// Adds the fixed items that keep parts out of each bed's keep-outs.
    pub fn add_keep_outs(&self, _fixed: &mut Vec<ArrangePolygon>) {
        todo!("ImexArranger::add_keep_outs")
    }

    /// Call after `arrange(items, fixed, Bed::from_points(&bed_shape()), params)`, with `before`
    /// holding `items` as they were before it.
    pub fn finish(
        &self,
        _items: &mut [ArrangePolygon],
        _before: &[ArrangePolygon],
        _fixed: &[ArrangePolygon],
        _params: &ArrangeParams,
    ) {
        todo!("ImexArranger::finish")
    }
}
