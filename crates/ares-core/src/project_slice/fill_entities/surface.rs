//! Shared surface-fill dispatch — the pure geometry entry consumed by both
//! the sparse `generate.rs` arms (via each module's `polylines`) and the
//! support-ironing path (`support_material/ironing.rs`). Mirrors
//! `FillBase.cpp:40-77` (`Fill::new_from_type`), which `SupportCommon.cpp:1879`
//! reaches with the full InfillPattern vocabulary from 3MF loads.
//!
//! Every dispatched pattern is pinned byte-identical to the Native oracle by
//! `support_ironing_pattern_domain` legs. The remaining patterns keep the
//! `UnsupportedProjectFeature` guard: Native probes (organic-overhang
//! fixture, `;TYPE:Ironing` extrusion-line counts) show each of them either
//! emits real ironing paths Ares cannot yet reproduce, crashes Native
//! outright, or panics in the Ares port — recorded per arm below.

use crate::{
    ProcessInfillPattern, SliceError,
    geometry::Polyline,
    project_slice::{
        group_fills::{RepresentativeSurface, SurfaceFill, SurfaceFillParams},
        raft::fills::concentric,
        region_slices::RegionSurfaceKind,
    },
};

/// One surface to fill: geometry values carried by `SurfaceFillParams` (for
/// support ironing: the `SupportCommon.cpp:1880-1888` parameterization on
/// `FillParams` defaults, `FillBase.hpp:56-61`).
pub(in crate::project_slice) struct SurfaceRequest {
    /// `Fill::new_from_type` argument. Typed as `ProcessInfillPattern` so
    /// the `ConcentricInternal` surface specialization cannot reach this
    /// entry (`FillBase.cpp:74-75` guards it to bottom/top surfaces).
    pub(in crate::project_slice) pattern: ProcessInfillPattern,
    pub(in crate::project_slice) expolygon: crate::geometry::ExPolygon,
    pub(in crate::project_slice) params: SurfaceFillParams,
    pub(in crate::project_slice) thickness_layers: u16,
    pub(in crate::project_slice) layer_height: f64,
    pub(in crate::project_slice) print_z: f64,
    pub(in crate::project_slice) call: super::types::SurfaceCall,
}

