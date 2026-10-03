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
    let estimate = Estimate::from_lines(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
    );

    let at_arc = estimate
        .elapsed_at(3)
        .expect("arc line exports the 9th segment's cumulative time");
    let after_arc = estimate
        .elapsed_at(4)
        .expect("trailing arc segment lands on the next motion line");

    // `process_line_move(g1_lines_counter + internal_g1_lines_counter)`
    // (`GCodeProcessor.cpp:1466`) reads the SECOND-TO-LAST internal segment's
    // cache entry — `internal` = `segments - 1` (9) lands on id10, the 9th of
    // 10 internal segments. The last segment's tail lands on the next line.
    assert!(
        (at_arc - 0.303_724_735_975_265_5).abs() < 1e-6,
        "arc marker {at_arc} should read the second-to-last internal segment (id10)"
    );
    // The trailing `G1 X1` lookup lands one entry later — the arc's last
    // internal segment (id11), a strictly larger cumulative.
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
    let estimate = Estimate::from_lines(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
    );

    // `elapsed_at(k)` reads `elapsed[k]`, the marker for source line `k-1`.
    // Source indices: 3 = `G1 X10` (seeds `first_vertex`), 4 = `G1 X0`
    // (wall close — cache-eligible id2), 5 = `G3` arc (id3 excluded by the
    // `Seam` vertex), 6 = `G1 X1`. So `elapsed_at(6)` is the arc,
    // `elapsed_at(7)` the trailing `G1 X1`.
    //
    // The arc's first block is cache-excluded, so `ExportLines::update`
    // returns `ret = 0` (`it->id > counter` at :790). `process_line_move`
    // then probes its OWN `g1_times_cache_it` at `it->id == g1_lines_counter
    // + internal` = `exported + 0` (`:1246-1248`, `:1466`) — landing on the
    // wall-close `G1 X0` entry (id2), the last eligible cache item. That
    // entry's `elapsed_time` is the M73 value — the "reads the seam's id"
    // behaviour, driven by `process_line_move`'s probe (not `m_times`).
    let arc_line = estimate
        .elapsed_at(6)
        .expect("seam-excluded arc emits the seam-side g1_times_cache entry");
    // `G1 X10` + `G1 X0` at F600 → the wall-close cumulative ~2.0 s.
    assert!(
        (arc_line - 2.000_199_794_769_287).abs() < 1e-6,
        "arc marker {arc_line} should read the seam-side `G1 X0` cumulative"
    );
    // The exported counter is now one behind the slicer-side ids, so the
    // trailing `G1 X1` probes `it->id > counter` and emits no new marker.
    assert!(
        estimate.elapsed_at(7).is_none(),
        "trailing G1 reads the divergent export cursor (id > counter → None)"
    );
}

#[test]
fn zero_block_arc_still_advances_the_slicer_id_not_the_retained_block_count() {
    // `GCodeProcessor.cpp:3868` `++m_g1_line_id` fires per internal
    // `process_G1` — BEFORE the `:3891` no-displacement early return — so a
    // degenerate arc whose every block drops out still advances the slicer
    // id by the internal-call count, not the retained block count. A
    // `count - 1` advance would underflow on a zero-block arc.
    let lines = [
        "M204 S1000",
        "G1 Z0.2 F600",
        // ~zero-radius arc: upstream emits one zero-displacement internal G1
        // (no retained block) but still increments m_g1_line_id.
        "G3 I0.01 F600",
        "G1 X1 F600",
    ]
    .map(str::to_owned);
    // Must not panic (debug usize underflow) and must produce a marker for
    // the trailing G1 — the id accounting stayed positive.
    let estimate = Estimate::from_lines(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
    );
    let _ = estimate.elapsed_at(4);
}

#[test]
fn degenerate_first_arc_segment_retains_upstream_hole_in_block_ids() {
    // Reviewer fixture: at start (2^53, 2^53) with radius 2, upstream
    // discretizes G3 into N=29 segments; the first internal target rounds to
    // unchanged XY (zero displacement → dropped), so the first RETAINED arc
    // block carries upstream id 3 (the dropped segment consumed id 2),
    // not the compact id 2 a `0..count` enumeration assigns.
    let lines = [
        "M204 S1000",
        "G1 X9007199254740992 Y9007199254740992 F600",
        "G3 I2 F600",
        "G1 X1 F600",
    ]
    .map(str::to_owned);
    let estimate = Estimate::from_lines(
        &lines,
        0.0,
        &nonbinding_axis_limits().with_settings(&Default::default()),
    );
    // The first retained arc block carries upstream id 3 (the dropped
    // zero-displacement first segment consumed id 2), not the compact id 2.
    assert_eq!(
        estimate.block_line_ids[1], 3,
        "first retained arc block should keep the upstream id-3 hole, got {:?}",
        estimate.block_line_ids
    );
    let _ = estimate.elapsed_at(4);
}
