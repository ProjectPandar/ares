//! Arc traversal time attribution to the arc line's own `g1_times_cache` id
//! (`GCodeProcessor.cpp:574` cache push, `:1466` arc lookup,
//! `:771-810` `update` internal-line skip).
use super::ProcessorLimits;
use super::estimate::Estimate;
use super::motion::MotionState;

fn nonbinding_axis_limits() -> ProcessorLimits {
    ProcessorLimits {
        max_acceleration: [f64::MAX; 4],
        ..ProcessorLimits::default()
    }
}

#[test]
fn arc_traversal_time_is_attributed_to_the_arc_lines_own_marker() {
    // From (0,0): a Z hop (id 1), then a G3 half circle (radius 1) that
    // discretizes into 10 internal segments (ids 2..=11), then a G1 (id 12).
    // Each internal segment consumes its own g1 line id
    // (`GCodeProcessor.cpp:3868` `++m_g1_line_id` per discretized G1), and
    // `process_line_move(g1_lines_counter + internal_g1_lines_counter)`
    // (`GCodeProcessor.cpp:1466`) reads the 9th segment's cache entry at the
    // arc line itself; only the last segment's time lands on the next line.
    let lines = [
        "M204 S1000",
        "G1 Z0.2 F600",
        "G3 X0 Y2 I0 J1 F600",
        "G1 X1 F600",
    ]
    .map(str::to_owned);

    assert_eq!(
        MotionState::default().motions("G3 X0 Y2 I0 J1 F600").len(),
        10
    );
    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());

    let at_arc = estimate
        .elapsed_at(3)
        .expect("arc line exports the 9th segment's cumulative time");
    let after_arc = estimate
        .elapsed_at(4)
        .expect("trailing arc segment lands on the next motion line");

    assert!(
        at_arc > 0.8 * after_arc,
        "arc marker {at_arc} should carry most of the traversal time {after_arc}"
    );
    assert!(after_arc > at_arc);
}

#[test]
fn arc_after_closed_wall_reads_the_seam_id_not_the_geometric_count() {
    // `GCodeProcessor.cpp:4215-4225` + `:484-488` + `:790-800`: a
    // `;TYPE:Outer wall` loop that closes just before a `G3` inserts a
    // `Seam` move vertex, associating the arc's FIRST internal block with
    // it. That block drops out of `g1_times_cache`, so
    // `ExportLines::update` sees `remaining_internal_g1_lines == 0` and the
    // arc's M73 marker reads the seam's id (the entry just before the
    // arc), not the geometric `segments - 1`.
    let lines = [
        "M204 S1000",
        "M83",
        ";TYPE:Outer wall",
        "G1 X10 E1 F600",
        "G1 X0 E1 F600",
        "G3 X0 Y2 I0 J1 F600",
        "G1 X1 F600",
    ]
    .map(str::to_owned);

    // Geometric discretization still yields 10 segments (internal 9).
    assert_eq!(
        MotionState::default().motions("G3 X0 Y2 I0 J1 F600").len(),
        10
    );
    let estimate = Estimate::from_lines(&lines, 0.0, nonbinding_axis_limits());

    // `elapsed_at(k)` reads `elapsed[k]`, the marker for source line `k-1`
    // (`elapsed[index + 1]` is written per source index). Source indices:
    // 3 = `G1 X10` (seeds `first_vertex`), 4 = `G1 X0` (wall close — the
    // move stays cache-eligible; the inserted `Seam` vertex is separate),
    // 5 = `G3` arc, 6 = `G1 X1`. The arc's FIRST internal block associates
    // with the seam vertex → `ExportLines::update` returns internal 0, and
    // the arc's M73 reads the last eligible entry (`G1 X0`, the seam-side
    // cumulative), not the geometric `segments - 1` arc segment.
    let arc_line = estimate
        .elapsed_at(6)
        .expect("arc line exports a marker (the seam-side id)");
    // `G1 X10 E1` + `G1 X0 E1` at F600 each traverse 10 mm at 10 mm/s →
    // the wall-close cumulative is ~2.0 s. The seam-side lookup must read
    // that value, never a later arc-internal segment.
    assert!(
        (arc_line - 2.000_199_794_769_287).abs() < 1e-6,
        "arc marker {arc_line} should read the seam-side `G1 X0` cumulative"
    );
}
