//! OrcaSlicer's IDEX/IQEX arranger at v2.5.0-beta (`345efec64`): `src/libslic3r/IMEXArrange.hpp`
//! and `IMEXArrange.cpp`. See `docs/architecture/web-plater-arrange-imex.md`, "A2a".

use super::{
    ArrangeParams, ArrangePolygon, Bed, BoundingBox, Coord, ExPolygon, MAX_NUM_PLATES, Point,
    Polygon, SCALING_FACTOR, UNARRANGED,
};
use crate::geometry::intersection_polygons_paths;

mod input;

/// The arranger's names for the keep-outs (`IMEXArrange.cpp:27-31`). `finish` leaves the fences
/// out when it arranges a plate inside its zone, where the zone itself is the bed shape.
const FENCE_NAME: &str = "IMEXOutsidePrimaryZone";
const STRIP_NAME: &str = "IMEXCollisionZone";
/// `EPSILON` (`libslic3r.h`).
const EPSILON: f64 = 1e-4;

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

/// A bed that has zones.
struct ZoneBed {
    bed_idx: i32,
    zones: ImexArrangeZones,
}

struct KeepOut {
    bed_idx: i32,
    bounds: BoundingBox,
    /// Outside the primary zone, rather than a strip.
    fence: bool,
}

/// `ImexArranger` (`IMEXArrange.hpp:42-99`): keeps an arrange inside each IDEX/IQEX plate's
/// primary zone and out of that zone's carriage collision strips.
///
/// The arranger packs every bed into one bed shape. When every bed has the same primary zone,
/// that shape becomes the zone. Otherwise each plate in a parallel mode fences off the rest of its
/// bed with fixed items on its own bed, and `finish` arranges each fenced plate again inside its
/// zone.
pub struct ImexArranger {
    bed: Vec<Point>,
    bed_bb: BoundingBox,
    margin: [Coord; 2],
    adds_plates: bool,
    beds: Vec<ZoneBed>,
    shared_room: Option<BoundingBox>,
    keep_outs: Vec<KeepOut>,
}

/// `scaled<coord_t>(double)`: divided and truncated.
fn scaled(value: f64) -> Coord {
    (value / SCALING_FACTOR) as Coord
}

/// `scaled<coord_t>(float)`: divided in float.
fn scaled_f32(value: f32) -> Coord {
    (value / SCALING_FACTOR as f32) as Coord
}

/// `scaled(BoundingBoxf)` (`BoundingBox.hpp:284`).
fn scaled_box(bounds: BoundingBoxf) -> BoundingBox {
    BoundingBox::new(
        Point::new(scaled(bounds.min[0]), scaled(bounds.min[1])),
        Point::new(scaled(bounds.max[0]), scaled(bounds.max[1])),
    )
}

/// `BoundingBox::polygon` (`BoundingBox.cpp:20-28`).
fn polygon(bounds: BoundingBox) -> Polygon {
    let (min, max) = (bounds.min(), bounds.max());
    Polygon::new(vec![
        min,
        Point::new(max.x(), min.y()),
        max,
        Point::new(min.x(), max.y()),
    ])
}

/// `is_approx(Vec2d, Vec2d)` (`Point.hpp:411-415`).
fn approx(a: [f64; 2], b: [f64; 2]) -> bool {
    (b[0] - a[0]).abs() < EPSILON && (b[1] - a[1]).abs() < EPSILON
}

/// `same_zone` (`IMEXArrange.cpp:32-36`): each plate computes its zones from where it sits, so the
/// same zone differs in the last bits.
fn same_zone(a: BoundingBoxf, b: BoundingBoxf) -> bool {
    approx(a.min, b.min) && approx(a.max, b.max)
}

/// `inset` (`IMEXArrange.cpp:38-43`).
fn inset(bounds: BoundingBox, [dx, dy]: [Coord; 2]) -> BoundingBox {
    let (min, max) = (bounds.min(), bounds.max());
    BoundingBox::new(
        Point::new(min.x() + dx, min.y() + dy),
        Point::new(max.x() - dx, max.y() - dy),
    )
}

/// `boxes_around` (`IMEXArrange.cpp:45-61`): the parts of `bed` outside `zone`, full-depth boxes
/// left and right of it and the pieces in front of and behind it; empty pieces are left out.
fn boxes_around(bed: BoundingBox, zone: BoundingBox) -> Vec<BoundingBox> {
    let mut out = Vec::new();
    let mut add = |x0: Coord, y0: Coord, x1: Coord, y1: Coord| {
        if x1 > x0 && y1 > y0 {
            out.push(BoundingBox::new(Point::new(x0, y0), Point::new(x1, y1)));
        }
    };
    let x0 = zone.min().x().max(bed.min().x());
    let x1 = zone.max().x().min(bed.max().x());
    add(bed.min().x(), bed.min().y(), zone.min().x(), bed.max().y());
    add(zone.max().x(), bed.min().y(), bed.max().x(), bed.max().y());
    add(x0, bed.min().y(), x1, zone.min().y());
    add(x0, zone.max().y(), x1, bed.max().y());
    out
}

