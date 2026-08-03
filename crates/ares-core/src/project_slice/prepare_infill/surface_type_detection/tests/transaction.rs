use crate::{
    SliceError,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::surface_type_detection::{
            GeometryStep,
            cracks::resolve,
            fail_geometry_at,
            geometry::{internal, opening, safety_difference, subtract_paths},
            geometry_events, reset_geometry_hooks,
            stage::clipped_fill,
        },
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

const ERROR: &str = "surface-type detection geometry is outside the supported Clipper range";

fn rectangle(x0: i64, y0: i64, x1: i64, y1: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x0, y0),
            Point::new(x1, y0),
            Point::new(x1, y1),
            Point::new(x0, y1),
        ]),
        Vec::new(),
    )
}

fn run(step: GeometryStep) -> Result<(), SliceError> {
    let square = rectangle(0, 0, 1_000, 1_000);
    match step {
        GeometryStep::TopSafetyDifference | GeometryStep::BottomSafetyDifference => {
            safety_difference(std::slice::from_ref(&square), &[], step).map(drop)
        }
        GeometryStep::TopShrink | GeometryStep::TopExpand => opening(
            std::slice::from_ref(&square),
            10.0,
            GeometryStep::TopShrink,
            GeometryStep::TopExpand,
        )
        .map(drop),
        GeometryStep::BottomShrink | GeometryStep::BottomExpand => opening(
            std::slice::from_ref(&square),
            10.0,
            GeometryStep::BottomShrink,
            GeometryStep::BottomExpand,
        )
        .map(drop),
        GeometryStep::CrackIntersection
        | GeometryStep::SingletonCrackErosion
        | GeometryStep::ContainmentDifference
        | GeometryStep::ResidualDifference
        | GeometryStep::CollectionResidualErosion => {
            let mut top = vec![RegionSurface::new(
                RegionSurfaceKind::Top,
                rectangle(400, 400, 500, 500),
            )];
            let mut bottom = vec![RegionSurface::new(RegionSurfaceKind::BottomBridge, square)];
            resolve(&mut top, &mut bottom, 100, true)
        }
        GeometryStep::SingletonCrackExpansion | GeometryStep::BottomSubtraction => {
            let tiny = rectangle(0, 0, 100, 100);
            let mut top = vec![RegionSurface::new(RegionSurfaceKind::Top, tiny.clone())];
            let mut bottom = vec![RegionSurface::new(RegionSurfaceKind::BottomBridge, tiny)];
            resolve(&mut top, &mut bottom, 100, true)
        }
        GeometryStep::TopDifference => {
            subtract_paths(std::slice::from_ref(square.contour()), &[]).map(drop)
        }
        GeometryStep::InternalDifference => internal(std::slice::from_ref(&square), &[]).map(drop),
        GeometryStep::FillTopIntersection
        | GeometryStep::FillBottomIntersection
        | GeometryStep::FillBottomBridgeIntersection
        | GeometryStep::FillInternalIntersection => {
            let kind = match step {
                GeometryStep::FillTopIntersection => RegionSurfaceKind::Top,
                GeometryStep::FillBottomIntersection => RegionSurfaceKind::Bottom,
                GeometryStep::FillBottomBridgeIntersection => RegionSurfaceKind::BottomBridge,
                GeometryStep::FillInternalIntersection => RegionSurfaceKind::Internal,
                _ => unreachable!(),
            };
            clipped_fill(
                &[RegionSurface::new(kind, square.clone())],
                std::slice::from_ref(&square),
            )
            .map(drop)
        }
    }
}

#[test]
fn every_o17_geometry_site_has_a_reached_stable_failure_hook() {
    for step in [
        GeometryStep::TopSafetyDifference,
        GeometryStep::TopShrink,
        GeometryStep::TopExpand,
        GeometryStep::BottomSafetyDifference,
        GeometryStep::BottomShrink,
        GeometryStep::BottomExpand,
        GeometryStep::CrackIntersection,
        GeometryStep::SingletonCrackErosion,
        GeometryStep::ContainmentDifference,
        GeometryStep::ResidualDifference,
        GeometryStep::CollectionResidualErosion,
        GeometryStep::SingletonCrackExpansion,
        GeometryStep::BottomSubtraction,
        GeometryStep::TopDifference,
        GeometryStep::InternalDifference,
        GeometryStep::FillTopIntersection,
        GeometryStep::FillBottomIntersection,
        GeometryStep::FillBottomBridgeIntersection,
        GeometryStep::FillInternalIntersection,
    ] {
        reset_geometry_hooks();
        fail_geometry_at(step);
        assert_eq!(
            run(step),
            Err(SliceError::InvalidInput(ERROR.to_owned())),
            "missing failure at {step:?}"
        );
        assert!(geometry_events().contains(&step));
    }
    reset_geometry_hooks();
}
