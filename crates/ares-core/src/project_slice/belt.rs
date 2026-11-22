//! BeltTransformPipeline and the bounded belt print admission.

use crate::{
    BeltRotationAxis, Point3d, ProcessBrimType, ProcessDraftShield, ProcessPrintSequence, Project,
    ProjectVolumeType, RemapAxis, SliceError,
    geometry::CoordinateScale,
    options::PrinterRemainingOptions,
    project::{Transform3d, effective_config::types::BoundedResolvedProjectConfig},
};

const EPSILON: f64 = 1e-4;

fn unsupported(feature: &str) -> SliceError {
    SliceError::UnsupportedProjectFeature(format!("belt_printer: {feature}"))
}

pub(super) fn admit(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
) -> Result<(), SliceError> {
    let settings = &resolved.views.full;
    let printer = &settings.printer.remaining;
    if !printer.belt_printer.0 {
        return Ok(());
    }
    let mut objects = resolved
        .objects
        .iter()
        .filter(|object| !object.print_objects.is_empty());
    let object = objects.next().ok_or_else(|| unsupported("instances"))?;
    if objects.next().is_some()
        || !matches!(object.print_objects.as_slice(), [group] if group.instance_indices.len() == 1)
        || settings.process.print.print_sequence != ProcessPrintSequence::ByLayer
    {
        return Err(unsupported("instances"));
    }
    let source = &project.objects()[object.source_object_index];
    if !matches!(source.volumes(), [volume] if volume.volume_type() == ProjectVolumeType::ModelPart)
    {
        return Err(unsupported("volumes"));
    }
    if resolved.logical_filament_count != 1
        || resolved
            .usage
            .supported_used_filaments
            .iter()
            .any(|&slot| slot != 0)
    {
        return Err(unsupported("materials"));
    }
    if object.object.brim_type != ProcessBrimType::NoBrim {
        return Err(unsupported("brim"));
    }
    if object.object.enable_support.0
        || object.object.raft_layers.0 != 0
        || object.object.enforce_support_layers.0 != 0
    {
        return Err(unsupported("support"));
    }
    if printer.enable_belt_purge_tower.0
        || project
            .documents()
            .project_settings_raw
            .get("belt_purge_tower_object")
            .is_some_and(|value| {
                use crate::project::raw_settings::RawConfigValue;
                match value {
                    RawConfigValue::Text(text) => text == "1",
                    RawConfigValue::Flag(flag) => *flag,
                    RawConfigValue::Number(number) => number.as_i64() == Some(1),
                    _ => false,
                }
            })
    {
        return Err(unsupported("belt_purge"));
    }
    if settings.process.print.skirt_loops.0 != 0
        || settings.process.print.draft_shield != ProcessDraftShield::Disabled
    {
        return Err(unsupported("skirt"));
    }
    if settings.process.print.spiral_mode.0 {
        return Err(unsupported("spiral_mode"));
    }
    if object.all_regions.iter().any(|region| region.zaa_enabled.0) {
        return Err(unsupported("contoured_toolpaths"));
    }
    if printer.belt_frame_tilt_decouple.0 {
        return Err(unsupported("belt_frame_tilt_decouple"));
    }
    let axis = printer.belt_slice_rotation;
    let angle = printer.belt_slice_rotation_angle.0;
    let (sin, cos) = angle.to_radians().sin_cos();
    if !matches!(axis, BeltRotationAxis::X | BeltRotationAxis::Y)
        || !angle.is_finite()
        || angle.abs() <= EPSILON
        || sin.abs() <= EPSILON
    {
        return Err(unsupported("belt_slice_rotation"));
    }
    // BeltTransform.cpp:94–108 and GCode.cpp:11629–11632.
    if cos.abs() > EPSILON && (sin / cos).abs() >= EPSILON {
        return Err(unsupported("first_layer_bands"));
    }
    let [x, y, z] = remap_axes(printer).map(|(index, _)| index);
    if x == y || x == z || y == z {
        return Err(unsupported("gcode_remap"));
    }
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) struct BeltRotation {
    axis: BeltRotationAxis,
    sin: f64,
    cos: f64,
}

impl BeltRotation {
    pub(super) fn from_printer(printer: &PrinterRemainingOptions) -> Option<Self> {
        printer.belt_printer.0.then(|| {
            let (sin, cos) = printer.belt_slice_rotation_angle.0.to_radians().sin_cos();
            Self {
                axis: printer.belt_slice_rotation,
                sin,
                cos,
            }
        })
    }

