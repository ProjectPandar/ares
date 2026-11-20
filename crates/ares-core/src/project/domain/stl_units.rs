use std::sync::Arc;

use super::{Point3d, ProjectVolume, shared_mesh::MeshPreparation};
use crate::SliceError;

/// Original units of a mesh that needs rescaling to millimeters.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StlUnit {
    Meters,
    Inches,
}

/// The single volume of a fresh STL import, before placement or profile application.
#[derive(Clone, Debug)]
pub struct ImportedStl(pub(super) ProjectVolume, Option<f32>);

impl ImportedStl {
    pub fn read(input: &[u8], filename: &str) -> Result<Self, SliceError> {
        super::multipart::stl_volume(input, filename, 0)
            .map(|(volume, solid_volume)| Self(volume, solid_volume))
    }

    pub fn volume(&self) -> &ProjectVolume {
        &self.0
    }

    /// Infer from solid volume, testing meters first (`Model.cpp:975–1041`).
    pub fn infer_unit(&self) -> Option<StlUnit> {
        if self.0.converted_from.is_some() {
            return None;
        }
        let volume = f64::from(self.1?);
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
            self.0.converted_from.is_none(),
            "STL units were already converted"
        );
        let factor = match unit {
            StlUnit::Meters => 1000.0_f32,
            StlUnit::Inches => 25.4_f32,
        };
        for vertex in &mut Arc::make_mut(&mut self.0.mesh.geometry).vertices {
            *vertex = Point3d::new(
                f64::from(vertex.x as f32 * factor),
                f64::from(vertex.y as f32 * factor),
                f64::from(vertex.z as f32 * factor),
            );
        }
        let matrix = self.0.transform.to_column_major();
        let offset = Point3d::new(matrix[12], matrix[13], matrix[14]);
        let scale = f64::from(factor);
        self.0.transform = self
            .0
            .transform
            .without_translation()
            .pretranslated(Point3d::new(
                offset.x * scale,
                offset.y * scale,
                offset.z * scale,
            ));
        self.0.mesh_preparation =
            MeshPreparation::new(&self.0.mesh, self.0.transform, self.0.id as i32);
        self.0.converted_from = Some(unit);
    }
}
