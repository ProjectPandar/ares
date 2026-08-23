use super::{CubicSpline, qr::FullPivQr};

#[test]
fn task22o129_constant_observations_remain_constant() {
    let points = [0.2, 0.4, 0.6, 0.8, 1.0, 1.2];
    let curve = CubicSpline::fit(&[(3.0, -2.0); 6], &points, &[1.0; 6], 1);

    for point in points {
        let fitted = curve.value(point);
        assert!((fitted.0 - 3.0).abs() < 1.0e-4, "{fitted:?}");
        assert!((fitted.1 + 2.0).abs() < 1.0e-4, "{fitted:?}");
    }
}

#[test]
fn full_pivot_qr_matches_eigen_vector_and_matrix_householder_paths() {
    assert_eq!(solve_generated_system(30, 2), [0x3e8f_ba88, 0x3e17_ab57]);
    assert_eq!(
        solve_generated_system(50, 3),
        [0x3ec9_9660, 0x3e2e_199e, 0xbdb4_9c50]
    );
}

fn solve_generated_system(rows: usize, cols: usize) -> Vec<u32> {
    let matrix = (0..rows)
        .map(|row| {
            (0..cols)
                .map(|column| {
                    let value = (row * 17 + column * 29 + row * column * 3) % 101;
                    (value as i32 - 50) as f32 / 32.0
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let right_hand_side = (0..rows)
        .map(|row| {
            let value = (row * 37 + 11) % 89;
            (value as i32 - 44) as f32 / 16.0
        })
        .collect::<Vec<_>>();

    FullPivQr::factorize(&matrix)
        .solve(&right_hand_side)
        .into_iter()
        .map(f32::to_bits)
        .collect()
}
