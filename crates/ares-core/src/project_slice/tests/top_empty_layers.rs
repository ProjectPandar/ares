use crate::{
    ProjectVolumeType, Transform3d,
    geometry::{ExPolygon, Point, Polygon},
    load_project,
    mesh_slicer::SlicingMode,
    task22j_browser_oracle, task22k_browser_input_oracle, task22k_browser_oracle,
};

use super::{
    super::{
        closing::{PostClosingLayer, PostClosingPrintObject, PostClosingVolume},
        layers::{PlannedLayer, PlannedPrintObject},
        region_slices::{
            PostRegion, PostRegionPrintObject, RegionLayer, RegionSurface, prepare_region_slices,
        },
        top_empty_layers::remove_project_top_empty_layers,
        volume_bounds::build_volume_bounds,
        volume_regions::VolumeRegionGraph,
    },
    region_fixture::checkpoint,
    support::{
        KsrArchive, object as project_object, plan, project_volume, region, resolved_object,
    },
};

#[test]
fn task22k_top_empty_layers_remove_only_maximal_suffix_and_preserve_ids() {
    let mut objects = [object(
        &[11, 20, 42, 99, 123],
        &[
            &[true, false, false, false, false],
            &[false, false, true, false, false],
        ],
    )];

    remove_project_top_empty_layers(&mut objects);
    assert_eq!(layer_ids(&objects[0]), vec![11, 20, 42]);
    assert_eq!(
        region_occupancy(&objects[0]),
        vec![vec![true, false, false], vec![false, false, true]]
    );

    remove_project_top_empty_layers(&mut objects);
    assert_eq!(layer_ids(&objects[0]), vec![11, 20, 42]);
    assert_eq!(
        region_occupancy(&objects[0]),
        vec![vec![true, false, false], vec![false, false, true]]
    );
}

#[test]
fn task22k_top_empty_layers_treat_surface_container_as_nonempty() {
    let mut object = object(&[4, 8], &[&[false, false]]);
    object.regions[0].layers[1]
        .surfaces
        .push(RegionSurface::internal(ExPolygon::new(
            Polygon::new(Vec::new()),
            Vec::new(),
        )));

    remove_project_top_empty_layers(std::slice::from_mut(&mut object));

    assert_eq!(layer_ids(&object), vec![4, 8]);
    assert_eq!(object.regions[0].layers[1].surfaces.len(), 1);
}

#[test]
fn task22k_top_empty_layers_trim_zero_region_all_empty_and_multiple_objects() {
    let mut objects = [
        object(&[1, 2], &[]),
        object(
            &[3, 5, 8],
            &[&[false, false, false], &[false, false, false]],
        ),
        object(&[13, 21, 34], &[&[true, false, true]]),
    ];

    remove_project_top_empty_layers(&mut objects);

    assert!(objects[0].plan.layers.is_empty());
    assert!(objects[0].regions.is_empty());
    assert!(objects[1].plan.layers.is_empty());
    assert!(
        objects[1]
            .regions
            .iter()
            .all(|region| region.layers.is_empty())
    );
    assert_eq!(layer_ids(&objects[2]), vec![13, 21, 34]);
    assert_eq!(region_occupancy(&objects[2]), vec![vec![true, false, true]]);
}

#[test]
fn task22k_top_empty_layers_preserve_complete_volume_sidecar() {
    let source = project_object(
        "task22k.model",
        1,
        vec![project_volume(
            "task22k.model",
            1,
            ProjectVolumeType::ModelPart,
            true,
            false,
        )],
        &[Transform3d::IDENTITY],
    );
    let resolved = resolved_object(0, &[Transform3d::IDENTITY]);
    let bounded = build_volume_bounds(
        &source,
        &resolved,
        PostClosingPrintObject::new(
            plan(0, 0, 2),
            vec![PostClosingVolume::new(
                0,
                17,
                ProjectVolumeType::ModelPart,
                vec![
                    PostClosingLayer::new(SlicingMode::Regular, vec![square_at(0)]),
                    PostClosingLayer::new(SlicingMode::Regular, vec![square_at(20)]),
                ],
            )],
        ),
    );
    let (mut object, ..) = prepare_region_slices(
        bounded,
        VolumeRegionGraph {
            all_regions: vec![region()],
            volume_regions: Vec::new(),
        },
    )
    .into_parts();
    object.regions[0].layers[0]
        .surfaces
        .push(RegionSurface::internal(square()));
    let expected_sidecar = sidecar_snapshot(&object);

    for _ in 0..2 {
        remove_project_top_empty_layers(std::slice::from_mut(&mut object));
        assert_eq!(object.plan.layers.len(), 1);
        assert_eq!(object.regions[0].layers.len(), 1);
        assert_eq!(sidecar_snapshot(&object), expected_sidecar);
    }
}

