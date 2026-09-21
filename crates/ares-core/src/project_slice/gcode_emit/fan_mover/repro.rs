use super::FanMover;
use crate::GCodeFlavor;

fn split_chunks(text: &str) -> Vec<&str> {
    let mut chunks = Vec::new();
    let mut chunk_start = 0;
    for (offset, _) in text.match_indices("\n;") {
        let line_end = offset + 1;
        if text[line_end..].starts_with(";LAYER_CHANGE") {
            let mut boundary = line_end;
            let bytes = text.as_bytes();
            while boundary > chunk_start {
                let mut prev = boundary - 1;
                while prev > chunk_start && bytes[prev - 1] != b'\n' {
                    prev -= 1;
                }
                if !text[prev..boundary].starts_with("M106")
                    && !text[prev..boundary].starts_with("M107")
                {
                    break;
                }
                boundary = prev;
            }
            chunks.push(&text[chunk_start..boundary]);
            chunk_start = boundary;
        }
    }
    chunks.push(&text[chunk_start..]);
    chunks
}

#[test]
#[ignore = "dev-only repro: reads /tmp/sv07-prefan2.txt"]
fn sv07_full_replay() {
    let text = std::fs::read_to_string("/tmp/sv07-prefan2.txt").unwrap();
    let chunks = split_chunks(&text);
    let mut mover = FanMover::new(0.5, 0.2, false, false, GCodeFlavor::Klipper);
    let mut out = String::new();
    for (n, chunk) in chunks.iter().enumerate() {
        if n < 4 {
            eprintln!(
                "chunk {n}: {} lines, head={:?}",
                chunk.lines().count(),
                chunk.lines().next()
            );
        }
        out.push_str(&mover.process_gcode(chunk, true));
    }
    std::fs::write("/tmp/sv07-replay-out.txt", &out).unwrap();
    let lines: Vec<&str> = out.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.contains("M106 S255") {
            println!("S255 at replay line {}", i + 1);
        }
    }
    println!("total lines: {}", lines.len());
}
