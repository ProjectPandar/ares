use super::camera::Vector;

const UNIT: i64 = 1 << 16;
// thumbnail.vs:3-15 and GLCanvas3D.cpp:7132-7133, rounded to Q16.
const TOP: Vector = [-29_982, 29_982, 49_971];
const FRONT: Vector = [45_777, 9_155, 45_777];
const AMBIENT_AND_EMISSION: i64 = 26_214;
const TOP_DIFFUSE: i64 = 31_457;
const FRONT_DIFFUSE: i64 = 11_796;
const TOP_SPECULAR: i64 = 4_915;

pub(super) type Intensity = [i64; 2];

fn unit(v: Vector) -> Vector {
    let length = v
        .map(i128::from)
        .map(|x| x * x)
        .into_iter()
        .sum::<i128>()
        .isqrt();
    v.map(|x| (i128::from(x) * i128::from(UNIT) / length) as i64)
}

fn dot(a: Vector, b: Vector) -> i64 {
    (a[0] * b[0] + a[1] * b[1] + a[2] * b[2]) / UNIT
}

fn mul(x: i64, y: i64) -> i64 {
    (x * y + UNIT / 2) / UNIT
}

pub(super) fn viewer(vector: Vector) -> Vector {
    unit(vector)
}

/// `thumbnail.vs:31-45`: directional diffuse and reflected top specular.
pub(super) fn intensity(normal: Vector, eye: Vector) -> Intensity {
    let normal = unit(normal);
    let top = dot(normal, TOP);
    let reflected = std::array::from_fn(|axis| 2 * top * normal[axis] / UNIT - TOP[axis]);
    let cosine = dot(eye, reflected).clamp(0, UNIT);
    let square = mul(cosine, cosine);
    let fourth = mul(square, square);
    let eighth = mul(fourth, fourth);
    let sixteenth = mul(eighth, eighth);
    [
        AMBIENT_AND_EMISSION
            + mul(top.max(0), TOP_DIFFUSE)
            + mul(dot(normal, FRONT).max(0), FRONT_DIFFUSE),
        mul(TOP_SPECULAR, mul(sixteenth, fourth)),
    ]
}

/// `thumbnail.fs:19-22`: shade interpolated intensity, retaining material alpha.
pub(super) fn shade([r, g, b, a]: [u8; 4], [diffuse, specular]: Intensity) -> [u8; 4] {
    let [r, g, b] = [r, g, b].map(|channel| {
        ((i64::from(channel) * diffuse + 255 * specular + UNIT / 2) / UNIT).min(255) as u8
    });
    [r, g, b, a]
}