impl ImexArranger {
    /// `ImexArranger::ImexArranger` (`IMEXArrange.cpp:92-125`). `bed` is the arranger's bed shape,
    /// shrunk by `params.bed_shrink_x/y`.
    pub fn new(input: &ImexArrangeInput, params: &ArrangeParams, bed: &[Point]) -> Self {
        let bed_bb = BoundingBox::from_points(bed.iter().copied()).expect("bed outline");
        let margin = [
            scaled_f32(params.bed_shrink_x.max(0.0)),
            scaled_f32(params.bed_shrink_y.max(0.0)),
        ];
        let mut arranger = Self {
            bed: bed.to_vec(),
            bed_bb,
            margin,
            adds_plates: input.adds_plates,
            beds: Vec::new(),
            shared_room: None,
            keep_outs: Vec::new(),
        };
        arranger.beds = (0..)
            .zip(&input.beds)
            .filter_map(|(bed_idx, zones)| zones.clone().map(|zones| ZoneBed { bed_idx, zones }))
            .collect();

        // The bed shape can be a zone only when every plate being arranged has it, and so does
        // every plate the arrange may add.
        if let Some(Some(first)) = input.beds.first() {
            let shares =
                |zones: &ImexArrangeZones| same_zone(zones.primary_zone, first.primary_zone);
            if input
                .beds
                .iter()
                .all(|zones| zones.as_ref().is_some_and(shares))
                && (!input.adds_plates || input.added_plate.as_ref().is_some_and(shares))
            {
                arranger.shared_room = arranger.room_in(first.primary_zone);
            }
        }

        // The beds past the plates being arranged become new plates.
        if let Some(added) = input.added_plate.as_ref().filter(|_| input.adds_plates) {
            for bed_idx in input.beds.len() as i32..MAX_NUM_PLATES as i32 {
                arranger.beds.push(ZoneBed {
                    bed_idx,
                    zones: added.clone(),
                });
            }
        }

        arranger.keep_outs = arranger.keep_outs_of_beds(scaled(input.brim));
        arranger
    }

    /// The constructor's keep-outs (`IMEXArrange.cpp:116-124`): without a shared room, the fences
    /// around each zone; on every zoned bed, its strips grown by the bed margin and the brim. A
    /// zone narrower than its margins leaves no room, and its fences then fill the whole bed.
    fn keep_outs_of_beds(&self, brim: Coord) -> Vec<KeepOut> {
        let strip_margin = [-(self.margin[0] + brim), -(self.margin[1] + brim)];
        self.beds
            .iter()
            .flat_map(|zone_bed| {
                let fences = if self.shared_room.is_none() {
                    let zone = inset(scaled_box(zone_bed.zones.primary_zone), self.margin);
                    boxes_around(self.bed_bb, zone)
                } else {
                    Vec::new()
                };
                let fences = fences.into_iter().map(|bounds| (bounds, true));
                let strips = zone_bed
                    .zones
                    .collision_zones
                    .iter()
                    .map(move |strip| (inset(scaled_box(*strip), strip_margin), false));
                fences.chain(strips).map(|(bounds, fence)| KeepOut {
                    bed_idx: zone_bed.bed_idx,
                    bounds,
                    fence,
                })
            })
            .collect()
    }

    /// Whether any bed has a zone. Nothing below has an effect otherwise.
    pub fn active(&self) -> bool {
        !self.beds.is_empty()
    }

    /// `room_in` (`IMEXArrange.cpp:127-135`).
    fn room_in(&self, zone: BoundingBoxf) -> Option<BoundingBox> {
        let room = inset(scaled_box(zone), self.margin);
        let (bed_min, bed_max) = (self.bed_bb.min(), self.bed_bb.max());
        let min = Point::new(
            room.min().x().max(bed_min.x()),
            room.min().y().max(bed_min.y()),
        );
        let max = Point::new(
            room.max().x().min(bed_max.x()),
            room.max().y().min(bed_max.y()),
        );
        (min.x() < max.x() && min.y() < max.y()).then(|| BoundingBox::new(min, max))
    }

    /// The bed shape to arrange in (`IMEXArrange.cpp:137`).
    pub fn bed_shape(&self) -> Vec<Point> {
        self.shared_room
            .map_or_else(|| self.bed.clone(), |room| polygon(room).points().to_vec())
    }

