use super::*;
#[test]
fn task22o9_thin_wall_entities_interleave_with_loops_and_emit() {
    // `traverse_loops` materializes `thin_walls` into the SAME collection the
    // outermost loop frame chains (PerimeterGenerator.cpp:231-238), so a thin
    // `ExtrusionPath` must appear in the output alongside the loop — not
    // appended blindly at the end.
    let nodes = vec![ChainedLoopNode {
        extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
        children: Vec::new(),
        child_order: Vec::new(),
    }];
    let seeds = vec![seed(true, 0, Vec::new())];
    let thin = vec![thin_wall(&[(1_000_000, 1_000_000), (2_000_000, 1_000_000)])];
    let (thin_entities, order) = root_thin_entities_and_order(&nodes, &thin);
    let collection = traverse_loops(
        nodes,
        &seeds,
        ProcessWallDirection::CounterClockwise,
        thin_entities,
        order,
    );
    assert_eq!(collection.entities.len(), 2);
    let has_path = collection
        .entities
        .iter()
        .any(|e| matches!(e, ExtrusionEntity::Path(_)));
    let has_loop = collection
        .entities
        .iter()
        .any(|e| matches!(e, ExtrusionEntity::Loop(_)));
    assert!(
        has_path && has_loop,
        "thin-wall path + loop both emitted; got {:?}",
        collection
            .entities
            .iter()
            .map(ExtrusionEntity::inset_idx)
            .collect::<Vec<_>>()
    );
}

#[test]
fn task22o9_thin_wall_chain_order_and_reversal() {
    // Two loops + one open thin wall with a deterministic chain order
    // `Loop, Path, Loop` (the thin wall placed nearest the first loop's
    // endpoint and far from the second). Orient the thin wall backwards
    // (its polyline runs away from the chain direction) so the chain reverses
    // it — assert the emitted endpoint order flipped.
    let nodes = vec![
        ChainedLoopNode {
            extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
            children: Vec::new(),
            child_order: Vec::new(),
        },
        ChainedLoopNode {
            extrusion_loop: Some(square_loop(9_000_000, 0, ExtrusionLoopRole::Default)),
            children: Vec::new(),
            child_order: Vec::new(),
        },
    ];
    let seeds = vec![seed(true, 0, Vec::new()), seed(true, 0, Vec::new())];
    // Open thin wall near the first loop, running right->left (reversed input).
    let thin = vec![thin_wall(&[(2_000_000, 500_000), (1_000_000, 500_000)])];
    let (thin_entities, order) = root_thin_entities_and_order(&nodes, &thin);
    let collection = traverse_loops(
        nodes,
        &seeds,
        ProcessWallDirection::CounterClockwise,
        thin_entities,
        order,
    );
    assert_eq!(collection.entities.len(), 3);
    let kinds: Vec<&str> = collection
        .entities
        .iter()
        .map(|e| match e {
            ExtrusionEntity::Loop(_) => "loop",
            ExtrusionEntity::Path(_) => "path",
            ExtrusionEntity::MultiPath(_) => "multipath",
        })
        .collect();
    // The thin wall must be interleaved by the chain, not appended blindly.
    assert_eq!(
        kinds,
        vec!["loop", "path", "loop"],
        "thin wall chains between the two loops"
    );
    // The open path's emitted direction honors the chain's reversal: the
    // chain starts it at the end nearer the preceding loop, so the emitted
    // first/last points flip relative to the (backward-oriented) input
    // `[(2_000_000,500_000),(1_000_000,500_000)]`.
    if let ExtrusionEntity::Path(p) = &collection.entities[1] {
        let first = p.polyline.points.first().unwrap();
        let last = p.polyline.points.last().unwrap();
        assert_eq!(
            first.x, 1_000_000,
            "reversed thin path starts at the near end"
        );
        assert_eq!(last.x, 2_000_000, "reversed thin path ends at the far end");
    } else {
        panic!("expected a Path entity in the chain");
    }
}

#[test]
fn task22o9_skipped_root_keeps_thin_wall_boundary() {
    // A `None` (empty-overhang) root node compacts out of the loop set; the
    // thin-entity boundary must still hold so a thin wall isn't mistaken for a
    // loop (round-6 review P1).
    let nodes = vec![
        ChainedLoopNode {
            extrusion_loop: None, // empty-overhang root — compacted out
            children: Vec::new(),
            child_order: Vec::new(),
        },
        ChainedLoopNode {
            extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
            children: Vec::new(),
            child_order: Vec::new(),
        },
    ];
    let seeds = vec![seed(true, 0, Vec::new()), seed(true, 0, Vec::new())];
    let thin = vec![thin_wall(&[(2_000_000, 500_000), (3_000_000, 500_000)])];
    let (thin_entities, order) = root_thin_entities_and_order(&nodes, &thin);
    let collection = traverse_loops(
        nodes,
        &seeds,
        ProcessWallDirection::CounterClockwise,
        thin_entities,
        order,
    );
    assert_eq!(collection.entities.len(), 2);
    assert!(
        collection
            .entities
            .iter()
            .any(|e| matches!(e, ExtrusionEntity::Path(_))),
        "thin wall emits as Path even with a skipped root node"
    );
}

#[test]
fn task22o9_closed_thin_wall_emits_loop() {
    // A closed thin ring wraps as `ExtrusionLoop` (VariableWidth.cpp:226-229),
    // so a thin-wall-only traversal emits a single loop entity.
    let nodes = vec![ChainedLoopNode {
        extrusion_loop: Some(square_loop(1, 0, ExtrusionLoopRole::Default)),
        children: Vec::new(),
        child_order: Vec::new(),
    }];
    let seeds = vec![seed(true, 0, Vec::new())];
    let thin = vec![thin_wall(&[
        (0, 0),
        (1_000_000, 0),
        (1_000_000, 1_000_000),
        (0, 0),
    ])];
    let (thin_entities, order) = root_thin_entities_and_order(&nodes, &thin);
    let collection = traverse_loops(
        nodes,
        &seeds,
        ProcessWallDirection::CounterClockwise,
        thin_entities,
        order,
    );
    // Two loops: the square + the closed thin ring.
    let loops = collection
        .entities
        .iter()
        .filter(|e| matches!(e, ExtrusionEntity::Loop(_)))
        .count();
    assert_eq!(loops, 2, "closed thin ring emits a second Loop entity");
}
