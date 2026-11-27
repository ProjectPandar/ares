//! BeltTransformPipeline and the bounded belt print admission.

pub(super) mod back_transform;
pub(super) mod kinematics;

use crate::{
    BeltRotationAxis, Point3d, ProcessDraftShield, ProcessPrintSequence, ProcessSupportStyle,
    ProcessSupportType, Project, ProjectVolumeType, RemapAxis, SliceError,
    geometry::CoordinateScale,
    options::PrinterRemainingOptions,
    project::{Transform3d, effective_config::types::BoundedResolvedProjectConfig},
};

pub(super) const EPSILON: f64 = 1e-4;

pub(super) mod brim;
pub(super) mod floor_context;

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
    // The admitted single slot cannot activate the planner (BeltPurge.cpp:48–61).
    if resolved.logical_filament_count != 1
        || resolved
            .usage
            .supported_used_filaments
            .iter()
            .any(|&slot| slot != 0)
    {
        return Err(unsupported("materials"));
    }

    let options = &object.object;
    if options.raft_layers.0 != 0
        || options.enforce_support_layers.0 != 0
        || (options.enable_support.0 && !normal_grid_support(settings, options, source))
    {
        return Err(unsupported("support"));
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
    let axis = printer.belt_slice_rotation;
    let angle = printer.belt_slice_rotation_angle.0;
    let sin = angle.to_radians().sin();
    if !matches!(axis, BeltRotationAxis::X | BeltRotationAxis::Y)
        || !angle.is_finite()
        || angle.abs() <= EPSILON
        || sin.abs() <= EPSILON
    {
        return Err(unsupported("belt_slice_rotation"));
    }

    let [x, y, z] = remap_axes(printer).map(|(index, _)| index);
    if x == y || x == z || y == z {
        return Err(unsupported("gcode_remap"));
    }
    Ok(())
}

fn normal_grid_support(
    settings: &crate::ProjectSettings,
    object: &crate::ObjectOptions,
    source: &crate::ProjectObject,
) -> bool {
    let printer = &settings.printer.remaining;
    let (sin, cos) = back_transform::deg2rad(printer.belt_slice_rotation_angle.0).sin_cos();
    cos.abs() > EPSILON
        && (sin / cos).abs() >= EPSILON
        && object.support_type == ProcessSupportType::NormalAuto
        && matches!(
            object.support_style,
            ProcessSupportStyle::Grid | ProcessSupportStyle::Default
        )
        && source.volumes()[0]
            .mesh()
            .supported_facets
            .mapping
            .is_empty()
        && !brim::enabled(settings, object)
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
            let (sin, cos) = back_transform::deg2rad(printer.belt_slice_rotation_angle.0).sin_cos();
            Self {
                axis: printer.belt_slice_rotation,
                sin,
                cos,
            }
        })
    }

    pub(super) fn forward(self) -> Transform3d {
        let (sin, cos) = (self.sin, self.cos);
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
    pub(super) floor: Option<BeltFloor>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct BeltFloor {
    axis: usize,
    shear: f64,
    shift: f64,
}

impl BeltFloor {
    pub(super) fn height(self, xy: [f64; 2], z: f64, width: f64) -> f64 {
        z - (self.shear * xy[self.axis] + self.shift) - 0.5 * width * self.shear.abs()
    }
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
        // PrintApply.cpp:213–214 adds to Z using the one-based printable instance ordinal.
        let instance_ordinal = 1.;
        let centered = group
            .transform
            .without_xy_translation()
            .pretranslated(Point3d::new(-center.x, -center.y, 1e-10 * instance_ordinal));
        let volume = &source.volumes()[0];
        let local = centered.then(volume.transform());
        let forward = rotation.forward();
        let rotated = forward.then(centered);
        let rotated_volume = rotated.then(volume.transform());
        let mut lower = [f64::INFINITY; 3];
        let mut upper = [f64::NEG_INFINITY; 3];
        let (axis, shear) = match rotation.axis {
            BeltRotationAxis::X => (1, rotation.sin / rotation.cos),
            BeltRotationAxis::Y => (0, -rotation.sin / rotation.cos),
            _ => unreachable!("belt admission requires a tilted axis"),
        };
        let floor = (rotation.cos.abs() > EPSILON && shear.abs() >= EPSILON).then_some(BeltFloor {
            axis,
            shear,
            shift: 0.,
        });
        let floor_z = |p: Point3d| floor.map_or(0., |floor| floor.shear * [p.x, p.y][floor.axis]);
        let margin = floor.map_or(0., |floor| 10. * floor.shear.abs());
        let mut min_z = f64::INFINITY;
        for &vertex in volume.mesh().vertices() {
            let point = local.transform_point(vertex);
            for (i, value) in [point.x, point.y, point.z].into_iter().enumerate() {
                lower[i] = lower[i].min(value);
                upper[i] = upper[i].max(value);
            }
            let rotated_point = rotated_volume.transform_point(vertex);
            min_z = min_z.min(rotated_point.z).min(floor_z(rotated_point));
        }
        let lift = (margin - min_z).max(0.);
        // BeltTransform.cpp:151–168 estimates the below-footprint floor from
        // the eight bbox corners, while BeltSliceStrategy.cpp scans the mesh.
        let (mut bottom, mut top) = (f64::INFINITY, f64::NEG_INFINITY);
        let bounds = [lower, upper];
        for corner in 0..8 {
            let [x, y, z] = std::array::from_fn(|i| bounds[(corner >> i) & 1][i]);
            let point = forward.transform_point(Point3d::new(x, y, z));
            bottom = bottom.min(point.z).min(floor_z(point));
            top = top.max(point.z);
        }
        // PrintObjectSlice.cpp:995–1038. The instance shift includes the center;
        // its contribution and the explicit centering correction sum to this.
        let placed = source.instances()[group.instance_indices[0]]
            .transform()
            .transform_point(Point3d::new(0., 0., 0.));
        let placement =
            forward.transform_point(Point3d::new(placed.x + center.x, placed.y + center.y, 0.));
        let correction = placement.z - lift;
        let print_z_offset = if correction.abs() > EPSILON {
            correction
        } else {
            0.
        };
        // PrintObjectSlice.cpp:910–921,1035–1041: lift the plane, then
        // shift it by the same correction as Layer::print_z.
        Ok(Self {
            height: top - (bottom - margin),
            centered_transform: rotated.pretranslated(Point3d::new(0., 0., lift)),
            print_z_offset,
            floor: floor.map(|floor| BeltFloor {
                shift: lift + print_z_offset,
                ..floor
            }),
        })
    }
}
