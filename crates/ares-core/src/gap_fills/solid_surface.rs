use super::{GapFillPath, LayerGapFills, SolidSurfaceGapFillInput};
use crate::{
    Point2, SliceError,
    bridge_support::fully_unsupported_layer,
    options::{GapFillTarget, InfillLayerRole},
};

pub(super) fn append(
    gap_fills: &mut [LayerGapFills],
    input: SolidSurfaceGapFillInput<'_>,
) -> Result<(), SliceError> {
    if input.target == GapFillTarget::Nowhere {
        return Ok(());
    }
    validate_layers(gap_fills, &input)?;
    for (layer_index, gap_fill) in gap_fills.iter_mut().enumerate() {
        if skip_bridge_layer(&input, layer_index) {
            continue;
        }
        let role = input
            .infill_options
            .layer_role_for_layers(input.print_layers, layer_index);
        if !target_allows_role(input.target, role) {
            continue;
        }
        let mut paths = Vec::new();
        for contour in input.layer_contours[layer_index].contours() {
            if let Some(path) =
                solid_surface_path(contour.points(), input.infill_options.solid_line_width())?
            {
                paths.push(path);
            }
        }
        gap_fill.append_paths(paths);
    }
    Ok(())
}

fn validate_layers(
    gap_fills: &[LayerGapFills],
    input: &SolidSurfaceGapFillInput<'_>,
) -> Result<(), SliceError> {
    if gap_fills.len() != input.print_layers.len() || gap_fills.len() != input.layer_contours.len()
    {
        return Err(SliceError::InvalidInput(
            "gap-fill, print layer and contour layer counts must match".to_owned(),
        ));
    }
    for ((gap_fill, print_layer), contour_layer) in gap_fills
        .iter()
        .zip(input.print_layers.iter())
        .zip(input.layer_contours.iter())
    {
        if gap_fill.layer_id() != print_layer.id()
            || gap_fill.layer_id() != contour_layer.layer_id()
            || gap_fill.print_z() != print_layer.print_z()
            || gap_fill.print_z() != contour_layer.print_z()
        {
            return Err(SliceError::InvalidInput(
                "gap-fill, print layer and contour layer metadata must match".to_owned(),
            ));
        }
    }
    Ok(())
}

fn target_allows_role(target: GapFillTarget, role: InfillLayerRole) -> bool {
    match role {
        InfillLayerRole::Sparse => false,
        InfillLayerRole::BottomSurface | InfillLayerRole::TopSurface => target.allows_top_bottom(),
        InfillLayerRole::InternalSolid => target.allows_internal_solid(),
    }
}

fn skip_bridge_layer(input: &SolidSurfaceGapFillInput<'_>, layer_index: usize) -> bool {
    if !input.bridge_no_support {
        return false;
    }
    let unsupported_bottom_bridge = !input
        .counterbore_hole_bridging
        .preserves_bridge_surfaces_for_solid_detection()
        && fully_unsupported_layer(input.layer_contours, layer_index);
    unsupported_bottom_bridge
        || (layer_index > 0
            && input.extra_bridge_layer.applies_to_external_bridge()
            && fully_unsupported_layer(input.layer_contours, layer_index - 1)
            && !fully_unsupported_layer(input.layer_contours, layer_index))
}

