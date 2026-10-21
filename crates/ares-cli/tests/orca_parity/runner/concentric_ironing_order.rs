//! Public Native/API/CLI RED for concentric ironing ring order.
//!
//! Upstream boundary: `Fill.cpp:1634` selects `FillConcentric` for closed-loop
//! ironing, `Fill.cpp:1714-1731` emits one `no_sort` `ExtrusionEntityCollection`
//! per `ironing_areas` expolygon, and
//! `ExtrusionEntityCollection.hpp:78-85,118-123` makes `can_reverse()` false for
//! `no_sort`, so `chained_path_from` returns the collection unchanged and
//! `first_point`/`last_point` are `entities.front()`/`back()`. Emitted ring order
//! is therefore the stored fill order produced by
//! `FillConcentric::_fill_surface_single`.
//!
//! Ares destination boundary:
//! `crates/ares-core/src/project_slice/fill_entities/ironing.rs:118-167` calls
//! `crate::fill::concentric_sparse::fill_concentric_sparse` and stores the
//! resulting polylines in that same order (`no_sort: true`), so the ring order
//! reaching the emitter is fixed on this side of the seam.
//!
//! The frozen input is GUI-reachable: `ironing_type=top`,
//! `ironing_pattern=concentric`, `ironing_angle=0`, `ironing_spacing=0.15`,
//! `ironing_inset=0.21`, `gcode_label_objects=0` on
//! `tests/parity/summary/time-cost.3mf`.
use crate::runner::native_stream_case;

const PROJECT: &[u8] =
    include_bytes!("../../../../../tests/parity/concentric-ironing-order/project.3mf");
/// Frozen input digest; a mismatch means the fixture was regenerated and this
/// RED no longer pins the documented divergence.
const PROJECT_SHA256: &str = "7143117d16ff40c408a30ecaba3abf504aa22aa64cd777b399834ff1d464dc6c";

#[test]
fn concentric_ironing_ring_order_matches_native_public_slicing() {
    native_stream_case::check(
        (
            "concentric-ironing-order",
            &["Fill.cpp:1714-1731", "FillConcentric.cpp"],
        ),
        "target",
        PROJECT,
        PROJECT_SHA256,
        (false, |_, _| {}),
    );
}