    /// Adds the fixed items that keep parts out of each bed's keep-outs
    /// (`IMEXArrange.cpp:139-150`).
    pub fn add_keep_outs(&self, fixed: &mut Vec<ArrangePolygon>) {
        fixed.extend(self.keep_outs.iter().map(|keep_out| {
            ArrangePolygon {
                poly: ExPolygon::new(polygon(keep_out.bounds), Vec::new()),
                is_virt_object: true,
                bed_idx: keep_out.bed_idx,
                height: 1.0,
                name: if keep_out.fence {
                    FENCE_NAME
                } else {
                    STRIP_NAME
                }
                .to_owned(),
                ..ArrangePolygon::default()
            }
        }));
    }

    /// `crosses_keep_out` (`IMEXArrange.cpp:152-158`).
    fn crosses_keep_out(&self, part: &ArrangePolygon, bed_idx: i32) -> bool {
        let hull = part.transformed_poly().contour().clone();
        self.keep_outs.iter().any(|keep_out| {
            keep_out.bed_idx == bed_idx
                && !intersection_polygons_paths(
                    std::slice::from_ref(&hull),
                    &[polygon(keep_out.bounds)],
                )
                .expect("keep-out intersection")
                .is_empty()
        })
    }

    /// `finish` (`IMEXArrange.cpp:160-218`). Call after
    /// `arrange(items, fixed, Bed::from_points(&bed_shape()), params)`, with `before` holding
    /// `items` as they were before it. Leaves unarranged any part across a keep-out, then centres
    /// each fenced plate's parts in its zone. The Ares arrange cannot be cancelled, so upstream's
    /// stop-condition return (`:196-198`) has no counterpart.
    pub fn finish(
        &self,
        items: &mut [ArrangePolygon],
        before: &[ArrangePolygon],
        fixed: &[ArrangePolygon],
        params: &ArrangeParams,
    ) {
        for part in items.iter_mut() {
            if self.crosses_keep_out(part, part.bed_idx) {
                part.bed_idx = UNARRANGED;
            }
        }
        // Only fenced plates need centring, and arranging one plate never fences.
        if self.shared_room.is_some() || !self.adds_plates {
            return;
        }
        for zone_bed in &self.beds {
            let on_bed: Vec<usize> = (0..items.len())
                .filter(|&i| items[i].bed_idx == zone_bed.bed_idx)
                .collect();
            let Some(room) = self.room_in(zone_bed.zones.primary_zone) else {
                continue;
            };
            if on_bed.is_empty() {
                continue;
            }
            self.rearrange_in_zone(
                items,
                (before, fixed, params),
                (zone_bed.bed_idx, &on_bed, room),
            );
        }
    }

    /// `finish`'s pass over one fenced plate (`IMEXArrange.cpp:178-216`): its parts arranged again
    /// on bed 0 inside the zone, kept when every one fits.
    fn rearrange_in_zone(
        &self,
        items: &mut [ArrangePolygon],
        (before, fixed, params): (&[ArrangePolygon], &[ArrangePolygon], &ArrangeParams),
        (bed_idx, on_bed, room): (i32, &[usize], BoundingBox),
    ) {
        // Bed 0, where the nester takes a part as placeable rather than as one that does not fit.
        let mut parts: Vec<ArrangePolygon> = on_bed
            .iter()
            .map(|&i| ArrangePolygon {
                bed_idx: 0,
                ..before[i].clone()
            })
            .collect();
        let bed_fixed: Vec<ArrangePolygon> = fixed
            .iter()
            .filter(|item| item.bed_idx == bed_idx && item.name != FENCE_NAME)
            .map(|item| ArrangePolygon {
                bed_idx: 0,
                ..item.clone()
            })
            .collect();
        super::arrange(
            &mut parts,
            &bed_fixed,
            &Bed::from_points(polygon(room).points()),
            params,
        );
        // Keep the first layout when this one does not fit.
        if !parts
            .iter()
            .all(|part| part.bed_idx == 0 && !self.crosses_keep_out(part, bed_idx))
        {
            return;
        }
        // By-object printing follows the packing order, which this pass numbered 0..n-1: carry it
        // over onto the ids the first pass gave these parts.
        let mut ids: Vec<i32> = on_bed.iter().map(|&i| items[i].itemid).collect();
        ids.sort_unstable();
        for (&i, part) in on_bed.iter().zip(&parts) {
            items[i].translation = part.translation;
            items[i].rotation = part.rotation;
            items[i].itemid = ids[part.itemid as usize];
        }
    }
}
