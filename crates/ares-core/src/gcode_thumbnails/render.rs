mod camera;
mod light;
mod mesh;
mod raster;

use crate::{
    CancelToken, Point3d, Project, ProjectObject, ProjectSettings, ProjectVolume,
    ProjectVolumeType, SliceError, ThumbnailData, Transform3d,
    project::facets_annotation::FacetSelector,
};
use camera::{Bounds, Camera, Vector, world};
use mesh::{LitMesh, Mesh, transform_points};
use raster::Raster;

pub(super) struct Scene {
    camera: Camera,
    meshes: Vec<LitMesh>,
}

impl Scene {
    pub(super) fn prepare(
        project: &Project,
        settings: &ProjectSettings,
        cancel: &CancelToken,
    ) -> Result<Option<Self>, SliceError> {
        let Some(plate) = plate_bounds(project, settings) else {
            return Ok(None);
        };
        let mut bounds: Option<Bounds> = None;
        let mut meshes = Vec::new();
        for object in project.objects() {
            append_object(&mut meshes, &mut bounds, object, (plate, settings), cancel)?;
        }
        Ok(bounds.map(|bounds| {
            let camera = Camera::fit(bounds, plate);
            let meshes = meshes
                .into_iter()
                .map(|mesh| mesh.illuminate(&camera))
                .collect();
            Self { camera, meshes }
        }))
    }

    pub(super) fn image(
        &self,
        dimensions: [u32; 2],
        cancel: &CancelToken,
    ) -> Result<Option<ThumbnailData>, SliceError> {
        if dimensions.contains(&0) {
            return Ok(None);
        }
        let mut raster = Raster::new(dimensions);
        for mesh in &self.meshes {
            cancel.check()?;
            mesh.draw(&self.camera, dimensions, &mut raster);
        }
        Ok(raster.finish())
    }
}

fn append_object(
    meshes: &mut Vec<Mesh>,
    bounds: &mut Option<Bounds>,
    object: &ProjectObject,
    (plate, settings): (Bounds, &ProjectSettings),
    cancel: &CancelToken,
) -> Result<(), SliceError> {
    let object_extruder = object.region_overrides().extruder.map_or(1, |v| v.0);
    let instances = object
        .instances()
        .iter()
        .filter(|instance| instance.printable());
    let parts = object
        .volumes()
        .iter()
        .filter(|v| v.volume_type() == ProjectVolumeType::ModelPart);
    for (volume, instance) in
        parts.flat_map(|volume| instances.clone().map(move |instance| (volume, instance)))
    {
        cancel.check()?;
        let transform = instance.transform().then(volume.transform());
        let vertices = transform_points(volume.mesh().vertices(), transform);
        let Some(current) = Bounds::points(&vertices) else {
            continue;
        };
        // GLCanvas3D.cpp:6960-6996: test the transformed bounds of each part on this plate.
        if current.max[2] <= 0
            || current.max[2] > plate.max[2]
            || (0..2).any(|axis| {
                current.min[axis] < plate.min[axis] || current.max[axis] > plate.max[axis]
            })
        {
            continue;
        }
        let extruder = volume
            .region_overrides()
            .extruder
            .filter(|v| v.0 > 0)
            .map_or(object_extruder, |v| v.0);
        let slot = extruder.max(1) as usize - 1;
        bounds.get_or_insert(current).include(current);
        append_meshes(meshes, volume, transform, vertices, (settings, slot))?;
    }
    Ok(())
}

fn plate_bounds(project: &Project, settings: &ProjectSettings) -> Option<Bounds> {
    let origin = project.plate_origin();
    let printer = &settings.printer.remaining;
    let points: Vec<_> = printer
        .printable_area
        .0
        .iter()
        .map(|p| [world(p.x + origin.0), world(p.y + origin.1), 0])
        .collect();
    let mut bounds = Bounds::points(&points)?;
    bounds.max[2] = world(printer.printable_height.0);
    // BuildVolume.hpp:110 and GLCanvas3D.cpp:6961-6968: inflate by the scene epsilon (0.0001 mm).
    bounds.min = bounds.min.map(|v| v - 1);
    bounds.max = bounds.max.map(|v| v + 1);
    Some(bounds)
}

fn append_meshes(
    output: &mut Vec<Mesh>,
    volume: &ProjectVolume,
    transform: Transform3d,
    world: Vec<Vector>,
    (settings, slot): (&ProjectSettings, usize),
) -> Result<(), SliceError> {
    let mesh = volume.mesh();
    let paint = &mesh.mmu_segmentation_facets;
    if paint.mapping.is_empty() {
        output.push(Mesh::new(
            mesh.vertices(),
            world,
            mesh.triangles(),
            transform,
            color(settings, slot),
        ));
    } else {
        // 3DScene.cpp:730-789 and Model.cpp:4043-4047: draw the ordinary leaves grouped by paint state.
        let selector = FacetSelector::deserialize(mesh, paint)?;
        for state in 0..=16 {
            let facets = selector.ordinary_facets(state);
            if facets.triangles.is_empty() {
                continue;
            }
            let color = color(settings, if state == 0 { slot } else { state as usize - 1 });
            let local: Vec<_> = facets
                .vertices
                .iter()
                .map(|v| {
                    let [x, y, z] = v.map(f64::from);
                    Point3d::new(x, y, z)
                })
                .collect();
            let world = transform_points(&local, transform);
            output.push(Mesh::new(
                &local,
                world,
                &facets.triangles,
                transform,
                color,
            ));
        }
    }
    Ok(())
}

fn color(settings: &ProjectSettings, slot: usize) -> [u8; 4] {
    let filament = &settings.filament.gcode.filament_colour.0;
    let extruder = &settings.project.print.extruder_colour.0;
    let mut rgba = filament
        .get(slot)
        .or_else(|| filament.first())
        .and_then(|s| hex(s))
        .or_else(|| {
            extruder
                .get(slot)
                .or_else(|| extruder.first())
                .and_then(|s| hex(s))
        })
        .unwrap_or([255; 4]);
    // 3DScene.cpp:114-124 and :1791-1840: visible alpha and lifted dark color.
    rgba[3] = rgba[3].max(26);
    if rgba[..3].iter().all(|&channel| channel < 51) {
        let gray = 51 + ((255 - u16::from(rgba[3])) * 3 + 5) / 10;
        rgba[..3].fill(gray as u8);
    }
    rgba
}

fn hex(value: &str) -> Option<[u8; 4]> {
    let value = value.strip_prefix('#')?;
    if !matches!(value.len(), 6 | 8) || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    let mut rgba = [255; 4];
    for (index, value) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
        let value = std::str::from_utf8(value).expect("validated hex digits");
        rgba[index] = u8::from_str_radix(value, 16).expect("validated hex pair");
    }
    Some(rgba)
}
