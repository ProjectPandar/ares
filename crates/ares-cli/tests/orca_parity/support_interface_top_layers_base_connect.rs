//! FillBase.cpp:2247-2650 `Fill::connect_base_support` RED; GREEN needs the in-function fork stage
//! narrowed with dual-side intermediate quantities.
use crate::runner::native_stream_case;

const UPSTREAM: (&str, &[&str]) = (
    "support-interface-top-layers-base-connect",
    &[
        "SupportCommon.cpp:1764-1816::generate_support_toolpaths base fill block",
        "FillRectilinear.cpp:3613-3634::FillSupportBase::fill_surface",
        "FillBase.cpp:2247-2650::Fill::connect_base_support",
    ],
);

/// The `;Z:<height>` layer slice of a G-code stream, or the empty string.
fn layer_slice<'a>(gcode: &'a str, marker: &str) -> &'a str {
    let start = gcode.find(marker).unwrap_or(gcode.len());
    let rest = &gcode[start..];
    let end = rest[marker.len()..]
        .find("\n;Z:")
        .map_or(gcode.len(), |offset| start + marker.len() + offset);
    &gcode[start..end]
}

#[test]
fn support_interface_top_layers_base_connect_target_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "target",
        include_bytes!(
            "../../../../tests/parity/support-interface-top-layers-base-connect/top4.3mf"
        ),
        "a4e969503b241509a5d8e87f02484b41351d6de1883e5391c064c7967f127380",
        (false, |reference, _replay| {
            assert!(
                reference.native.contains(";TYPE:Support\n"),
                "target must extrude support: {:?}",
                reference.root
            );
            // The proven first fork lives in the print_z 0.8 support layer.
            assert!(
                layer_slice(&reference.native, ";Z:0.8").contains(";TYPE:Support\n"),
                "target native print_z 0.8 layer must extrude support: {:?}",
                reference.root
            );
        }),
    );
}

#[test]
fn support_interface_top_layers_base_connect_control_matches_native() {
    native_stream_case::check(
        UPSTREAM,
        "control",
        include_bytes!(
            "../../../../tests/parity/support-interface-top-layers-base-connect/control.3mf"
        ),
        "7a8b9f2abfb2bbc8b74b76bd9affb458ad5a96c81353cfa767b66b773db23239",
        (false, |reference, _replay| {
            // Default `support_interface_top_layers` stays green through GREEN.
            assert!(
                reference.native.contains(";TYPE:Support\n"),
                "control must extrude support: {:?}",
                reference.root
            );
        }),
    );
}