fn solid_surface_path(
    points: &[Point2],
    solid_width: f64,
) -> Result<Option<GapFillPath>, SliceError> {
    let Some((min_x, min_y, max_x, max_y)) = super::wall::rectangular_bounds(points) else {
        return Ok(None);
    };
    let width = max_x - min_x;
    let height = max_y - min_y;
    if height > 0.0 && height <= 2.0 * solid_width && width > 2.0 * solid_width {
        let center_y = (min_y + max_y) / 2.0;
        GapFillPath::new(vec![
            Point2::new(min_x + solid_width, center_y),
            Point2::new(max_x - solid_width, center_y),
        ])
        .map(Some)
    } else if width > 0.0 && width <= 2.0 * solid_width && height > 2.0 * solid_width {
        let center_x = (min_x + max_x) / 2.0;
        GapFillPath::new(vec![
            Point2::new(center_x, min_y + solid_width),
            Point2::new(center_x, max_y - solid_width),
        ])
        .map(Some)
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        Contour, InfillOptions, Layer, LayerContours, LayerGapFills, Point2,
        bridges::{CounterboreHoleBridging, ExtraBridgeLayer},
        gap_fills::{SolidSurfaceGapFillInput, append_solid_surface_gap_fills},
        options::GapFillTarget,
    };

    #[test]
    fn topbottom_adds_top_and_bottom_gap_fill_but_skips_internal_solid() {
        let gap_fills = generated(GapFillTarget::TopBottom, false, ExtraBridgeLayer::Disabled);

        assert_eq!(gap_fills[0].paths().len(), 1);
        assert!(gap_fills[1].paths().is_empty());
        assert_eq!(gap_fills[2].paths().len(), 1);
    }

    #[test]
    fn everywhere_adds_internal_solid_gap_fill() {
        let gap_fills = generated(GapFillTarget::Everywhere, false, ExtraBridgeLayer::Disabled);

        assert_eq!(gap_fills[1].paths().len(), 1);
    }

    #[test]
    fn nowhere_adds_no_solid_surface_gap_fill() {
        let gap_fills = generated(GapFillTarget::Nowhere, false, ExtraBridgeLayer::Disabled);

        assert!(gap_fills.iter().all(|layer| layer.paths().is_empty()));
    }

    #[test]
    fn spiral_base_topbottom_gap_fill_stops_above_base() {
        let layers = vec![
            rectangle_layer(0, 0.2, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
            rectangle_layer(1, 0.4, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
            rectangle_layer(2, 0.6, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
        ];
        let print_layers = print_layers(&layers);
        let options = InfillOptions::new_for_tests(0.0, 0.0, 0.4)
            .with_minimum_sparse_infill_area_for_tests(0.0)
            .with_spiral_mode_for_tests(true)
            .with_shell_layers_for_tests(2, 0);
        let mut gap_fills = empty_gap_fills(&layers);

        append_solid_surface_gap_fills(
            &mut gap_fills,
            SolidSurfaceGapFillInput {
                print_layers: &print_layers,
                layer_contours: &layers,
                infill_options: &options,
                target: GapFillTarget::TopBottom,
                bridge_no_support: false,
                extra_bridge_layer: ExtraBridgeLayer::Disabled,
                counterbore_hole_bridging: CounterboreHoleBridging::None,
            },
        )
        .unwrap();

        assert_eq!(gap_fills[0].paths().len(), 1);
        assert_eq!(gap_fills[1].paths().len(), 1);
        assert!(gap_fills[2].paths().is_empty());
    }

    #[test]
    fn bridge_no_support_suppresses_bridge_layer_gap_fill() {
        let layers = vec![
            rectangle_layer(0, 0.2, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
            rectangle_layer(1, 0.4, Point2::new(10.0, 0.0), Point2::new(13.0, 0.7)),
        ];
        let gap_fills = generated_for_layers(
            layers,
            GapFillTarget::Everywhere,
            true,
            ExtraBridgeLayer::Disabled,
        );

        assert!(gap_fills[1].paths().is_empty());
    }

    #[test]
    fn uses_solid_line_width_for_threshold_and_inset() {
        let layers = vec![rectangle_layer(
            0,
            0.2,
            Point2::new(0.0, 0.0),
            Point2::new(3.0, 0.7),
        )];
        let gap_fills = generated_for_layers(
            layers,
            GapFillTarget::TopBottom,
            false,
            ExtraBridgeLayer::Disabled,
        );

        assert_eq!(
            gap_fills[0].paths()[0].points(),
            &[Point2::new(0.4, 0.35), Point2::new(2.6, 0.35)]
        );
    }

    fn generated(
        target: GapFillTarget,
        bridge_no_support: bool,
        extra_bridge_layer: ExtraBridgeLayer,
    ) -> Vec<LayerGapFills> {
        generated_for_layers(
            vec![
                rectangle_layer(0, 0.2, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
                rectangle_layer(1, 0.4, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
                rectangle_layer(2, 0.6, Point2::new(0.0, 0.0), Point2::new(3.0, 0.7)),
            ],
            target,
            bridge_no_support,
            extra_bridge_layer,
        )
    }

    fn generated_for_layers(
        layers: Vec<LayerContours>,
        target: GapFillTarget,
        bridge_no_support: bool,
        extra_bridge_layer: ExtraBridgeLayer,
    ) -> Vec<LayerGapFills> {
        let print_layers = print_layers(&layers);
        let options = options();
        let mut gap_fills = empty_gap_fills(&layers);
        append_solid_surface_gap_fills(
            &mut gap_fills,
            SolidSurfaceGapFillInput {
                print_layers: &print_layers,
                layer_contours: &layers,
                infill_options: &options,
                target,
                bridge_no_support,
                extra_bridge_layer,
                counterbore_hole_bridging: CounterboreHoleBridging::None,
            },
        )
        .unwrap();
        gap_fills
    }

    fn empty_gap_fills(layers: &[LayerContours]) -> Vec<LayerGapFills> {
        layers
            .iter()
            .map(|layer| LayerGapFills::new(layer.layer_id(), layer.print_z(), Vec::new()))
            .collect()
    }

    fn print_layers(layers: &[LayerContours]) -> Vec<Layer> {
        layers
            .iter()
            .map(|layer| Layer::new(layer.layer_id(), 0.2, layer.print_z()))
            .collect()
    }

    fn options() -> InfillOptions {
        InfillOptions::new_for_tests(100.0, 0.0, 0.4)
            .with_minimum_sparse_infill_area_for_tests(0.0)
            .with_shell_layers_for_tests(1, 1)
    }

    fn rectangle_layer(layer_id: usize, print_z: f64, min: Point2, max: Point2) -> LayerContours {
        LayerContours::new(
            layer_id,
            print_z,
            vec![Contour::new(vec![
                min,
                Point2::new(max.x(), min.y()),
                max,
                Point2::new(min.x(), max.y()),
            ])],
        )
    }
}
