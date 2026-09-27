//! Byte-parity reference values produced by the real upstream libnoise
//! modules (Orca-deps-libnoise 1.0, compiled from `noisegen.cpp` +
//! `module/{perlin,billow,ridgedmulti,voronoi}.cpp` with the fuzzy-skin
//! configuration `FuzzySkin.cpp:41-64` applies: frequency `1/scale`,
//! persistence/octaves forwarded, voronoi `displacement = 1.0`).

use super::*;

fn cfg() -> FuzzySkinConfig {
    // `FuzzySkinConfig::disabled` already carries the upstream-default
    // scale 1.0, octaves 4, persistence 0.5 (`fuzzy_skin.rs:45-59`).
    FuzzySkinConfig::disabled()
}

fn pt(x: f64, y: f64) -> crate::Point2 {
    crate::Point2::new(x, y)
}

#[test]
fn perlin_matches_upstream_bitwise() {
    // C++ reference values from `/tmp/noiseprobe` (deps/libnoise 1.0
    // Perlin::GetValue, seed 0, lacunarity 2, QUALITY_STD).
    let cases: [(f64, f64, f64, f64); 4] = [
        (100.5, 100.5, 0.2, -0.81704050808959994),
        (105.73, 105.73, 0.3, -0.048974795831455462),
        (1.234, -5.678, 9.99, -0.82770749022786971),
        (0.0, 0.0, 0.0, 0.0),
    ];
    for (x, y, z, want) in cases {
        let got = coherent_value(FuzzySkinNoiseType::Perlin, pt(x, y), z, cfg());
        assert_eq!(got.to_bits(), want.to_bits(), "perlin({x},{y},{z})");
    }
}

#[test]
fn billow_matches_upstream_bitwise() {
    let cases: [(f64, f64, f64, f64); 4] = [
        (100.5, 100.5, 0.2, 0.57713831655679981),
        (105.73, 105.73, 0.3, -0.23584759515474119),
        (1.234, -5.678, 9.99, 0.28041498045573932),
        (0.0, 0.0, 0.0, -1.375),
    ];
    for (x, y, z, want) in cases {
        let got = coherent_value(FuzzySkinNoiseType::Billow, pt(x, y), z, cfg());
        assert_eq!(got.to_bits(), want.to_bits(), "billow({x},{y},{z})");
    }
}

#[test]
fn ridged_multi_matches_upstream_bitwise() {
    let cases: [(f64, f64, f64, f64); 4] = [
        (100.5, 100.5, 0.2, -0.8995409292830705),
        (105.73, 105.73, 0.3, -0.036474040141535324),
        (1.234, -5.678, 9.99, -0.53791461921273687),
        (0.0, 0.0, 0.0, 1.34375),
    ];
    for (x, y, z, want) in cases {
        let got = coherent_value(FuzzySkinNoiseType::RidgedMulti, pt(x, y), z, cfg());
        assert_eq!(got.to_bits(), want.to_bits(), "ridged({x},{y},{z})");
    }
}

#[test]
fn voronoi_matches_upstream_bitwise() {
    let cases: [(f64, f64, f64, f64); 4] = [
        (100.5, 100.5, 0.2, 0.21855374332517385),
        (105.73, 105.73, 0.3, -0.13823694083839655),
        (1.234, -5.678, 9.99, -0.15797757636755705),
        (0.0, 0.0, 0.0, 0.37672762479633093),
    ];
    for (x, y, z, want) in cases {
        let got = coherent_value(FuzzySkinNoiseType::Voronoi, pt(x, y), z, cfg());
        assert_eq!(got.to_bits(), want.to_bits(), "voronoi({x},{y},{z})");
    }
}
