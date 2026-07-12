use super::*;
use crate::SliceOptions;
use serde_json::json;

#[test]
fn accepts_all_registered_fuzzy_skin_noise_types() {
    for noise_type in [
        "classic",
        "ripple",
        "perlin",
        "billow",
        "ridgedmulti",
        "voronoi",
    ] {
        assert!(
            options(json!({
                "fuzzy_skin": "external",
                "fuzzy_skin_first_layer": true,
                "fuzzy_skin_noise_type": noise_type
            }))
            .perimeter_options()
            .is_ok(),
            "{noise_type} should parse"
        );
    }
}

#[test]
fn coherent_noise_parameters_enforce_upstream_ranges() {
    for extra in [
        json!({ "fuzzy_skin_scale": 0.09 }),
        json!({ "fuzzy_skin_scale": 500.1 }),
        json!({ "fuzzy_skin_scale": "NaN" }),
        json!({ "fuzzy_skin_octaves": 0 }),
        json!({ "fuzzy_skin_octaves": 11 }),
        json!({ "fuzzy_skin_octaves": "many" }),
        json!({ "fuzzy_skin_persistence": 0.0 }),
        json!({ "fuzzy_skin_persistence": 1.1 }),
        json!({ "fuzzy_skin_persistence": "NaN" }),
    ] {
        assert!(matches!(
            options(extra).perimeter_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn coherent_noise_types_fuzzify_external_perimeters() {
    for noise_type in ["perlin", "billow", "ridgedmulti", "voronoi"] {
        let points = coherent_points(
            1,
            0.4,
            json!({
                "fuzzy_skin_noise_type": noise_type,
                "fuzzy_skin_scale": 1.0,
                "fuzzy_skin_octaves": 4,
                "fuzzy_skin_persistence": 0.5
            }),
        );

        assert_ne!(points, rectangle_points(0.0, 0.0, 4.0, 4.0));
        assert!(points.len() > 4);
        assert!(points.iter().all(|point| point.x() >= -0.2
            && point.x() <= 4.2
            && point.y() >= -0.2
            && point.y() <= 4.2));
    }
}

#[test]
fn coherent_noise_is_deterministic_and_not_classic_or_ripple() {
    let first = coherent_points(1, 0.4, json!({ "fuzzy_skin_noise_type": "perlin" }));
    let second = coherent_points(1, 0.4, json!({ "fuzzy_skin_noise_type": "perlin" }));
    let classic = coherent_points(1, 0.4, json!({ "fuzzy_skin_noise_type": "classic" }));
    let ripple = coherent_points(1, 0.4, json!({ "fuzzy_skin_noise_type": "ripple" }));

    assert_eq!(first, second);
    assert_ne!(first, classic);
    assert_ne!(first, ripple);
}

#[test]
fn coherent_noise_parameters_affect_consuming_noise_types() {
    assert_ne!(
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "perlin", "fuzzy_skin_scale": 0.5 })
        ),
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "perlin", "fuzzy_skin_scale": 2.0 })
        )
    );
    assert_ne!(
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "billow", "fuzzy_skin_octaves": 1 })
        ),
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "billow", "fuzzy_skin_octaves": 5 })
        )
    );
    assert_ne!(
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "perlin", "fuzzy_skin_persistence": 0.2 })
        ),
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "perlin", "fuzzy_skin_persistence": 0.9 })
        )
    );
    assert_ne!(
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "ridgedmulti", "fuzzy_skin_octaves": 1 })
        ),
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "ridgedmulti", "fuzzy_skin_octaves": 5 })
        )
    );
    assert_ne!(
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "voronoi", "fuzzy_skin_scale": 0.5 })
        ),
        coherent_points(
            1,
            0.4,
            json!({ "fuzzy_skin_noise_type": "voronoi", "fuzzy_skin_scale": 2.0 })
        )
    );
}

#[test]
fn coherent_noise_uses_print_z_not_layer_id_as_z_coordinate() {
    assert_ne!(
        coherent_points(1, 0.4, json!({ "fuzzy_skin_noise_type": "perlin" })),
        coherent_points(1, 0.8, json!({ "fuzzy_skin_noise_type": "perlin" }))
    );
}

fn coherent_points(layer_id: usize, print_z: f64, extra: serde_json::Value) -> Vec<Point2> {
    let layers = [LayerContours::new(
        layer_id,
        print_z,
        vec![rectangle(0.0, 0.0, 4.0, 4.0)],
    )];
    let mut value = json!({
        "fuzzy_skin": "external",
        "fuzzy_skin_first_layer": true,
        "fuzzy_skin_thickness": 0.2,
        "fuzzy_skin_point_distance": 0.3,
        "fuzzy_skin_scale": 1.0,
        "fuzzy_skin_octaves": 4,
        "fuzzy_skin_persistence": 0.5
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    let options = options(value).perimeter_options().unwrap();
    let perimeters = generate_perimeters(&layers, options).unwrap();
    perimeters[0].paths()[0].points().to_vec()
}

fn options(extra: serde_json::Value) -> SliceOptions {
    let mut value = json!({
        "layer_height": 0.2,
        "initial_layer_height": 0.2,
        "wall_loops": 1,
        "line_width": 0.4,
        "outer_wall_line_width": 0.4,
        "inner_wall_line_width": 0.4
    });
    for (key, value_extra) in extra.as_object().unwrap() {
        value[key] = value_extra.clone();
    }
    serde_json::from_value(value).unwrap()
}
