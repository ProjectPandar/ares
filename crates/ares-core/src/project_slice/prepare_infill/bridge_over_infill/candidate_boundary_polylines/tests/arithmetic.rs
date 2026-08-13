use super::*;

#[test]
fn task22o59_preserves_f64_multiplier_promotion_and_direct_f32_casts() {
    let limiting = vec![rectangle(100, 0, 110, 10)];
    let area = candidate_area(vec![rectangle(0, 0, 1, 1)], limiting);
    let total = vec![rectangle(0, 0, 10, 10)];
    let mut total_points_with_spare_capacity = Vec::with_capacity(5);
    total_points_with_spare_capacity.extend([
        Point::new(-1, -1),
        Point::new(11, -1),
        Point::new(11, 11),
        Point::new(-1, 11),
    ]);
    let total_result = vec![Polygon::new(total_points_with_spare_capacity)];
    let limiting_result = vec![rectangle(99, -1, 111, 11)];
    let total_points = total_result[0].points().as_ptr() as usize;
    let mut total_result = Some(total_result);
    let mut limiting_result = Some(limiting_result);
    let call = std::cell::Cell::new(0);
    let scaled_spacing = 16_777_217_i64;
    let spacing = 0.333_333_34_f32;

    let output = prepare_candidate_boundary_polylines_using(
        operation_input(&area, &total, scaled_spacing, spacing),
        |subject, delta| {
            let index = call.get();
            call.set(index + 1);
            match index {
                0 => {
                    assert_eq!(subject.as_ptr(), total.as_ptr());
                    assert_eq!(delta.to_bits(), 0x4ba6_6667);
                    assert_eq!(
                        delta.to_bits(),
                        ((1.3_f64 * scaled_spacing as f64) as f32).to_bits()
                    );
                    Ok(total_result.take().unwrap())
                }
                1 => {
                    assert_eq!(subject.as_ptr(), area.limiting_area.as_ptr());
                    assert_eq!(delta.to_bits(), 0x3dcc_cccd);
                    assert_eq!(
                        delta.to_bits(),
                        ((0.3_f64 * f64::from(spacing)) as f32).to_bits()
                    );
                    Ok(limiting_result.take().unwrap())
                }
                _ => unreachable!(),
            }
        },
    )
    .unwrap()
    .unwrap();

    assert_eq!(call.get(), 2);
    assert_eq!(output[0].points().as_ptr() as usize, total_points);
    assert_eq!(output[0].points().first(), output[0].points().last());
    assert_eq!(output[1].points().first(), output[1].points().last());
}
