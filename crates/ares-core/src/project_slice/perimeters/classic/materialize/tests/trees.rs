use crate::project_slice::perimeters::classic::materialize::{
    tree::consume_nodes, types::RawPathNode,
};

#[test]
fn task22o7_terminal_sink_consumes_a_deep_raw_path_tree_iteratively() {
    std::thread::Builder::new()
        .stack_size(64 * 1024)
        .spawn(|| {
            let mut node = RawPathNode {
                paths: Vec::new(),
                children: Vec::new(),
            };
            for _ in 0..50_000 {
                node = RawPathNode {
                    paths: Vec::new(),
                    children: vec![node],
                };
            }
            consume_nodes(vec![node]);
        })
        .unwrap()
        .join()
        .unwrap();
}
