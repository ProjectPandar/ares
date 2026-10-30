//! OrcaSlicer's arrangement kernel at v2.5.0-beta (`345efec64`): `src/libslic3r/Arrange.hpp`
//! and `Arrange.cpp` over the libnest2d first-fit / NFP placer and NLopt 2.5.0 subplex. See
//! `docs/architecture/web-plater-move-rotate-arrange.md`, "Auto-arrange on OrcaSlicer
//! v2.5.0-beta".

pub use crate::geometry::{BoundingBox, ExPolygon, Point, Polygon};

/// Scaled coordinate (`coord_t`).
pub type Coord = i64;

/// The logical bed of an item arrange has not placed (`Arrange.hpp:48`).
pub const UNARRANGED: i32 = -1;

/// `MAX_OUTER_NOZZLE_DIAMETER` (`Print.hpp:72`), in mm.
pub const MAX_OUTER_NOZZLE_DIAMETER: f64 = 4.0;

/// `SCALING_FACTOR_INTERNAL` (`libslic3r.h:65`): the scale of a printer below 2147 mm.
pub const SCALING_FACTOR: f64 = 0.000_001;

/// `ArrangePolygon` (`Arrange.hpp:58-124`): an item's silhouette and its placement. `poly` is
/// not modified; `translation`, `rotation` and `bed_idx` carry the result.
#[derive(Clone, Debug, PartialEq)]
pub struct ArrangePolygon {
    pub poly: ExPolygon,
    pub translation: Point,
    /// Radians.
    pub rotation: f64,
    pub inflation: Coord,
    pub bed_idx: i32,
    pub priority: i32,
    pub locked_plate: i32,
    pub is_virt_object: bool,
    pub is_extrusion_cali_object: bool,
    pub is_wipe_tower: bool,
    pub has_tree_support: bool,
    pub row: i32,
    pub col: i32,
    /// Logical filament ids, for the least filament switches.
    pub extrude_ids: Vec<i32>,
    pub filament_temp_type: i32,
    pub bed_temp: i32,
    pub print_temp: i32,
    pub first_bed_temp: i32,
    pub first_print_temp: i32,
    pub vitrify_temp: i32,
    pub itemid: i32,
    pub is_applied: i32,
    /// mm.
    pub height: f64,
    pub z_min: f64,
    pub z_max: f64,
    pub has_z_range: bool,
    pub is_bed_exclusion: bool,
    /// Physical nozzles for per-nozzle exclusions; empty is unresolved and applies to every one.
    pub bed_exclusion_extruder_ids: Vec<i32>,
    /// The zero-based nozzle owning an exclusion obstacle; -1 is shared.
    pub bed_exclusion_extruder_id: i32,
    /// mm.
    pub brim_width: f64,
    pub name: String,
    /// Radians; only a zero allows no rotation.
    pub allowed_rotations: Vec<f64>,
}

impl Default for ArrangePolygon {
    fn default() -> Self {
        Self {
            poly: ExPolygon::new(Polygon::new(Vec::new()), Vec::new()),
            translation: Point::new(0, 0),
            rotation: 0.0,
            inflation: 0,
            bed_idx: UNARRANGED,
            priority: 0,
            locked_plate: -1,
            is_virt_object: false,
            is_extrusion_cali_object: false,
            is_wipe_tower: false,
            has_tree_support: false,
            row: 0,
            col: 0,
            extrude_ids: Vec::new(),
            filament_temp_type: -1,
            bed_temp: 0,
            print_temp: 0,
            first_bed_temp: 0,
            first_print_temp: 0,
            vitrify_temp: 0,
            itemid: 0,
            is_applied: 0,
            height: 0.0,
            z_min: 0.0,
            z_max: 0.0,
            has_z_range: false,
            is_bed_exclusion: false,
            bed_exclusion_extruder_ids: Vec::new(),
            bed_exclusion_extruder_id: -1,
            brim_width: 0.0,
            name: String::new(),
            allowed_rotations: vec![0.0],
        }
    }
}

impl ArrangePolygon {
    /// Whether arrange placed the item on a bed (`is_arranged`).
    pub fn is_arranged(&self) -> bool {
        self.bed_idx != UNARRANGED
    }

    /// `poly` rotated by `rotation` and moved by `translation` (`transformed_poly`).
    pub fn transformed_poly(&self) -> ExPolygon {
        todo!("ArrangePolygon::transformed_poly")
    }
}

