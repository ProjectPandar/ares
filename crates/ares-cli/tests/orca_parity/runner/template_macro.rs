//! Inline macro-block statement grammar (`PlaceholderParser.cpp:2157-2186,
//! 2162-2174, 2287-2300`): statement lists, `local`/`global` and inline
//! `if/then/elsif/else/endif` inside a single `{...}` block. The frozen
//! two-tool towerless host carries the verbatim Prusa XL stock
//! `change_filament_gcode`, which stock profiles ship today.

#[test]
fn macro_block_statements_render_prusa_xl_stock_script() {
    crate::runner::native_stream_case::check(
        (
            "template-macro",
            &[
                "PlaceholderParser.cpp:2157-2186",
                "PlaceholderParser.cpp:2162-2174,2287-2300",
                "PlaceholderParser.cpp:889-906,996-1000,1384-1448",
            ],
        ),
        "prusa-xl-change-filament",
        include_bytes!("../../../../../tests/parity/template-macro/prusa-xl-change-filament.3mf"),
        "2c1b4c18ebd99c029991bd33130b6f17e286671fd1b285bcdc7c7f2285af55ce",
        (true, |reference, _| {
            let lines: Vec<&str> = reference.native.lines().collect();
            let start = lines
                .iter()
                .position(|line| *line == "; Change Tool0 -> Tool1 (layer 0)")
                .expect("Native must render the stock toolchange header");
            let block = &lines[start..(start + 6).min(lines.len())];
            for expected in [
                "G1 F21000",
                "P0 S1 L2 D0",
                "; 0",
                "M109 S220 T1",
                "T1 S1 L0 D0",
            ] {
                assert!(
                    block.contains(&expected),
                    "Native macro block must contain {expected}: {block:?}"
                );
            }
        }),
    );
}