    pub(super) fn forward(self) -> Transform3d {
        self.transform(self.sin)
    }

    pub(super) fn inverse(self) -> Transform3d {
        self.transform(-self.sin)
    }

    fn transform(self, sin: f64) -> Transform3d {
        let cos = self.cos;
        Transform3d::from_rows(match self.axis {
            BeltRotationAxis::X => [
                [1., 0., 0., 0.],
                [0., cos, -sin, 0.],
                [0., sin, cos, 0.],
                [0., 0., 0., 1.],
            ],
            BeltRotationAxis::Y => [
                [cos, 0., sin, 0.],
                [0., 1., 0., 0.],
                [-sin, 0., cos, 0.],
                [0., 0., 0., 1.],
            ],
            _ => unreachable!("belt admission requires a tilted axis"),
        })
    }

    pub(super) fn machine_frame(self, [x, y, z]: [f64; 3]) -> [f64; 3] {
        let cot = self.cos / self.sin;
        let scale = self.sin.abs();
        match self.axis {
            BeltRotationAxis::X => [x, y / scale, z + cot * y],
            BeltRotationAxis::Y => [x / scale, y, z - cot * x],
            _ => unreachable!("belt admission requires a tilted axis"),
        }
    }
}

pub(super) fn remap_axes(printer: &PrinterRemainingOptions) -> [(usize, RemapAxis); 3] {
    [
        printer.gcode_remap_x,
        printer.gcode_remap_y,
        printer.gcode_remap_z,
    ]
    .map(|axis| {
        let index = match axis {
            RemapAxis::PosX | RemapAxis::NegX | RemapAxis::RevX => 0,
            RemapAxis::PosY | RemapAxis::NegY | RemapAxis::RevY => 1,
            RemapAxis::PosZ | RemapAxis::NegZ | RemapAxis::RevZ => 2,
        };
        (index, axis)
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BeltSlice {
    pub(super) height: f64,
    pub(super) centered_transform: Transform3d,
    pub(super) print_z_offset: f64,
}

impl BeltSlice {
    pub(super) fn new(
        rotation: BeltRotation,
        source: &crate::ProjectObject,
        group: &crate::project::effective_config::types::ResolvedPrintObjectConfig,
        scale: CoordinateScale,
    ) -> Result<Self, SliceError> {
        let center = super::centering::center(source, group, scale)?;
        let center = Point3d::new(scale.unscale(center.x()), scale.unscale(center.y()), 0.);
        let centered = group
            .transform
            .without_xy_translation()
            .pretranslated(Point3d::new(-center.x, -center.y, 0.));
        let volume = &source.volumes()[0];
        let local = centered.then(volume.transform());
        let forward = rotation.forward();
        let rotated = forward.then(centered);
        let rotated_volume = rotated.then(volume.transform());
        let mut lower = [f64::INFINITY; 3];
        let mut upper = [f64::NEG_INFINITY; 3];
        let mut min_z: f64 = 0.;
        for &vertex in volume.mesh().vertices() {
            let point = local.transform_point(vertex);
            for (i, value) in [point.x, point.y, point.z].into_iter().enumerate() {
                lower[i] = lower[i].min(value);
                upper[i] = upper[i].max(value);
            }
            min_z = min_z.min(rotated_volume.transform_point(vertex).z);
        }
        // BeltTransform.cpp:151–168: estimate the plan from the eight bbox corners,
        // whereas BeltSliceStrategy.cpp:54–75 lifts the actual rotated mesh.
        let (mut bottom, mut top) = (0.0_f64, f64::NEG_INFINITY);
        let bounds = [lower, upper];
        for corner in 0..8 {
            let [x, y, z] = std::array::from_fn(|i| bounds[(corner >> i) & 1][i]);
            let z = forward.transform_point(Point3d::new(x, y, z)).z;
            bottom = bottom.min(z);
            top = top.max(z);
        }
        // PrintObjectSlice.cpp:995–1038. The instance shift includes the center;
        // its contribution and the explicit centering correction sum to this.
        let placed = source.instances()[group.instance_indices[0]]
            .transform()
            .transform_point(Point3d::new(0., 0., 0.));
        let placement =
            forward.transform_point(Point3d::new(placed.x + center.x, placed.y + center.y, 0.));
        Ok(Self {
            height: top - bottom,
            centered_transform: rotated.pretranslated(Point3d::new(0., 0., -min_z)),
            print_z_offset: min_z + placement.z,
        })
    }
}
