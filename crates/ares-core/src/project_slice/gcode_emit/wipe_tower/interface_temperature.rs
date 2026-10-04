//! Contact-temperature command rewriting in `GCode.cpp::append_tcr2`.

pub(super) fn rewrite(
    custom: &str,
    tower: &str,
    tool: usize,
    base: i32,
    interface: i32,
) -> (String, String) {
    let token = format!(" T{tool}");
    let custom = custom
        .split_inclusive('\n')
        .filter(|line| {
            let line = line.trim_start_matches([' ', '\t']);
            let matches = line.contains(&token) || number(line, 'T') == i32::try_from(tool).ok();
            !(line.starts_with("M109") && matches && number(line, 'S') == Some(base))
        })
        .collect();
    let preheat = format!("preheat T{tool}");
    let mut rewritten = String::with_capacity(tower.len());
    for line in tower.split_inclusive('\n') {
        let mut line = line.to_owned();
        if line.contains(&preheat)
            && let Some(start) = line.find('S')
        {
            let end = start
                + 1
                + line[start + 1..]
                    .bytes()
                    .take_while(u8::is_ascii_digit)
                    .count();
            line.replace_range(start + 1..end, &interface.to_string());
        }
        let trimmed = line.trim_start_matches([' ', '\t']);
        let matches = !trimmed.contains('T') || trimmed.contains(&token);
        if !(trimmed.starts_with("M109") && matches && number(trimmed, 'S') == Some(interface)) {
            rewritten.push_str(&line);
        }
    }
    (custom, rewritten)
}

fn number(line: &str, letter: char) -> Option<i32> {
    let start = line.find(letter)? + 1;
    let length = line[start..].bytes().take_while(u8::is_ascii_digit).count();
    line[start..start + length].parse().ok()
}

#[cfg(test)]
mod tests {
    #[test]
    fn contact_change_preserves_other_tools_and_preheat_timing() {
        let custom = " M109 S210 T1\nM109 S210 T0\nM109 S210\nM109 S250 T1\n";
        let tower = "  M104 S210 T1 ; preheat T1\nM109 S250\nM109 S250 T0\nM109 S250 T1\nG1 X2\n";
        let (custom, tower) = super::rewrite(custom, tower, 1, 210, 250);
        assert_eq!(custom, "M109 S210 T0\nM109 S210\nM109 S250 T1\n");
        assert_eq!(tower, "  M104 S250 T1 ; preheat T1\nM109 S250 T0\nG1 X2\n");
    }
}
