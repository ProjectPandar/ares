use crate::{Model, Point3, SliceError, SliceOptions, Triangle};

pub(crate) fn apply(model: Model, options: &SliceOptions) -> Result<Model, SliceError> {
    let xy = options.filament_shrink_xy()?;
    if xy == 1.0 {
        return Ok(model);
    }

    let triangles = model
        .triangles()
        .iter()
        .map(|triangle| {
            Triangle::new(triangle.vertices().map(|vertex| {
                Point3::new(scaled_f32(vertex.x, xy), scaled_f32(vertex.y, xy), vertex.z)
            }))
        })
        .collect();
    Ok(Model::new(model.format(), triangles))
}

fn scaled_f32(value: f32, scale: f64) -> f32 {
    (f64::from(value) * scale) as f32
}
