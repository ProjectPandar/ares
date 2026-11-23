use std::sync::Arc;

use super::{Point3d, ProjectVolume, Transform3d, shared_mesh::MeshPreparation};
use crate::SliceError;

/// Original units of a mesh that needs rescaling to millimeters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StlUnit {
    Meters,
    Inches,
}

/// The single volume of a fresh STL import, before placement or profile application.
#[derive(Clone, Debug)]
pub struct ImportedStl {
    pub(super) volume: ProjectVolume,
    solid_volume: Option<f32>,
    ground_z: f64,
}

impl ImportedStl {
    pub fn read(input: &[u8], filename: &str) -> Result<Self, SliceError> {
        let (volume, solid_volume) = super::multipart::stl_volume(input, filename, 0)?;
        // CLI::run puts the imported instance on the bed before it scales the
        // volume (OrcaSlicer.cpp:1978,4961–4972).
        let min_z = volume
            .mesh
            .vertices()
            .iter()
            .map(|vertex| vertex.z)
            .fold(f64::INFINITY, f64::min);
        let ground_z = -(min_z + volume.transform.to_column_major()[14]);
        Ok(Self {
            volume,
            solid_volume,
            ground_z,
        })
    }

    pub fn volume(&self) -> &ProjectVolume {
        &self.volume
    }

    pub(super) fn ground_transform(&self) -> Transform3d {
        Transform3d::IDENTITY.pretranslated(Point3d::new(0.0, 0.0, self.ground_z))
    }

    /// Infer from solid volume, testing meters first (`Model.cpp:975–1041`).
    pub fn infer_unit(&self) -> Option<StlUnit> {
        if self.volume.stl_conversion.is_some() {
            return None;
        }
        let volume = f64::from(self.solid_volume?);
        if volume < 0.008 {
            Some(StlUnit::Meters)
        } else if volume < 8.0 {
            Some(StlUnit::Inches)
        } else {
            None
        }
    }

    /// Scale the imported mesh and its offset, recording its source unit.
    pub fn convert_to_millimeters(&mut self, unit: StlUnit) {
        assert!(
            self.volume.stl_conversion.is_none(),
            "STL units were already converted"
        );
        let factor = match unit {
            StlUnit::Meters => 1000.0_f32,
            StlUnit::Inches => 25.4_f32,
        };
        for vertex in &mut Arc::make_mut(&mut self.volume.mesh.geometry).vertices {
            *vertex = Point3d::new(
                f64::from(vertex.x as f32 * factor),
                f64::from(vertex.y as f32 * factor),
                f64::from(vertex.z as f32 * factor),
            );
        }
        let matrix = self.volume.transform.to_column_major();
        let offset = Point3d::new(matrix[12], matrix[13], matrix[14]);
        self.volume.stl_conversion = Some((unit, self.volume.stl_source_offset()));
        let scale = f64::from(factor);
        self.volume.transform =
            self.volume
                .transform
                .without_translation()
                .pretranslated(Point3d::new(
                    offset.x * scale,
                    offset.y * scale,
                    offset.z * scale,
                ));
        self.volume.mesh_preparation = MeshPreparation::new(
            &self.volume.mesh,
            self.volume.transform,
            self.volume.id as i32,
        );
    }
}