/// `ArrangeParams` (`Arrange.hpp:128-205`) without the GUI callbacks and the belt fields
/// (belt arrangement is deferred).
#[derive(Clone, Debug, PartialEq)]
pub struct ArrangeParams {
    pub min_obj_distance: Coord,
    pub accuracy: f32,
    pub parallel: bool,
    pub allow_rotations: bool,
    pub do_final_align: bool,
    pub allow_multi_materials_on_same_plate: bool,
    pub avoid_extrusion_cali_region: bool,
    pub is_seq_print: bool,
    pub align_to_y_axis: bool,
    pub bed_shrink_x: f32,
    pub bed_shrink_y: f32,
    pub brim_skirt_distance: f32,
    pub clearance_height_to_rod: f32,
    pub clearance_height_to_lid: f32,
    pub clearance_radius: f32,
    pub object_skirt_offset: f32,
    pub nozzle_height: f32,
    pub printable_height: f32,
    /// `best_object_pos`: the pile's centre as a fraction of the bed.
    pub align_center: [f64; 2],
    pub excluded_regions: Vec<ArrangePolygon>,
    pub nonprefered_regions: Vec<ArrangePolygon>,
    /// `SCALING_FACTOR` of the coordinates (mm per unit).
    pub scaling_factor: f64,
}

impl Default for ArrangeParams {
    fn default() -> Self {
        Self {
            min_obj_distance: 0,
            accuracy: 1.0,
            parallel: true,
            allow_rotations: false,
            do_final_align: true,
            allow_multi_materials_on_same_plate: true,
            avoid_extrusion_cali_region: true,
            is_seq_print: false,
            align_to_y_axis: false,
            bed_shrink_x: 1.0,
            bed_shrink_y: 1.0,
            brim_skirt_distance: 0.0,
            clearance_height_to_rod: 0.0,
            clearance_height_to_lid: 0.0,
            clearance_radius: 0.0,
            object_skirt_offset: 0.0,
            nozzle_height: 0.0,
            printable_height: 256.0,
            align_center: [0.5, 0.5],
            excluded_regions: Vec::new(),
            nonprefered_regions: Vec::new(),
            scaling_factor: SCALING_FACTOR,
        }
    }
}

/// What the parameter updates read from the print config: `get_bed_shape(cfg)`
/// (`PrintConfig.cpp:13837`) and `get_real_skirt_dist(cfg)` (`:14413`).
#[derive(Clone, Debug, PartialEq)]
pub struct ArrangeConfig {
    pub bed_shape: Vec<Point>,
    /// mm.
    pub real_skirt_distance: f64,
}

/// The bed shapes `arrange` packs into (`Arrange.hpp:23-45`, `call_with_bed`).
#[derive(Clone, Debug, PartialEq)]
pub enum Bed {
    Box(BoundingBox),
    Circle { center: Point, radius: f64 },
    Polygon(Polygon),
    Infinite { center: Point },
}

impl Bed {
    /// A bed from its outline points, as `arrange(items, excludes, Points, params)` dispatches
    /// (`Arrange.cpp:1259-1288`): a rectangle, a circle or a polygon.
    pub fn from_points(_points: &[Point]) -> Self {
        todo!("Bed::from_points")
    }
}

/// `update_arrange_params` (`Arrange.cpp:148-161`).
pub fn update_arrange_params(
    _params: &mut ArrangeParams,
    _config: &ArrangeConfig,
    _selected: &[ArrangePolygon],
) {
    todo!("update_arrange_params")
}

/// `update_selected_items_inflation` (`Arrange.cpp:163-211`).
pub fn update_selected_items_inflation(
    _selected: &mut [ArrangePolygon],
    _config: &ArrangeConfig,
    _params: &mut ArrangeParams,
) {
    todo!("update_selected_items_inflation")
}

/// `update_unselected_items_inflation` (`Arrange.cpp:213-232`).
pub fn update_unselected_items_inflation(
    _unselected: &mut [ArrangePolygon],
    _config: &ArrangeConfig,
    _params: &ArrangeParams,
) {
    todo!("update_unselected_items_inflation")
}

/// `update_selected_items_axis_align` (`Arrange.cpp:234-337`).
pub fn update_selected_items_axis_align(
    _selected: &mut [ArrangePolygon],
    _config: &ArrangeConfig,
    _params: &ArrangeParams,
) {
    todo!("update_selected_items_axis_align")
}

/// `get_shrink_bedpts` (`Arrange.cpp:339-353`).
pub fn get_shrink_bedpts(_config: &ArrangeConfig, _params: &ArrangeParams) -> Vec<Point> {
    todo!("get_shrink_bedpts")
}

/// `has_bed_exclusion_regions` (`Arrange.cpp:136-141`).
pub fn has_bed_exclusion_regions(_polygons: &[ArrangePolygon]) -> bool {
    todo!("has_bed_exclusion_regions")
}

/// `bed_exclusion_applies` (`Arrange.cpp:108-146`).
pub fn bed_exclusion_applies(_item: &ArrangePolygon, _fixed_item: &ArrangePolygon) -> bool {
    todo!("bed_exclusion_applies")
}

/// `arrangement::arrange` (`Arrange.cpp:1279-1321`): places `items` on `bed` and the virtual
/// beds after it around the fixed `excludes`, setting each item's `translation`, `rotation` and
/// `bed_idx`; an item that fits no bed keeps its input and stays `UNARRANGED`.
pub fn arrange(
    _items: &mut [ArrangePolygon],
    _excludes: &[ArrangePolygon],
    _bed: &Bed,
    _params: &ArrangeParams,
) {
    todo!("arrangement::arrange")
}
