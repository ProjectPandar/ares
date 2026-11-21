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
        .filter(|machine| fits(object_size, machine, wrapping_enabled))
        .map(|machine| machine.name.clone())
        .collect()
}

pub(crate) type ProjectCheck<'a> = (&'a [DownwardMachine], &'a mut Vec<String>);

pub(crate) fn check_project(
    project: &crate::Project,
    selection: Option<u32>,
    check: Option<ProjectCheck<'_>>,
) {
    let Some((machines, report)) = check else {
        return;
    };
    let sizes = project.imported_plate_sizes(selection);
    let wrapping_enabled = project.settings().process.gcode.enable_wrapping_detection.0;
    report.extend(
        machines
            .iter()
            .filter(|machine| {
                sizes
                    .iter()
                    .all(|&size| fits(size, machine, wrapping_enabled))
            })
            .map(|machine| machine.name.clone()),
    );
}

fn fits(size: [f64; 3], machine: &DownwardMachine, wrapping_enabled: bool) -> bool {
    if size[2] > f64::from(machine.printable_height)
        || size[1] > f64::from(machine.printable_depth)
        || size[0] > f64::from(machine.printable_width)
    {
        return false;
    }
    if machine.exclude_width > 0 {
        let real_width = machine.printable_width - machine.exclude_width;
        let real_depth = machine.printable_depth - machine.exclude_depth;
        if size[0] > f64::from(real_width) && size[1] > f64::from(real_depth) {
            return false;
        }
    }
    !wrapping_enabled
        || machine.wrapping_width <= 0
        || size[1] <= f64::from(machine.printable_depth - machine.wrapping_depth)
}