#[test]
fn task22k_loaded_top_negative_slab_trims_only_empty_suffix() {
    assert_loaded_slab(0.2, 0.4, &[true, false], 1);
}

#[test]
fn task22k_loaded_bottom_negative_slab_preserves_leading_empty_layer() {
    assert_loaded_slab(0.0, 0.2, &[false, true], 2);
}

fn assert_loaded_slab(z0: f64, z1: f64, occupancy: &[bool], retained: usize) {
    let project = slab_project(z0, z1);
    let loaded = load_project(&project).unwrap();
    let volumes = loaded.objects()[0].volumes();
    assert_eq!(
        volumes
            .iter()
            .map(|volume| volume.volume_type())
            .collect::<Vec<_>>(),
        vec![
            ProjectVolumeType::ModelPart,
            ProjectVolumeType::NegativeVolume
        ]
    );
    assert_eq!(volumes[0].mesh().triangles(), volumes[1].mesh().triangles());
    let xy = |index: usize| {
        volumes[index]
            .mesh()
            .vertices()
            .iter()
            .map(|point| (point.x, point.y))
            .collect::<Vec<_>>()
    };
    assert_eq!(xy(0), xy(1));
    let z_bounds = |index: usize| {
        let transform = volumes[index].transform();
        volumes[index].mesh().vertices().iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(min, max), point| {
                let z = transform.transform_point(*point).z;
                (min.min(z), max.max(z))
            },
        )
    };
    let (normal_min, normal_max) = z_bounds(0);
    let (negative_min, negative_max) = z_bounds(1);
    assert!((normal_max - normal_min - 0.4).abs() < 1e-6);
    assert!((negative_min - normal_min - z0).abs() < 1e-6);
    assert!((negative_max - normal_min - z1).abs() < 1e-6);
    let j_bytes = task22k_browser_input_oracle(&project).unwrap();
    assert_eq!(j_bytes, task22j_browser_oracle(&project).unwrap());
    assert_eq!(task22k_browser_input_oracle(&project).unwrap(), j_bytes);
    let j = checkpoint::parse_j(&j_bytes);
    let j_object = &j.stream.objects[0];
    assert_eq!(
        (j_object.planned_layer_count, j_object.retained_layers.len()),
        (2, 2)
    );
    assert_eq!(
        j_object
            .sidecars
            .iter()
            .map(|sidecar| (sidecar.occurrence_id, sidecar.layers.len()))
            .collect::<Vec<_>>(),
        vec![(1, 2), (2, 2)]
    );
    assert_eq!(
        j_object
            .retained_layers
            .iter()
            .map(|layer| layer
                .regions
                .iter()
                .any(|region| !region.surfaces.is_empty()))
            .collect::<Vec<_>>(),
        occupancy
    );
    let k_bytes = task22k_browser_oracle(&project).unwrap();
    assert_eq!(task22k_browser_oracle(&project).unwrap(), k_bytes);
    assert_eq!(k_bytes[8..] == j_bytes[8..], retained == 2);
    let k = checkpoint::parse_k(&k_bytes);
    let k_object = &k.stream.objects[0];
    assert_eq!(
        (k_object.planned_layer_count, k_object.retained_layers.len()),
        (retained as u64, retained)
    );
    assert_eq!(
        k_object.retained_layers,
        j_object.retained_layers[..retained]
    );
    assert_eq!(k_object.sidecars, j_object.sidecars);
}

fn slab_project(z0: f64, z1: f64) -> Vec<u8> {
    let mut archive = KsrArchive::new();
    for (path, text) in [
        ("3D/3dmodel.model", SLAB_ROOT.to_owned()),
        ("3D/_rels/3dmodel.model.rels", SLAB_RELATIONSHIPS.to_owned()),
        ("3D/Objects/task22k_normal.model", box_leaf(1, 0.0, 0.4)),
        ("3D/Objects/task22k_negative.model", box_leaf(3, z0, z1)),
        ("Metadata/model_settings.config", SLAB_SETTINGS.to_owned()),
    ] {
        archive.insert_text(path, &text);
    }
    archive.bytes()
}

