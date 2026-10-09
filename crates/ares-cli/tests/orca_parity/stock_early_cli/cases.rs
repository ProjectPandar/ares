pub(super) struct Case {
    pub(super) vendor: &'static str,
    pub(super) printer: &'static str,
    pub(super) upstream_code: i32,
}

pub(super) const CASES: [Case; 8] = [
    Case {
        vendor: "Anycubic",
        printer: "Anycubic Kobra 3 0.2 nozzle",
        upstream_code: -50,
    },
    Case {
        vendor: "Anycubic",
        printer: "Anycubic Kobra 3 0.4 nozzle",
        upstream_code: -50,
    },
    Case {
        vendor: "Anycubic",
        printer: "Anycubic Kobra 3 0.6 nozzle",
        upstream_code: -50,
    },
    Case {
        vendor: "Anycubic",
        printer: "Anycubic Kobra 3 0.8 nozzle",
        upstream_code: -50,
    },
    Case {
        vendor: "Prusa",
        printer: "Prusa MK3S 0.25 nozzle",
        upstream_code: -19,
    },
    Case {
        vendor: "Prusa",
        printer: "Prusa MK3S 0.4 nozzle",
        upstream_code: -19,
    },
    Case {
        vendor: "Prusa",
        printer: "Prusa MK3S 0.6 nozzle",
        upstream_code: -19,
    },
    Case {
        vendor: "Prusa",
        printer: "Prusa MK3S 0.8 nozzle",
        upstream_code: -19,
    },
];
