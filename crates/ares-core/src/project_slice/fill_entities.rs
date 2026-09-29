pub(in crate::project_slice) mod adaptive;
mod concentric;
mod crosshatch;
mod cubic;
mod dump;
mod flow;
mod gap_residual;
mod grid;
mod gyroid;
mod honeycomb;
mod ironing;
mod lateral_honeycomb;
mod lateral_lattice;
mod lightning;
mod line;
mod locked_zag;
mod monotonic;
mod plane_path;
mod quartercubic;
mod simplify;
mod stars;
mod tpms_d;
mod tpms_fk;

pub(in crate::project_slice) use flow::{
    extended_object_bounding_box_x, materialized_flow, object_center,
};
#[cfg(test)]
pub(in crate::project_slice) use generate::generate_layer;

mod generate;
#[cfg(test)]
mod tests;
mod three_d_honeycomb;
mod triangles;
mod types;

pub(in crate::project_slice) use types::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
};

use crate::{
    SliceError,
    project_slice::prepare_infill::{
        combine_infill::{self, PreparedPostInfillCombination},
        external_surfaces::PreparedPostExternalSurfaces,
    },
};

#[cfg(test)]
thread_local! {
    static INVOCATIONS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DISPOSALS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

pub(in crate::project_slice) struct PreparedPostFillEntities {
    pub(in crate::project_slice) predecessor: PreparedPostInfillCombination,
    pub(in crate::project_slice) objects: Vec<Vec<LayerFillEntities>>,
}

pub(in crate::project_slice) fn prepare(
    mut predecessor: PreparedPostInfillCombination,
) -> Result<PreparedPostFillEntities, SliceError> {
    #[cfg(test)]
    INVOCATIONS.with(|count| count.set(count.get() + 1));
    let result = {
        let external = &predecessor.predecessor.predecessor;
        let traversal = &external.predecessor.predecessor;
        traversal
            .objects
            .iter()
            .enumerate()
            .map(|(object_index, object)| {
                let prelude = &object
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object;
                let (compensated, _) = prelude.as_parts();
                let (post_regions, _) = compensated.as_parts();
                let (plan, _, _) = post_regions.as_parts();
                // `prepare_lightning_infill_data` — reuse the generator built
                // in `bridge_over_infill` on the expanded surfaces
                // (`PrintObject.cpp:2711` builds it once, `make_fills` reuses).
                let lightning_generator = predecessor
                    .predecessor
                    .lightning_generators
                    .get(object_index)
                    .and_then(|g| g.as_ref());
                (0..plan.layers.len())
                    .map(|layer_index| {
                        generate::generate_layer(
                            external,
                            object_index,
                            layer_index,
                            lightning_generator.map(|g| g.as_ref()),
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()
    };
    match result {
        Ok(mut objects) => {
            move_thin_fills(&mut objects, &mut predecessor.predecessor.predecessor);
            Ok(PreparedPostFillEntities {
                predecessor,
                objects,
            })
        }
        Err(error) => {
            combine_infill::dispose(predecessor);
            Err(error)
        }
    }
}

fn move_thin_fills(
    objects: &mut [Vec<LayerFillEntities>],
    external: &mut PreparedPostExternalSurfaces,
) {
    for (output, source) in objects.iter_mut().zip(&mut external.predecessor.objects) {
        for (layer, record) in output.iter_mut().zip(&mut source.records) {
            if let Some(record) = record {
                layer.perimeters = std::mem::take(&mut record.perimeters);
                layer
                    .thin_fills
                    .extend(std::mem::take(&mut record.thin_fills));
            }
        }
    }
}

pub(in crate::project_slice) fn dispose(prepared: PreparedPostFillEntities) {
    #[cfg(test)]
    DISPOSALS.with(|count| count.set(count.get() + 1));
    combine_infill::dispose(prepared.predecessor);
}

#[cfg(test)]
pub(in crate::project_slice) fn reset_hooks() {
    INVOCATIONS.with(|count| count.set(0));
    DISPOSALS.with(|count| count.set(0));
}

#[cfg(test)]
pub(in crate::project_slice) fn invocations() -> usize {
    INVOCATIONS.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(in crate::project_slice) fn disposals() -> usize {
    DISPOSALS.with(std::cell::Cell::get)
}

pub(in crate::project_slice) fn simplify_before_ordering(
    enable_arc_fitting: bool,
    spiral_mode: bool,
) -> bool {
    !enable_arc_fitting || spiral_mode
}

pub(in crate::project_slice) fn geometry_error(error: crate::geometry::ClipperError) -> SliceError {
    match error {
        crate::geometry::ClipperError::CoordinateOutOfRange => SliceError::InvalidInput(
            "fill generation coordinate is outside the supported Clipper range".to_owned(),
        ),
        crate::geometry::ClipperError::OpenPathMustBeSubject
        | crate::geometry::ClipperError::OpenPathsRequirePolyTree => {
            unreachable!("fill generators use valid open subjects and closed clips")
        }
    }
}