#[rustfmt::skip]
const SLAB_ROOT: &str = r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" xmlns:p="http://schemas.microsoft.com/3dmanufacturing/production/2015/06" requiredextensions="p"><resources><object id="2" type="model"><components><component p:path="/3D/Objects/task22k_normal.model" objectid="1"/><component p:path="/3D/Objects/task22k_negative.model" objectid="3"/></components></object></resources><build><item objectid="2" transform="1 0 0 0 1 0 0 0 1 0 0 0" printable="1" auto_drop="1"/></build></model>"#;
#[rustfmt::skip]
const SLAB_RELATIONSHIPS: &str = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Target="/3D/Objects/task22k_normal.model" Id="normal" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/><Relationship Target="/3D/Objects/task22k_negative.model" Id="negative" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel"/></Relationships>"#;
#[rustfmt::skip]
const SLAB_SETTINGS: &str = r#"<config><object id="2"><part id="1" subtype="normal_part"/><part id="3" subtype="negative_part"/></object><plate><metadata key="plater_id" value="1"/><model_instance><metadata key="object_id" value="2"/><metadata key="instance_id" value="0"/><metadata key="identify_id" value="22001"/></model_instance></plate><assemble><assemble_item object_id="2" instance_id="0" transform="1 0 0 0 1 0 0 0 1 0 0 0" offset="0 0 0"/></assemble></config>"#;

#[rustfmt::skip]
fn box_leaf(id: u32, z0: f64, z1: f64) -> String {
    format!(r#"<model unit="millimeter" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02"><resources><object id="{id}" type="model"><mesh><vertices><vertex x="0" y="0" z="{z0}"/><vertex x="20" y="0" z="{z0}"/><vertex x="20" y="2" z="{z0}"/><vertex x="0" y="2" z="{z0}"/><vertex x="0" y="0" z="{z1}"/><vertex x="20" y="0" z="{z1}"/><vertex x="20" y="2" z="{z1}"/><vertex x="0" y="2" z="{z1}"/></vertices><triangles><triangle v1="0" v2="2" v3="1"/><triangle v1="0" v2="3" v3="2"/><triangle v1="4" v2="5" v3="6"/><triangle v1="4" v2="6" v3="7"/><triangle v1="0" v2="1" v3="5"/><triangle v1="0" v2="5" v3="4"/><triangle v1="1" v2="2" v3="6"/><triangle v1="1" v2="6" v3="5"/><triangle v1="2" v2="3" v3="7"/><triangle v1="2" v2="7" v3="6"/><triangle v1="3" v2="0" v3="4"/><triangle v1="3" v2="4" v3="7"/></triangles></mesh></object></resources><build/></model>"#)
}

fn object(ids: &[usize], occupancy: &[&[bool]]) -> PostRegionPrintObject {
    assert!(occupancy.iter().all(|layers| layers.len() == ids.len()));
    PostRegionPrintObject {
        plan: PlannedPrintObject {
            source_object_index: 7,
            transform_index: 9,
            layers: ids
                .iter()
                .copied()
                .map(|id| PlannedLayer {
                    id,
                    height: 0.2,
                    print_z: id as f64,
                    slice_z: id as f64 - 0.1,
                })
                .collect(),
        },
        volume_slices: Vec::new(),
        regions: occupancy
            .iter()
            .enumerate()
            .map(|(id, layers)| PostRegion {
                id,
                options: region(),
                layers: layers
                    .iter()
                    .copied()
                    .map(|nonempty| RegionLayer {
                        surfaces: nonempty
                            .then(|| RegionSurface::internal(square()))
                            .into_iter()
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn square() -> ExPolygon {
    square_at(0)
}

fn square_at(x: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, 0),
            Point::new(x + 10, 0),
            Point::new(x + 10, 10),
            Point::new(x, 10),
        ]),
        Vec::new(),
    )
}

fn sidecar_snapshot(object: &PostRegionPrintObject) -> Vec<(u32, Vec<Vec<ExPolygon>>)> {
    object
        .volume_slices
        .iter()
        .map(|volume| {
            let (occurrence_id, layers) = volume.as_parts();
            (occurrence_id.get(), layers.to_vec())
        })
        .collect()
}

fn layer_ids(object: &PostRegionPrintObject) -> Vec<usize> {
    object.plan.layers.iter().map(|layer| layer.id).collect()
}

fn region_occupancy(object: &PostRegionPrintObject) -> Vec<Vec<bool>> {
    object
        .regions
        .iter()
        .map(|region| {
            region
                .layers
                .iter()
                .map(|layer| !layer.surfaces.is_empty())
                .collect()
        })
        .collect()
}
