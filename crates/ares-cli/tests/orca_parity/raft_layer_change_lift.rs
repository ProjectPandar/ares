use crate::runner::native_stream_case;

#[test]
fn mk4_moves_directly_to_the_first_object_layer_after_raft_interface() {
    native_stream_case::check(
        (
            "raft-layer-change-lift",
            &[
                "GCode.cpp:8458-8484,10593-10651",
                "GCodeWriter.cpp:901-931,1001-1011,1096-1140",
            ],
        ),
        "mk4-tree-slim",
        include_bytes!("../../../../tests/parity/raft-layer-change-lift/mk4-tree-slim.3mf"),
        "dcee0185e7f24572d065919002ff60e772b4e8f3f2911235cc71d0dbb299180f",
        (false, |reference, _| {
            let (raft, object) = reference.native.split_once(";Z:0.8\n").unwrap();
            assert_eq!(
                raft.lines().rfind(|line| line.starts_with(";TYPE:")),
                Some(";TYPE:Support interface")
            );
            let start = object.split_once("; printing object ").unwrap().1;
            let mut approach = Vec::new();
            for line in start.lines().take_while(|line| *line != ";LAYER_CHANGE") {
                let Some(move_args) = line.strip_prefix("G1 ") else {
                    continue;
                };
                let mut axes = move_args.split_ascii_whitespace();
                let has_xy = axes.clone().any(|axis| axis.starts_with(['X', 'Y']));
                if has_xy && axes.clone().any(|axis| axis.starts_with('E')) {
                    break;
                }
                let z = axes.find_map(|axis| axis.strip_prefix('Z'));
                if has_xy || z.is_some() {
                    approach.push((has_xy, z));
                }
            }
            assert_eq!(approach, [(true, Some(".8"))], "{:?}", reference.root);
        }),
    );
}
