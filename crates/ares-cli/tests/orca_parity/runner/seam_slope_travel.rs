//! `seam_slope_type=external` scarf-joint travel parity: upstream
//! `GCode::extrude_path` (GCode.cpp:6359-6368) derives
//! `slope_need_z_travel = m_writer.will_move_z(target_z)` from the
//! writer's live nozzle z (`GCodeWriter::will_move_z`,
//! GCodeWriter.cpp:916-930: with a positive lift a z inside
//! `[pos - m_lifted, pos]` does not move, otherwise the bare
//! `|m_pos.z - z| < EPSILON` shortcut). At an aligned-seam boundary
//! where the previous loop's scarf tip left the nozzle exactly at the
//! next sloped path's start z and the XY first points coincide, every
//! guard arm is false and upstream never calls `travel_to` — no
//! `M204 T`, no zero-displacement `G1 … F18000`, and the pre-scarf
//! feedrate stays modal. Ares' `start_travel.rs` compared against
//! `scarf_z.unwrap_or(layer_z)` while `path.rs` resets `scarf_z` to the
//! nominal layer z after a contour-less sloped path, fabricating a
//! zero-length travel at the MK4 `seam_slope_type=external` Outer-wall
//! scarf tip → Overhang-wall transition.

#[test]
fn mk4_seam_slope_external_travel() {
    crate::runner::native_stream_case::check(
        (
            "seam-slope-travel",
            &[
                "GCode.cpp:extrude_path:6359-6368:slope_need_z_travel-uses-writer-will_move_z",
                "GCodeWriter.cpp:will_move_z:916-930:live-writer-z-and-lift-band",
            ],
        ),
        "mk4-external",
        include_bytes!("../../../../../tests/parity/seam-slope-travel/mk4-external.3mf"),
        "d83a07b1d2eec79be8d911799677e73ca56e49b6babf9d141e35d997f9ad5d9c",
        (false, |_, _| {}),
    );
}
