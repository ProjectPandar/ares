use super::{
    CubicSpline,
    qr::{FullPivQr, solve_upper_triangular},
};

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
    assert_eq!(
        solve_generated_system(70, 4),
        [0x3e43_f44b, 0x3e24_7436, 0xbc56_251c, 0xbe47_9c1c]
    );
}

#[test]
fn upper_triangular_solve_matches_eigen_panel_updates() {
    const SIZE: usize = 10;
    let mut matrix = vec![vec![0.0; SIZE]; SIZE];
    let mut right_hand_side = vec![0.0; SIZE];
    for row in 0..SIZE {
        for (column, coefficient) in matrix[row].iter_mut().enumerate().skip(row) {
            let value = (row * 17 + column * 29 + row * column * 3) % 101;
            *coefficient = (value as i32 - 50) as f32 / 32.0;
        }
        matrix[row][row] += if matrix[row][row] < 0.0 { -2.0 } else { 2.0 };
        let value = (row * 37 + 11) % 89;
        right_hand_side[row] = (value as i32 - 44) as f32 / 16.0;
    }

    solve_upper_triangular(&matrix, SIZE, &mut right_hand_side);

    assert_eq!(
        right_hand_side
            .into_iter()
            .map(f32::to_bits)
            .collect::<Vec<_>>(),
        [
            0x3f16_bf90,
            0xbdf4_d26e,
            0xbfa1_1b4b,
            0xbfd5_fa78,
            0xbf19_b435,
            0x3e8b_e07c,
            0x3e3c_39e9,
            0xbf7b_7e60,
            0x3f04_4f3a,
            0x3f81_f820,
        ]
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