pub(in crate::project_slice) fn surface_polylines(
    request: &SurfaceRequest,
) -> Result<Vec<Polyline>, SliceError> {
    match request.pattern {
        ProcessInfillPattern::Rectilinear
        | ProcessInfillPattern::AlignedRectilinear
        | ProcessInfillPattern::Monotonic
        | ProcessInfillPattern::MonotonicLine => {
            // `FillRectilinear::fill_surface` and its monotonic subclasses
            // share `fill_surface_by_lines` (`FillRectilinear.cpp:3386-3419`).
            // These four never read the extended-object bounds
            // (`has_consistent_pattern` matches ZigZag|CrossZag|LockedZag
            // only, `enums.rs:225-227`), so the consistent-pattern anchor is
            // out of play here.
            let fill = surface_fill(request);
            let object_geometry = super::types::ObjectFillGeometry {
                scale: request.call.scale,
                extended_object_min_x: 0,
                extended_object_max_x: 0,
            };
            let surfaces = super::monotonic::expolygon_polylines(
                &fill,
                request.pattern,
                &object_geometry,
                &request.call,
            )?;
            Ok(surfaces
                .into_iter()
                .flat_map(|surface| surface.polylines)
                .collect())
        }
        // `FillZigZag`/`FillCrossZag`/`FillLockedZag` ironing: the Polylines
        // `fill_surface` overload never reaches the locked-zag extrusion
        // path (`FillRectilinear.hpp:210-224` overrides only
        // `fill_surface_extrusion`), so all three reduce to the consistent
        // rectilinear traversal with `FillParams` defaults. The grid anchors
        // on the `bbox_object` dummy box — `BoundingBox(±scale_(1mm))`
        // (`SupportCommon.cpp:1433`) installed via `set_bounding_box` at
        // `SupportCommon.cpp:1880` — extended by
        // `FillRectilinear::extended_object_bounding_box`
        // (`FillRectilinear.cpp:2741-2748`); Native emits identical toolpaths
        // for the three patterns at this call site.
        ProcessInfillPattern::ZigZag
        | ProcessInfillPattern::CrossZag
        | ProcessInfillPattern::LockedZag => {
            let fill = surface_fill(request);
            // `scale_(1mm)` — scaled coordinates are `mm / factor()`.
            let one_mm = (1.0 / request.call.scale.factor()) as i64;
            let dummy = crate::geometry::BoundingBox::new(
                crate::geometry::Point::new(-one_mm, -one_mm),
                crate::geometry::Point::new(one_mm, one_mm),
            );
            let (extended_object_min_x, extended_object_max_x) =
                super::flow::extended_object_bounding_box_x(dummy);
            let object_geometry = super::types::ObjectFillGeometry {
                scale: request.call.scale,
                extended_object_min_x,
                extended_object_max_x,
            };
            let surfaces = super::monotonic::expolygon_polylines(
                &fill,
                request.pattern,
                &object_geometry,
                &request.call,
            )?;
            Ok(surfaces
                .into_iter()
                .flat_map(|surface| surface.polylines)
                .collect())
        }
        ProcessInfillPattern::Concentric => {
            // `raft::fills::concentric` is the `SupportCommon`-context
            // `FillConcentric` port (byte-parity pinned by the ironing legs).
            let density = (0.01_f64 * f64::from(request.params.density)) as f32;
            concentric(
                &request.expolygon,
                request.params.spacing,
                density,
                request.call.scale,
            )
            .map_err(super::geometry_error)
        }
        ProcessInfillPattern::Grid => {
            let fill = surface_fill(request);
            super::grid::polylines(
                &fill,
                request.call.scale,
                request.call.reference,
                super::grid::LayerGeometry {
                    layer_id: request.call.layer_id,
                    bounds: None,
                },
            )
        }
        ProcessInfillPattern::Triangles => {
            let fill = surface_fill(request);
            super::triangles::polylines(
                &fill,
                request.call.scale,
                request.call.reference,
                super::triangles::LayerGeometry {
                    layer_id: request.call.layer_id,
                    bounds: None,
                },
            )
        }
        ProcessInfillPattern::Honeycomb => {
            let fill = surface_fill(request);
            // `Fill::fill_surface` offsets the surface by `overlap - spacing/2`
            // (`FillBase.cpp:108`) before `_fill_surface_single`.
            let delta = ((request.params.overlap - 0.5 * request.params.spacing)
                / request.call.scale.factor()) as f32;
            super::honeycomb::expolygon_polylines(
                &request.expolygon,
                delta,
                super::honeycomb::polylines_params(&fill, &request.call),
                request.call.scale,
            )
            .map(|per_component| per_component.into_iter().flatten().collect())
        }
        ProcessInfillPattern::Gyroid => {
            let fill = surface_fill(request);
            super::gyroid::polylines(
                &fill,
                request.print_z,
                request.layer_height,
                request.call.scale,
            )
        }
        ProcessInfillPattern::Cubic => {
            let fill = surface_fill(request);
            super::cubic::polylines(
                &fill,
                request.print_z,
                request.call.scale,
                request.call.reference,
            )
        }
        ProcessInfillPattern::QuarterCubic => {
            let fill = surface_fill(request);
            super::quartercubic::polylines(
                &fill,
                request.print_z,
                request.call.scale,
                request.call.reference,
            )
        }
        ProcessInfillPattern::ThreeDHoneycomb => {
            let fill = surface_fill(request);
            super::three_d_honeycomb::polylines(&fill, request.print_z, request.call.scale)
        }
        // `tri-hexagon` maps to `ipStars` (`PrintConfig.cpp:238`).
        ProcessInfillPattern::TriHexagon => {
            let fill = surface_fill(request);
            super::stars::polylines(
                &fill,
                request.call.scale,
                request.call.reference,
                super::stars::LayerGeometry {
                    layer_id: request.call.layer_id,
                    bounds: None,
                },
            )
        }
        ProcessInfillPattern::TpmsD => {
            let fill = surface_fill(request);
            super::tpms_d::polylines(&fill, request.print_z, request.call.scale)
        }
        ProcessInfillPattern::TpmsFk => {
            let fill = surface_fill(request);
            super::tpms_fk::polylines(&fill, request.print_z, request.call.scale)
        }
        ProcessInfillPattern::LateralHoneycomb => {
            let fill = surface_fill(request);
            super::lateral_honeycomb::polylines(
                &fill,
                request.call.layer_id,
                request.print_z,
                request.call.scale,
                request.call.reference,
            )
        }
        // Equal-index full arcs (dense ironing spacing may project two
        // distinct intersections onto one contour vertex) walk the whole
        // contour like `take_ccw_full` (`FillBase.cpp:474-486`); the
        // `append_full` loop mirrors that release behavior.
        ProcessInfillPattern::Line => {
            let fill = surface_fill(request);
            super::line::expolygon_polylines(&request.expolygon, &fill, &request.call)
        }
        ProcessInfillPattern::LateralLattice => {
            let fill = surface_fill(request);
            super::lateral_lattice::polylines(
                &fill,
                request.call.layer_id,
                request.print_z,
                request.call.scale,
                request.call.reference,
            )
        }
        // Guarded: Native emits real ironing paths that Ares does not
        // reproduce byte-identically (probe evidence, organic-overhang
        // fixture, `;TYPE:Ironing` E-line counts vs Ares CLI diffs):
        // - Lightning / AdaptiveCubic / SupportCubic: Native segfaults
        //   (exit 139) on the uninitialized filler at this call site — there
        //   is no oracle output to mirror.
        // `FillCrossHatch` keys its layered pattern on the layer print_z
        // (`FillCrossHatch.cpp:160-189`).
        ProcessInfillPattern::CrossHatch => super::crosshatch::expolygon_polylines(
            &request.expolygon,
            request.params,
            request.print_z,
            request.call.scale,
        ),
        // At ironing density the plane-path generator runs unaligned on the
        // snug bounds (`FillPlanePath.cpp:80`), so no object context is
        // needed and the pattern centers itself (`centered() == true`).
        ProcessInfillPattern::OctagramSpiral => super::plane_path::expolygon_polylines(
            &request.expolygon,
            crate::fill::plane_path::PlanePathPattern::OctagramSpiral,
            request.params,
            request.call.scale,
        ),
        // `FillArchimedeanChords` shares the center shift
        // (`FillPlanePath.hpp:65`, `centered() == true`).
        ProcessInfillPattern::ArchimedeanChords => super::plane_path::expolygon_polylines(
            &request.expolygon,
            crate::fill::plane_path::PlanePathPattern::ArchimedeanChords,
            request.params,
            request.call.scale,
        ),
        // `FillHilbertCurve` shifts by `bounding_box.min`
        // (`FillPlanePath.hpp:72`, `centered() == false`).
        ProcessInfillPattern::HilbertCurve => super::plane_path::expolygon_polylines(
            &request.expolygon,
            crate::fill::plane_path::PlanePathPattern::HilbertCurve,
            request.params,
            request.call.scale,
        ),
        ProcessInfillPattern::Lightning
        | ProcessInfillPattern::AdaptiveCubic
        | ProcessInfillPattern::SupportCubic => Err(SliceError::UnsupportedProjectFeature(
            "support_ironing_pattern".into(),
        )),
    }
}

fn surface_fill(request: &SurfaceRequest) -> SurfaceFill {
    SurfaceFill {
        region_id: 0,
        representative: RepresentativeSurface {
            kind: RegionSurfaceKind::InternalSolid,
            thickness: 0.0,
            thickness_layers: request.thickness_layers,
        },
        expolygons: vec![request.expolygon.clone()],
        params: request.params,
        region_id_group: Vec::new(),
        no_overlap_expolygons: Vec::new(),
    }
}
