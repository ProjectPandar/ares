use serde::{Serialize, de::DeserializeOwned};

use super::super::super::{
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessInfillPattern, ProcessIroningType, ProcessNoiseType,
    ProcessRegionSourceOptions, ProcessSeamScarfType, ProcessWallDirection, ProcessWallSequence,
};

#[test]
fn region_enum_domains_are_exact_global_maps() {
    assert_domain::<ProcessEnsureVerticalShellThickness>(&[
        "none", "ensure_critical_only", "ensure_moderate", "ensure_all",
    ]);
    assert_domain::<ProcessFuzzySkinType>(&[
        "none", "external", "hole", "all", "allwalls", "disabled_fuzzy",
    ]);
    assert_domain::<ProcessNoiseType>(&[
        "classic", "perlin", "billow", "ridgedmulti", "voronoi", "ripple",
    ]);
    assert_domain::<ProcessFuzzySkinMode>(&["displacement", "extrusion", "combined"]);
    assert_domain::<ProcessIroningType>(&["no ironing", "top", "topmost", "solid"]);
    assert_domain::<ProcessCounterboreHoleBridging>(&[
        "none", "partiallybridge", "sacrificiallayer",
    ]);
    assert_domain::<ProcessWallSequence>(&[
        "inner wall/outer wall", "outer wall/inner wall", "inner-outer-inner wall",
    ]);
    assert_domain::<ProcessWallDirection>(&["ccw", "cw"]);
    assert_domain::<ProcessSeamScarfType>(&["none", "external", "all"]);
    assert_domain::<ProcessInfillPattern>(&[
        "monotonic", "monotonicline", "rectilinear", "alignedrectilinear", "zig-zag",
        "crosszag", "lockedzag", "line", "grid", "triangles", "tri-hexagon", "cubic",
        "adaptivecubic", "quartercubic", "supportcubic", "lightning", "honeycomb",
        "3dhoneycomb", "lateral-honeycomb", "lateral-lattice", "crosshatch", "tpmsd",
        "tpmsfk", "gyroid", "concentric", "hilbertcurve", "archimedeanchords",
        "octagramspiral",
    ]);
}

#[test]
fn all_five_region_patterns_use_the_full_shared_pattern_type() {
    let region = ProcessRegionSourceOptions::default();
    let _: &ProcessInfillPattern = &region.top_surface_pattern;
    let _: &ProcessInfillPattern = &region.bottom_surface_pattern;
    let _: &ProcessInfillPattern = &region.internal_solid_infill_pattern;
    let _: &ProcessInfillPattern = &region.sparse_infill_pattern;
    let _: &ProcessInfillPattern = &region.ironing_pattern;
    for key in [
        "top_surface_pattern",
        "bottom_surface_pattern",
        "internal_solid_infill_pattern",
        "sparse_infill_pattern",
        "ironing_pattern",
    ] {
        let json = format!(r#"{{"{key}":"octagramspiral"}}"#);
        let parsed: ProcessRegionSourceOptions = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_value(parsed).unwrap()[key], "octagramspiral");
    }
}

fn assert_domain<T>(tokens: &[&str])
where
    T: DeserializeOwned + Serialize,
{
    for token in tokens {
        let value: T = serde_json::from_value((*token).into()).unwrap();
        assert_eq!(serde_json::to_value(value).unwrap(), *token);
    }
    assert!(serde_json::from_value::<T>("__invalid__".into()).is_err());
}
