pub(crate) fn assert_extrusion_move_command_block(output: &str, marker: &str, command: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let index = lines.iter().position(|line| *line == marker).unwrap();
    let move_marker = matching_move_marker(marker);
    assert_eq!(lines.get(index + 1), Some(&move_marker.as_str()));
    assert_eq!(movement_command_after(&lines, index + 1), command);
}

fn matching_move_marker(extrusion_marker: &str) -> String {
    extrusion_marker
        .replacen(";EXTRUSION:", ";MOVE:", 1)
        .rsplit_once(':')
        .unwrap()
        .0
        .to_owned()
}

pub(crate) fn assert_move_commands_have_extrusion_contract(output: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    for (index, line) in lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(";MOVE:"))
    {
        let command = movement_command_after(&lines, index);
        if line.starts_with(";MOVE:print:") {
            assert!(command.starts_with("G1"));
            assert!(command.contains(" E"));
        } else if line.starts_with(";MOVE:travel:") {
            assert!(command.starts_with("G1"));
            assert!(!command.contains(" E"));
        }
    }
}

pub(crate) fn path_following_command_count(output: &str) -> usize {
    move_command_count(output, "G0") + move_command_count(output, "G1")
}

pub(crate) fn print_move_commands_without_e(output: &str) -> usize {
    let lines = output.lines().collect::<Vec<_>>();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(";MOVE:print:"))
        .filter(|(index, _)| !movement_command_after(&lines, *index).contains(" E"))
        .count()
}

pub(crate) fn assert_move_command_pair(output: &str, marker: &str, command: &str) {
    let lines = output.lines().collect::<Vec<_>>();
    let index = lines.iter().position(|line| *line == marker).unwrap();
    assert_eq!(movement_command_after(&lines, index), command);
}

pub(crate) fn move_command_count(output: &str, prefix: &str) -> usize {
    let lines = output.lines().collect::<Vec<_>>();
    lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.starts_with(";MOVE:"))
        .filter(|(index, _)| movement_command_after(&lines, *index).starts_with(prefix))
        .count()
}

fn movement_command_after<'a>(lines: &'a [&str], marker_index: usize) -> &'a str {
    lines[marker_index + 1..]
        .iter()
        .copied()
        .take_while(|line| !line.starts_with(';'))
        .find(|line| line.starts_with("G1 X") || line.starts_with("G1 Y"))
        .unwrap()
}

pub(crate) fn square_pyramid_ascii_stl() -> Vec<u8> {
    b"solid pyramid\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 1 0 0.4\nvertex 0 1 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 -1 0.4\nvertex 1 0 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex -1 0 0.4\nvertex 0 -1 0.4\nendloop\nendfacet\nfacet normal 0 0 1\nouter loop\nvertex 0 0 0\nvertex 0 1 0.4\nvertex -1 0 0.4\nendloop\nendfacet\nendsolid pyramid"
        .to_vec()
}
