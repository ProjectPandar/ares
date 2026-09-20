//! Debug repro: run a dumped layer chunk through the mover (dev only).

#[test]
#[ignore = "dev-only repro: reads /tmp/sv07-chunk1.txt"]
fn sv07_chunk1_repro() {
    let chunk = std::fs::read_to_string("/tmp/sv07-chunk1.txt").unwrap();
    let mut mover =
        super::FanMover::new(0.5, 0.2, false, false, crate::GCodeFlavor::Klipper);
    let out = mover.process_gcode(&chunk, true);
    std::fs::write("/tmp/sv07-mover-out.txt", &out).unwrap();
    let count = out.matches("M106 S255").count();
    println!("M106 S255 count: {count}");
    assert_eq!(count, 1);
}
