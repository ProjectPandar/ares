use crate::{
    Layer, LayerBrims, LayerContours, LayerGapFills, LayerInfills, LayerPerimeters, LayerSkirts,
    Point2, SliceError,
};

mod gap_fill_filter;
pub use gap_fill_filter::filter_short_gap_fill_paths;
mod generate;
#[cfg(test)]
pub(crate) use generate::finalize_print_paths;
pub use generate::generate_print_paths;
pub(crate) use generate::{
    finalize_print_paths_with_layer_contours, generate_print_paths_with_bridge_policy,
};
mod ironing;
pub(crate) use ironing::apply_ironing;
mod ironing_scanlines;
mod shell_layers;
pub use shell_layers::ShellLayerOptions;
mod support_interface;
pub(crate) use support_interface::{
    apply_raft_expansion, apply_raft_first_layer_expansion, apply_support_expansion,
    apply_support_interface_top_layers,
};
mod support_angle;
pub(crate) use support_angle::{parse_support_angle, rotated_rectangle_lines};
mod support_base_pattern_spacing;
pub(crate) use support_base_pattern_spacing::{
    SupportBaseSpacingConfig, apply_support_base_pattern_spacing,
};
mod support_critical_regions_only;
pub(crate) use support_critical_regions_only::apply_support_critical_regions_only;
mod support_interface_spacing;
pub(crate) use support_interface_spacing::{
    SupportInterfaceSpacingConfig, apply_support_interface_spacing,
};
mod support_ironing;
pub(crate) use support_ironing::apply_support_ironing;
mod support_on_build_plate_only;
pub(crate) use support_on_build_plate_only::apply_support_on_build_plate_only;
mod support_object_xy_distance;
pub(crate) use support_object_xy_distance::apply_support_object_xy_distance;
mod support_remove_small_overhang;
pub(crate) use support_remove_small_overhang::apply_support_remove_small_overhang;
mod support_rectangle;
mod support_style_snug;
pub(crate) use support_style_snug::apply_support_style_snug;
mod support_threshold_contacts;
pub(crate) use support_threshold_contacts::apply_support_threshold_contacts;
mod support_tree_brim;
pub(crate) use support_tree_brim::apply_tree_support_brim;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PrintPathRole {
    Skirt,
    Brim,
    Bridge,
    InternalBridge,
    GapFill,
    ExternalPerimeter,
    OverhangPerimeter,
    InternalPerimeter,
    SparseInfill,
    SolidInfill,
    TopSolidInfill,
    BottomSurface,
    SupportMaterial,
    SupportMaterialInterface,
    Ironing,
}

impl PrintPathRole {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Skirt => "skirt",
            Self::Brim => "brim",
            Self::Bridge => "bridge",
            Self::InternalBridge => "internal_bridge",
            Self::GapFill => "gap_fill",
            Self::ExternalPerimeter => "external_perimeter",
            Self::OverhangPerimeter => "overhang_perimeter",
            Self::InternalPerimeter => "internal_perimeter",
            Self::SparseInfill => "sparse_infill",
            Self::SolidInfill => "solid_infill",
            Self::TopSolidInfill => "top_solid_infill",
            Self::BottomSurface => "bottom_surface",
            Self::SupportMaterial => "support_material",
            Self::SupportMaterialInterface => "support_material_interface",
            Self::Ironing => "ironing",
        }
    }
}

pub(crate) const fn diagnostic_role_label(
    role: PrintPathRole,
    extrusion_role: Option<PrintPathRole>,
) -> &'static str {
    match (role, extrusion_role) {
        (PrintPathRole::Ironing, Some(PrintPathRole::SupportMaterialInterface)) => {
            "support_ironing"
        }
        _ => role.as_str(),
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PrintPathInput<'a> {
    layer_contours: Option<&'a [LayerContours]>,
    print_layers: Option<&'a [Layer]>,
    skirts: &'a [LayerSkirts],
    brims: &'a [LayerBrims],
    perimeters: &'a [LayerPerimeters],
    gap_fills: &'a [LayerGapFills],
    infills: &'a [LayerInfills],
}

impl<'a> PrintPathInput<'a> {
    pub const fn new(
        skirts: &'a [LayerSkirts],
        brims: &'a [LayerBrims],
        perimeters: &'a [LayerPerimeters],
        gap_fills: &'a [LayerGapFills],
        infills: &'a [LayerInfills],
    ) -> Self {
        Self {
            layer_contours: None,
            print_layers: None,
            skirts,
            brims,
            perimeters,
            gap_fills,
            infills,
        }
    }

