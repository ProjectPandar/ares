//! GCode.cpp:2585-2601,5139-5203: global inventory and live layer skip sets.

use crate::runner::{native_api_cli::NativeCase, native_stream_case};

#[test]
fn two_object_inventory_and_layer_envelopes() {
    check(
        "two-object",
        include_bytes!("../../../../tests/parity/bbl-layer-object-labels/two-object.3mf"),
        "e78d0feba53df6318deb96929eb0620d5392f16a88832fb4ffd75969b853ed28",
        (true, |native, _| {
            let layers = assert_layer_masks(native, &[45, 56]);
            assert_eq!(layers.len(), 50);
            assert!(layers.iter().all(|ids| ids == &[45, 56]));
        }),
    );
}

#[test]
fn differing_heights_follow_live_layer_membership() {
    check(
        "differing-heights",
        include_bytes!("../../../../tests/parity/bbl-layer-object-labels/differing-heights.3mf"),
        "cf61c496ff6f3e2f2802e3b77da11aee9520480c69f87211cd8a639b0b625b63",
        (true, |native, _| {
            let layers = assert_layer_masks(native, &[45, 56]);
            let first_single = layers.iter().position(|ids| ids == &[45]).unwrap();
            assert!(first_single > 0 && first_single < layers.len());
            assert!(layers[..first_single].iter().all(|ids| ids == &[45, 56]));
            assert!(layers[first_single..].iter().all(|ids| ids == &[45]));
        }),
    );
}

#[test]
fn one_object_bbl_has_no_layer_envelope() {
    check(
        "one-object",
        include_bytes!("../../../../tests/parity/bbl-layer-object-labels/one-object.3mf"),
        "858fee95543f6bc6b773930197852d778579dd3ef4b4d782da5a019708665f64",
        (false, |native, _| {
            assert!(
                native
                    .native
                    .lines()
                    .any(|line| line == "; model label id: 45")
            );
            assert!(!native.native.contains("; object ids of layer "));
            assert!(!native.native.contains("; object ids of this layer"));
        }),
    );
}

#[test]
fn empty_gui_template_still_has_layer_envelopes() {
    check(
        "empty-template",
        include_bytes!("../../../../tests/parity/bbl-layer-object-labels/empty-template.3mf"),
        "bed39b49300df0bf8c164b4e11dd28a7795f2b3c2e2ed5f8285a72cc285093f3",
        (true, |native, _| {
            let layers = assert_layer_masks(native, &[45, 56]);
            assert_eq!(layers.len(), 50);
            let lines = native.native.lines().collect::<Vec<_>>();
            for (index, line) in lines.iter().enumerate() {
                if line.starts_with("; object ids of layer ") {
                    assert!(lines[index + 2].starts_with("; object ids of this layer"));
                }
            }
        }),
    );
}

#[test]
fn non_bbl_has_no_bbl_inventory_or_envelopes() {
    check(
        "non-bbl",
        include_bytes!("../../../../tests/parity/bbl-layer-object-labels/non-bbl.3mf"),
        "33980328409429587aa8d6b2d97be1d5755c010917113018f2c8fb2bf2e6ffe5",
        (false, |native, _| {
            assert!(!native.native.contains("; model label id:"));
            assert!(!native.native.contains("; object ids of layer "));
            assert!(!native.native.lines().any(|line| line.starts_with("M624 ")));
            assert!(!native.native.lines().any(|line| line == "M625"));
        }),
    );
}

fn check(
    label: &str,
    project: &[u8],
    sha: &str,
    qualification: (bool, impl Fn(&NativeCase, &NativeCase)),
) {
    native_stream_case::check(
        (
            "bbl-layer-labels",
            &[
                "GCode.cpp:2585-2601::m_label_objects_ids",
                "GCode.cpp:5139-5144::layer_object_label_ids",
                "GCode.cpp:5147-5203::insert_timelapse_gcode",
                "GCode.cpp:7318-7347::_encode_label_ids_to_base64",
            ],
        ),
        label,
        project,
        sha,
        qualification,
    );
}

fn assert_layer_masks(native: &NativeCase, inventory: &[u32]) -> Vec<Vec<u32>> {
    let lines = native.native.lines().collect::<Vec<_>>();
    let header = lines
        .iter()
        .find_map(|line| line.strip_prefix("; model label id: "))
        .unwrap();
    assert_eq!(parse_ids(header), inventory);
    let mut layers = Vec::new();
    let mut ends = 0;
    for (index, line) in lines.iter().enumerate() {
        if let Some(start) = line.strip_prefix("; object ids of layer ") {
            let (number, ids) = start.split_once(" start: ").unwrap();
            assert_eq!(number.parse::<usize>().unwrap(), layers.len() + 1);
            let ids = parse_ids(ids);
            let mask = ids.iter().fold(0_u64, |mask, id| {
                mask | (1 << inventory.binary_search(id).unwrap())
            });
            let encoded = lines[index + 1].strip_prefix("M624 ").unwrap();
            assert_eq!(decode_mask(encoded), mask);
            layers.push(ids);
        }
        if let Some(end) = line.strip_prefix("; object ids of this layer") {
            let (number, ids) = end.split_once(" end: ").unwrap();
            assert_eq!(number.parse::<usize>().unwrap(), ends + 1);
            assert_eq!(parse_ids(ids), layers[ends]);
            assert_eq!(lines[index + 1], "M625");
            ends += 1;
        }
    }
    assert!(!layers.is_empty());
    assert_eq!(layers.len(), ends);
    layers
}

fn parse_ids(text: &str) -> Vec<u32> {
    text.split(',').map(|id| id.parse().unwrap()).collect()
}

fn decode_mask(text: &str) -> u64 {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bytes = Vec::new();
    let mut bits = 0_u32;
    let mut count = 0;
    for byte in text.bytes().take_while(|&byte| byte != b'=') {
        bits = (bits << 6) | ALPHABET.iter().position(|&symbol| symbol == byte).unwrap() as u32;
        count += 6;
        if count >= 8 {
            count -= 8;
            bytes.push((bits >> count) as u8);
        }
    }
    u64::from_le_bytes(bytes.try_into().unwrap())
}
