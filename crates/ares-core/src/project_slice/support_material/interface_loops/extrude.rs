//! `SupportCommon.cpp::LoopInterfaceProcessor::generate` Flow and path payload.

use super::LoopInterfaceProcessor;
use crate::{
    ExtrusionRole, SliceError,
    geometry::CoordinateScale,
    project_slice::{
        perimeters::types::Flow,
        support_material::{
            contact_layer::TopContactLayer, extrusion_path::SupportExtrusionEntity,
        },
    },
};

pub(in crate::project_slice::support_material) struct LoopExtrusions {
    pub(in crate::project_slice::support_material) paths: Vec<SupportExtrusionEntity>,
    pub(in crate::project_slice::support_material) remaining: Vec<crate::geometry::Polygon>,
}

impl LoopInterfaceProcessor {
    pub(in crate::project_slice::support_material) fn extrude(
        &self,
        contact: &mut TopContactLayer,
        interface_flow: Flow,
        scale: CoordinateScale,
    ) -> Result<Option<LoopExtrusions>, SliceError> {
        if !self.enabled || contact.polygons.is_empty() {
            return Ok(None);
        }
        let flow = interface_flow.with_height(contact.height as f32)?;
        let width = scale.scaled_delta(f64::from(flow.width)) as i64;
        let geometry = self
            .generate(
                &contact.polygons,
                contact.overhang_polygons.as_mut(),
                width,
                scale,
            )
            .map_err(|error| {
                SliceError::InvalidInput(format!("support loop geometry error: {error:?}"))
            })?
            .expect("enabled contact loops with nonempty polygons produce geometry");
        let mut paths = Vec::new();
        SupportExtrusionEntity::append_paths(
            &mut paths,
            geometry.lines,
            ExtrusionRole::SupportMaterialInterface,
            flow,
        );
        Ok(Some(LoopExtrusions {
            paths,
            remaining: geometry.remaining,
        }))
    }
}
