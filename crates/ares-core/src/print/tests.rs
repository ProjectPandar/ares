use super::*;
use crate::{Contour, Point2, PrintPath, PrintPathRole};

#[test]
fn builds_default_print_object_region_layers_and_surfaces() {
    let print = build_print_domain(&sample_layers(), &sample_contours(), &sample_paths()).unwrap();

    assert_eq!(print.objects().len(), 1);
    assert_eq!(print.regions(), &[PrintRegion::new(0)]);
    let layers = print.objects()[0].layers();
    assert_eq!(layers.len(), 2);
    assert_eq!(layers[0].id(), 0);
    assert_eq!(layers[0].print_z(), 0.2);
    assert_eq!(layers[0].height(), 0.2);
    assert_eq!(layers[0].regions().len(), 1);
    assert_eq!(layers[0].regions()[0].region_id(), 0);
    assert_eq!(layers[0].regions()[0].slices().len(), 1);
    assert_eq!(
        layers[0].regions()[0].slices()[0].surface_type(),
        SurfaceType::Perimeter
    );
}

#[test]
fn splits_paths_into_perimeters_fills_and_extras_in_order() {
    let print = build_print_domain(
        &sample_layers()[..1],
        &sample_contours()[..1],
        &sample_paths()[..1],
    )
    .unwrap();
    let region = &print.objects()[0].layers()[0].regions()[0];

    assert_eq!(
        region.perimeters().paths()[0].role(),
        ExtrusionRole::ExternalPerimeter
    );
    assert_eq!(
        region.fills().paths()[0].role(),
        ExtrusionRole::InternalInfill
    );
    assert_eq!(region.fills().paths()[1].role(), ExtrusionRole::SolidInfill);
    assert_eq!(
        region.fills().paths()[2].role(),
        ExtrusionRole::BridgeInfill
    );
    assert_eq!(region.extras().paths()[0].role(), ExtrusionRole::Skirt);
    assert_eq!(region.extras().paths()[1].role(), ExtrusionRole::Brim);
}

#[test]
fn rejects_count_mismatch() {
    assert!(matches!(
        build_print_domain(&sample_layers(), &sample_contours()[..1], &sample_paths()),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn rejects_layer_metadata_mismatch() {
    let contours = vec![LayerContours::new(7, 0.2, vec![square_contour()])];

    assert_eq!(
        build_print_domain(&sample_layers()[..1], &contours, &sample_paths()[..1]),
        Err(SliceError::InvalidInput(
            "layer, contour and print path metadata must match".to_owned()
        ))
    );
}

fn sample_layers() -> Vec<Layer> {
    vec![Layer::new(0, 0.2, 0.2), Layer::new(1, 0.2, 0.4)]
}

fn sample_contours() -> Vec<LayerContours> {
    vec![
        LayerContours::new(0, 0.2, vec![square_contour()]),
        LayerContours::new(1, 0.4, Vec::new()),
    ]
}

fn sample_paths() -> Vec<LayerPrintPaths> {
    vec![
        LayerPrintPaths::new(
            0,
            0.2,
            vec![
                path(PrintPathRole::Skirt),
                path(PrintPathRole::Brim),
                path(PrintPathRole::ExternalPerimeter),
                path(PrintPathRole::SparseInfill),
                path(PrintPathRole::SolidInfill),
                path(PrintPathRole::Bridge),
            ],
        ),
        LayerPrintPaths::new(1, 0.4, Vec::new()),
    ]
}

fn path(role: PrintPathRole) -> PrintPath {
    PrintPath::new(role, vec![Point2::new(0.0, 0.0), Point2::new(1.0, 0.0)]).unwrap()
}

fn square_contour() -> Contour {
    Contour::new(vec![
        Point2::new(0.0, 0.0),
        Point2::new(1.0, 0.0),
        Point2::new(1.0, 1.0),
        Point2::new(0.0, 1.0),
    ])
}
