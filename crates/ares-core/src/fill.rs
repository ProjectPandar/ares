pub(crate) mod checked_rotate;
pub(crate) mod connect;
pub(crate) mod cross_hatch;
#[cfg_attr(
    not(test),
    allow(
        dead_code,
        unused_imports,
        reason = "vertical segmentation is consumed by a later rectilinear link-graph slice"
    )
)]
pub(crate) mod rectilinear;
