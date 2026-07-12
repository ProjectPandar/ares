use super::FuzzySkinConfig;
use crate::Point2;

mod arachne;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallDirection {
    CounterClockwise,
    Clockwise,
}

impl WallDirection {
    pub(super) fn orient_points(self, mut points: Vec<Point2>) -> Vec<Point2> {
        match self {
            Self::CounterClockwise => points,
            Self::Clockwise => {
                points.reverse();
                points
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallSequence {
    InnerOuter,
    OuterInner,
    InnerOuterInner,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallGenerator {
    Classic,
    Arachne,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SeamPosition {
    Nearest,
    Aligned,
    AlignedBack,
    Back,
    Random,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PerimeterOptions {
    wall_loops: u32,
    external_line_width: f64,
    internal_line_width: f64,
    wall_generator: WallGenerator,
    wall_transition_length_percent: f64,
    wall_transition_filter_deviation_percent: f64,
    wall_transition_angle_degrees: f64,
    wall_distribution_count: u32,
    min_nozzle_diameter: f64,
    min_feature_size_percent: f64,
    initial_layer_min_bead_width_percent: f64,
    min_bead_width_percent: f64,
    wall_maximum_resolution_mm: f64,
    wall_maximum_deviation_mm: f64,
    min_length_factor: f64,
    precise_outer_wall: bool,
    layer_height_mm: f64,
    wall_direction: WallDirection,
    wall_sequence: WallSequence,
    seam_position: SeamPosition,
    staggered_inner_seams: bool,
    seam_gap_mm: f64,
    only_one_wall_first_layer: bool,
    only_one_wall_top: bool,
    alternate_extra_wall: bool,
    sparse_infill_density_percent: f64,
    detect_overhang_wall: bool,
    extra_perimeters_on_overhangs: bool,
    overhang_reverse: bool,
    overhang_reverse_internal_only: bool,
    overhang_reverse_threshold_mm: f64,
    make_overhang_printable: bool,
    make_overhang_printable_angle_degrees: f64,
    make_overhang_printable_hole_size_mm2: f64,
    fuzzy_skin: FuzzySkinConfig,
    detect_thin_wall: bool,
}

impl PerimeterOptions {
    pub const fn new(
        wall_loops: u32,
        external_line_width: f64,
        internal_line_width: f64,
        wall_direction: WallDirection,
        wall_sequence: WallSequence,
    ) -> Self {
        Self {
            wall_loops,
            external_line_width,
            internal_line_width,
            wall_generator: WallGenerator::Arachne,
            wall_transition_length_percent: 100.0,
            wall_transition_filter_deviation_percent: 25.0,
            wall_transition_angle_degrees: 10.0,
            wall_distribution_count: 1,
            min_nozzle_diameter: external_line_width,
            min_feature_size_percent: 25.0,
            initial_layer_min_bead_width_percent: 85.0,
            min_bead_width_percent: 85.0,
            wall_maximum_resolution_mm: 0.5,
            wall_maximum_deviation_mm: 0.025,
            min_length_factor: 0.5,
            precise_outer_wall: true,
            layer_height_mm: 0.2,
            wall_direction,
            wall_sequence,
            seam_position: SeamPosition::Aligned,
            staggered_inner_seams: false,
            seam_gap_mm: external_line_width * 0.1,
            only_one_wall_first_layer: false,
            only_one_wall_top: false,
            alternate_extra_wall: false,
            sparse_infill_density_percent: 20.0,
            detect_overhang_wall: true,
            extra_perimeters_on_overhangs: false,
            overhang_reverse: false,
            overhang_reverse_internal_only: false,
            overhang_reverse_threshold_mm: external_line_width * 0.5,
            make_overhang_printable: false,
            make_overhang_printable_angle_degrees: 55.0,
            make_overhang_printable_hole_size_mm2: 0.0,
            fuzzy_skin: FuzzySkinConfig::disabled(),
            detect_thin_wall: false,
        }
    }

    pub const fn with_only_one_wall_first_layer(mut self, only_one_wall_first_layer: bool) -> Self {
        self.only_one_wall_first_layer = only_one_wall_first_layer;
        self
    }

    pub const fn with_only_one_wall_top(mut self, only_one_wall_top: bool) -> Self {
        self.only_one_wall_top = only_one_wall_top;
        self
    }

    pub const fn with_alternate_extra_wall(mut self, alternate_extra_wall: bool) -> Self {
        self.alternate_extra_wall = alternate_extra_wall;
        self
    }

    pub const fn with_sparse_infill_density_percent(
        mut self,
        sparse_infill_density_percent: f64,
    ) -> Self {
        self.sparse_infill_density_percent = sparse_infill_density_percent;
        self
    }

    pub const fn with_precise_outer_wall(mut self, precise_outer_wall: bool) -> Self {
        self.precise_outer_wall = precise_outer_wall;
        self
    }

    pub const fn with_layer_height_mm(mut self, layer_height_mm: f64) -> Self {
        self.layer_height_mm = layer_height_mm;
        self
    }

    pub const fn with_detect_overhang_wall(mut self, detect_overhang_wall: bool) -> Self {
        self.detect_overhang_wall = detect_overhang_wall;
        self
    }

    pub const fn with_extra_perimeters_on_overhangs(
        mut self,
        extra_perimeters_on_overhangs: bool,
    ) -> Self {
        self.extra_perimeters_on_overhangs = extra_perimeters_on_overhangs;
        self
    }

    pub const fn with_overhang_reverse(mut self, overhang_reverse: bool) -> Self {
        self.overhang_reverse = overhang_reverse;
        self
    }

    pub const fn with_overhang_reverse_internal_only(
        mut self,
        overhang_reverse_internal_only: bool,
    ) -> Self {
        self.overhang_reverse_internal_only = overhang_reverse_internal_only;
        self
    }

    pub const fn with_overhang_reverse_threshold_mm(
        mut self,
        overhang_reverse_threshold_mm: f64,
    ) -> Self {
        self.overhang_reverse_threshold_mm = overhang_reverse_threshold_mm;
        self
    }

    pub const fn with_make_overhang_printable(mut self, make_overhang_printable: bool) -> Self {
        self.make_overhang_printable = make_overhang_printable;
        self
    }

    pub const fn with_make_overhang_printable_angle_degrees(
        mut self,
        make_overhang_printable_angle_degrees: f64,
    ) -> Self {
        self.make_overhang_printable_angle_degrees = make_overhang_printable_angle_degrees;
        self
    }

    pub const fn with_make_overhang_printable_hole_size_mm2(
        mut self,
        make_overhang_printable_hole_size_mm2: f64,
    ) -> Self {
        self.make_overhang_printable_hole_size_mm2 = make_overhang_printable_hole_size_mm2;
        self
    }

    pub const fn with_seam_position(mut self, seam_position: SeamPosition) -> Self {
        self.seam_position = seam_position;
        self
    }

    pub const fn with_staggered_inner_seams(mut self, staggered_inner_seams: bool) -> Self {
        self.staggered_inner_seams = staggered_inner_seams;
        self
    }

    pub const fn with_seam_gap_mm(mut self, seam_gap_mm: f64) -> Self {
        self.seam_gap_mm = seam_gap_mm;
        self
    }

    pub(crate) const fn with_fuzzy_skin(mut self, fuzzy_skin: FuzzySkinConfig) -> Self {
        self.fuzzy_skin = fuzzy_skin;
        self
    }

    pub const fn with_detect_thin_wall(mut self, detect_thin_wall: bool) -> Self {
        self.detect_thin_wall = detect_thin_wall;
        self
    }

    pub const fn with_min_length_factor(mut self, min_length_factor: f64) -> Self {
        self.min_length_factor = min_length_factor;
        self
    }

    pub const fn with_wall_generator(mut self, wall_generator: WallGenerator) -> Self {
        self.wall_generator = wall_generator;
        self
    }

    pub const fn wall_loops(&self) -> u32 {
        self.wall_loops
    }

    pub const fn external_line_width(&self) -> f64 {
        self.external_line_width
    }

    pub const fn internal_line_width(&self) -> f64 {
        self.internal_line_width
    }

    pub const fn min_length_factor(&self) -> f64 {
        self.min_length_factor
    }

    pub const fn wall_generator(&self) -> WallGenerator {
        self.wall_generator
    }

    pub const fn precise_outer_wall(&self) -> bool {
        self.precise_outer_wall
    }

    pub const fn layer_height_mm(&self) -> f64 {
        self.layer_height_mm
    }

    pub const fn wall_direction(&self) -> WallDirection {
        self.wall_direction
    }

    pub const fn wall_sequence(&self) -> WallSequence {
        self.wall_sequence
    }

    pub const fn seam_position(&self) -> SeamPosition {
        self.seam_position
    }

    pub const fn staggered_inner_seams(&self) -> bool {
        self.staggered_inner_seams
    }

    pub const fn seam_gap_mm(&self) -> f64 {
        self.seam_gap_mm
    }

    pub const fn only_one_wall_first_layer(&self) -> bool {
        self.only_one_wall_first_layer
    }

    pub const fn only_one_wall_top(&self) -> bool {
        self.only_one_wall_top
    }

    pub const fn alternate_extra_wall(&self) -> bool {
        self.alternate_extra_wall
    }

    pub const fn sparse_infill_density_percent(&self) -> f64 {
        self.sparse_infill_density_percent
    }

    pub const fn detect_overhang_wall(&self) -> bool {
        self.detect_overhang_wall
    }

    pub const fn extra_perimeters_on_overhangs(&self) -> bool {
        self.extra_perimeters_on_overhangs
    }

    pub const fn overhang_reverse(&self) -> bool {
        self.overhang_reverse
    }

    pub const fn overhang_reverse_internal_only(&self) -> bool {
        self.overhang_reverse_internal_only
    }

    pub const fn overhang_reverse_threshold_mm(&self) -> f64 {
        self.overhang_reverse_threshold_mm
    }

    pub const fn make_overhang_printable(&self) -> bool {
        self.make_overhang_printable
    }

    pub const fn make_overhang_printable_angle_degrees(&self) -> f64 {
        self.make_overhang_printable_angle_degrees
    }

    pub const fn make_overhang_printable_hole_size_mm2(&self) -> f64 {
        self.make_overhang_printable_hole_size_mm2
    }

    pub(crate) const fn fuzzy_skin(&self) -> FuzzySkinConfig {
        self.fuzzy_skin
    }

    pub const fn detect_thin_wall(&self) -> bool {
        self.detect_thin_wall
    }
}