    pub const fn with_layer_contours(mut self, layer_contours: &'a [LayerContours]) -> Self {
        self.layer_contours = Some(layer_contours);
        self
    }

    pub const fn with_print_layers(mut self, print_layers: &'a [Layer]) -> Self {
        self.print_layers = Some(print_layers);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PrintPath {
    role: PrintPathRole,
    extrusion_role: Option<PrintPathRole>,
    points: Vec<Point2>,
    effective_layer_height_mm: Option<f64>,
    effective_line_width_mm: Option<f64>,
    unsupported_span_mm: Option<f64>,
    seam_gap_mm: f64,
    closed: bool,
}

impl PrintPath {
    pub fn new(role: PrintPathRole, points: Vec<Point2>) -> Result<Self, SliceError> {
        if points.is_empty() {
            return Err(SliceError::InvalidInput(
                "print path requires at least one point".to_owned(),
            ));
        }
        Ok(Self {
            role,
            extrusion_role: None,
            points,
            effective_layer_height_mm: None,
            effective_line_width_mm: None,
            unsupported_span_mm: None,
            seam_gap_mm: 0.0,
            closed: default_closed(role),
        })
    }

    pub const fn role(&self) -> PrintPathRole {
        self.role
    }

    pub(crate) const fn extrusion_role(&self) -> Option<PrintPathRole> {
        self.extrusion_role
    }

    pub(crate) const fn with_extrusion_role(mut self, extrusion_role: PrintPathRole) -> Self {
        self.extrusion_role = Some(extrusion_role);
        self
    }

    pub fn points(&self) -> &[Point2] {
        &self.points
    }

    pub fn with_effective_layer_height_mm(mut self, effective_layer_height_mm: f64) -> Self {
        self.effective_layer_height_mm = Some(effective_layer_height_mm);
        self
    }

    pub const fn effective_layer_height_mm(&self) -> Option<f64> {
        self.effective_layer_height_mm
    }

    pub const fn with_effective_line_width_mm(
        mut self,
        effective_line_width_mm: Option<f64>,
    ) -> Self {
        self.effective_line_width_mm = effective_line_width_mm;
        self
    }

    pub const fn effective_line_width_mm(&self) -> Option<f64> {
        self.effective_line_width_mm
    }

    pub const fn with_unsupported_span_mm(mut self, unsupported_span_mm: Option<f64>) -> Self {
        self.unsupported_span_mm = unsupported_span_mm;
        self
    }

    pub const fn unsupported_span_mm(&self) -> Option<f64> {
        self.unsupported_span_mm
    }

    pub const fn with_seam_gap_mm(mut self, seam_gap_mm: f64) -> Self {
        self.seam_gap_mm = seam_gap_mm;
        self
    }

    pub const fn seam_gap_mm(&self) -> f64 {
        self.seam_gap_mm
    }

    pub const fn with_closed(mut self, closed: bool) -> Self {
        self.closed = closed;
        self
    }

    pub const fn is_closed(&self) -> bool {
        self.closed
    }

    /// Closing move target honoring the seam gap: the interpolated point along
    /// the closing segment where extrusion stops short of the path start.
    pub(crate) fn closing_target(&self) -> Option<Point2> {
        let start = self.points[0];
        let end = *self.points.last()?;
        let length = ((end.x() - start.x()).powi(2) + (end.y() - start.y()).powi(2)).sqrt();
        if length <= f64::EPSILON {
            return None;
        }
        if self.seam_gap_mm <= 0.0 {
            return Some(start);
        }
        if self.seam_gap_mm >= length {
            return None;
        }
        let ratio = (length - self.seam_gap_mm) / length;
        Some(Point2::new(
            end.x() + (start.x() - end.x()) * ratio,
            end.y() + (start.y() - end.y()) * ratio,
        ))
    }
}

const fn default_closed(role: PrintPathRole) -> bool {
    matches!(
        role,
        PrintPathRole::Skirt
            | PrintPathRole::Brim
            | PrintPathRole::ExternalPerimeter
            | PrintPathRole::OverhangPerimeter
            | PrintPathRole::InternalPerimeter
    )
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayerPrintPaths {
    layer_id: usize,
    print_z: f64,
    paths: Vec<PrintPath>,
}

impl LayerPrintPaths {
    pub fn new(layer_id: usize, print_z: f64, paths: Vec<PrintPath>) -> Self {
        Self {
            layer_id,
            print_z,
            paths,
        }
    }

    pub const fn layer_id(&self) -> usize {
        self.layer_id
    }

    pub const fn print_z(&self) -> f64 {
        self.print_z
    }

    pub fn paths(&self) -> &[PrintPath] {
        &self.paths
    }
}

#[cfg(test)]
mod tests;
