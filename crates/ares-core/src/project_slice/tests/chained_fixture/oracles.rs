pub(super) const LAYER_COUNT: usize = 460;
pub(super) const RAW_LINE_COUNT: usize = 116_472;
pub(super) const CLOSED_POLYGON_COUNT: usize = 3_288;
pub(super) const CLOSED_POINT_COUNT: usize = 116_472;
pub(super) const SEMANTIC_ENCODING_LEN: usize = 2_190_993;
pub(super) const CONFIG_BLOCK_LEN: usize = 49_004;

pub(super) const FACE_ORDER_SHA256: &str =
    "6654d9a95ef1bb024f986552b0e8c866ad55dcbe5de3af0cf9c34ff52372adbe";
pub(super) const SEMANTIC_SHA256: &str =
    "7df1e0f90f90e4ff5ca6249c1ceb61e5e1aca74dbdb7b9153fffeff4cd165cdd";
pub(super) const CONFIG_BLOCK_SHA256: &str =
    "b33c979097a4900700d1e5dfcaa16f1454a79ce5fec48da7eb9458cfa2fdeeb8";

pub(super) const REPRESENTATIVE_LAYERS: &[(usize, usize, usize, usize)] = &[
    (0, 1_046, 12, 0),
    (2, 932, 12, 0),
    (12, 1_265, 12, 0),
    (17, 1_138, 12, 0),
    (37, 880, 15, 0),
    (46, 3_011, 41, 0),
    (230, 38, 1, 0),
    (459, 72, 9, 0),
];
pub(super) const LAYER_0_POLYGON_LENGTHS: [usize; 12] =
    [67, 68, 69, 70, 71, 80, 80, 80, 80, 80, 88, 213];
pub(super) const LAYER_230_POLYGON_LENGTHS: [usize; 1] = [38];
pub(super) const LAYER_459_POLYGON_LENGTHS: [usize; 9] = [8; 9];
