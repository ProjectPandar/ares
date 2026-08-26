use crate::{
    FloatOrPercent, ObjectOptions, OrcaBool, OrcaFloat, OrcaFloats, OrcaInt, Percent,
    RegionOptions,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        compensation::{PostCompensationPrintObject, apply_project_compensation},
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface},
    },
};

use super::super::support::{identity_resolved, object_options, region};

pub(super) struct Case {
    pub(super) object: PostCompensationPrintObject,
    pub(super) resolved: ResolvedProjectObject,
}

#[derive(Debug, PartialEq)]
pub(super) struct Snapshot {
    geometry: Vec<Vec<Vec<ExPolygon>>>,
    layers: Vec<Vec<[u64; 4]>>,
    objects: Vec<ObjectOptions>,
    regions: Vec<Vec<RegionOptions>>,
}

pub(super) fn flow_options() -> (RegionOptions, ObjectOptions) {
    let mut region = region();
    region.outer_wall_line_width = FloatOrPercent::Float(0.42);
    region.inner_wall_line_width = FloatOrPercent::Float(0.45);
    region.internal_solid_infill_line_width = FloatOrPercent::Float(0.42);
    region.bridge_line_width = FloatOrPercent::Percent(Percent(100.0));
    region.bridge_flow = OrcaFloat(1.0);
    region.outer_wall_filament_id = OrcaInt(1);
    region.inner_wall_filament_id = OrcaInt(1);
    region.internal_solid_filament_id = OrcaInt(1);
    let mut object = object_options();
    object.line_width = FloatOrPercent::Float(0.42);
    object.thick_bridges = OrcaBool(false);
    (region, object)
}

pub(super) fn case(
    source_object_index: usize,
    region: RegionOptions,
    object: ObjectOptions,
    layers: &[(f64, usize)],
    scale: CoordinateScale,
) -> Case {
    let mut resolved = identity_resolved(source_object_index);
    resolved.object = object;
    let mut print_z = 0.0;
    let mut planned = Vec::with_capacity(layers.len());
    let mut region_layers = Vec::with_capacity(layers.len());
    for (id, &(height, surface_count)) in layers.iter().enumerate() {
        let slice_z = print_z + 0.5 * height;
        print_z += height;
        planned.push(PlannedLayer {
            id,
            height,
            print_z,
            slice_z,
        });
        region_layers.push(RegionLayer {
            surfaces: (0..surface_count)
                .map(|surface| RegionSurface::internal(rectangle(source_object_index, id, surface)))
                .collect(),
        });
    }
    let post_region = PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index,
            transform_index: 0,
            layers: planned,
        },
        volume_slices: Vec::new(),
        regions: vec![PostRegion {
            id: 0,
            options: region,
            layers: region_layers,
        }],
    };
    let mut objects = apply_project_compensation(
        vec![post_region],
        std::slice::from_ref(&resolved),
        FloatOrPercent::Float(0.5),
        &OrcaFloats(vec![OrcaFloat(0.4), OrcaFloat(0.4)]),
        scale,
    )
    .unwrap();
    Case {
        object: objects.pop().unwrap(),
        resolved,
    }
}

pub(super) fn split(
    cases: Vec<Case>,
) -> (Vec<PostCompensationPrintObject>, Vec<ResolvedProjectObject>) {
    cases
        .into_iter()
        .map(|case| (case.object, case.resolved))
        .unzip()
}

pub(super) fn snapshot(
    objects: &[PostCompensationPrintObject],
    resolved: &[ResolvedProjectObject],
) -> Snapshot {
    let mut geometry = Vec::with_capacity(objects.len());
    let mut layers = Vec::with_capacity(objects.len());
    let mut regions = Vec::with_capacity(objects.len());
    for object in objects {
        let (post_region, lslices) = object.as_parts();
        let (plan, _, object_regions) = post_region.as_parts();
        layers.push(
            plan.layers
                .iter()
                .map(|layer| {
                    [
                        layer.id as u64,
                        layer.height.to_bits(),
                        layer.print_z.to_bits(),
                        layer.slice_z.to_bits(),
                    ]
                })
                .collect(),
        );
        regions.push(
            object_regions
                .iter()
                .map(|region| region.as_parts().1.clone())
                .collect(),
        );
        geometry.push(
            object_regions
                .iter()
                .flat_map(|region| &region.layers)
                .map(|layer| {
                    layer
                        .surfaces
                        .iter()
                        .map(|surface| surface.as_parts().1.clone())
                        .collect()
                })
                .chain(lslices.iter().cloned())
                .collect(),
        );
    }
    Snapshot {
        geometry,
        layers,
        objects: resolved.iter().map(|item| item.object.clone()).collect(),
        regions,
    }
}

fn rectangle(source: usize, layer: usize, surface: usize) -> ExPolygon {
    let base = (source * 1_000 + layer * 100 + surface * 10) as i64;
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(base, base),
            Point::new(base + 8, base),
            Point::new(base + 8, base + 6),
            Point::new(base, base + 6),
        ]),
        Vec::new(),
    )
}
