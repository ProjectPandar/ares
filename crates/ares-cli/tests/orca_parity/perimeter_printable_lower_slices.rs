//! `LayerRegion::make_perimeters` / `PerimeterGenerator::printable_slices`
//! at OrcaSlicer 345efec64: perimeter support excludes unprintable lower
//! features before Arachne intersection/difference assigns path roles.

use crate::runner::native_stream_case;

const UPSTREAM: &[&str] = &[
    "LayerRegion.cpp:163-173;PerimeterGenerator.cpp:3189-3202,2664-2670,407-608",
    "Arachne/WallToolPaths.cpp:47-56;WallToolPaths.hpp;ClipperUtils.hpp:447-448",
];

#[test]
fn mk4_printable_lower_slices_strict() {
    native_stream_case::check(
        ("perimeter-printable-lower-slices", UPSTREAM),
        "mk4",
        include_bytes!("../../../../tests/parity/mk4-curl-lifecycle/witness.3mf"),
        "43ee2c6d9574c291cad8f3530b7dc233c3c38c7243bc8f0c365296848ada5549",
        (false, |reference, replay| {
            for native in [reference, replay] {
                assert!(
                    native
                        .native
                        .lines()
                        .any(|line| line == ";TYPE:Overhang wall"),
                    "Native fixture must exercise overhang perimeter emission at {:?}",
                    native.root
                );
            }
        }),
    );
}
