#[derive(Clone, Debug, PartialEq)]
pub struct DownwardMachine {
    pub name: String,
    pub printable_width: i32,
    pub printable_depth: i32,
    pub printable_height: i32,
    pub exclude_width: i32,
    pub exclude_depth: i32,
    pub wrapping_width: i32,
    pub wrapping_depth: i32,
}

pub fn compatible_downward_machines(
    object_size: [f64; 3],
    machines: &[DownwardMachine],
    wrapping_enabled: bool,
) -> Vec<String> {
    machines
        .iter()
        .filter(|machine| {
            if object_size[2] > f64::from(machine.printable_height)
                || object_size[1] > f64::from(machine.printable_depth)
                || object_size[0] > f64::from(machine.printable_width)
            {
                return false;
            }
            if machine.exclude_width > 0 {
                let real_width = machine.printable_width - machine.exclude_width;
                let real_depth = machine.printable_depth - machine.exclude_depth;
                if object_size[0] > f64::from(real_width) && object_size[1] > f64::from(real_depth)
                {
                    return false;
                }
            }
            if wrapping_enabled && machine.wrapping_width > 0 {
                let real_depth = machine.printable_depth - machine.wrapping_depth;
                if object_size[1] > f64::from(real_depth) {
                    return false;
                }
            }
            true
        })
        .map(|machine| machine.name.clone())
        .collect()
}
