use super::mesh::{TriangleMesh, Vec3};

const SEED: u64 = 27_644_437;
const MT_N: usize = 312;
const MT_M: usize = 156;
const MATRIX_A: u64 = 0xb502_6f5a_a966_19e9;
const UPPER_MASK: u64 = 0xffff_ffff_8000_0000;
const LOWER_MASK: u64 = 0x0000_0000_7fff_ffff;

#[derive(Debug, PartialEq)]
pub(super) struct TriangleSamples {
    pub(super) total_area: f32,
    pub(super) positions: Vec<Vec3>,
    pub(super) normals: Vec<Vec3>,
}

pub(super) fn sample_uniform(mesh: &TriangleMesh, sample_count: usize) -> TriangleSamples {
    let areas = mesh
        .triangles
        .iter()
        .map(|triangle| triangle.area())
        .collect::<Vec<_>>();
    let mut total_area = 0.0_f32;
    let cumulative = areas
        .into_iter()
        .map(|area| {
            total_area += area as f32;
            total_area
        })
        .collect::<Vec<_>>();

    let mut random = Mt19937_64::new(SEED);
    let random_samples = (0..sample_count)
        .map(|_| [random.unit_f64(), random.unit_f64(), random.unit_f64()])
        .collect::<Vec<_>>();
    let mut positions = Vec::with_capacity(sample_count);
    let mut normals = Vec::with_capacity(sample_count);
    for [triangle_sample, u, v] in random_samples {
        let target = (triangle_sample * f64::from(total_area)) as f32;
        let triangle_index = cumulative.partition_point(|&sum| sum <= target);
        let triangle = mesh.triangles[triangle_index];
        let square_u = u.sqrt() as f32;
        let v = v as f32;
        positions.push(
            triangle.vertices[0] * (1.0 - square_u)
                + triangle.vertices[1] * (square_u * (1.0 - v))
                + triangle.vertices[2] * (v * square_u),
        );
        normals.push(triangle.normal());
    }
    TriangleSamples {
        total_area,
        positions,
        normals,
    }
}

struct Mt19937_64 {
    state: [u64; MT_N],
    index: usize,
}

impl Mt19937_64 {
    fn new(seed: u64) -> Self {
        let mut state = [0; MT_N];
        state[0] = seed;
        for index in 1..MT_N {
            state[index] = 6_364_136_223_846_793_005_u64
                .wrapping_mul(state[index - 1] ^ (state[index - 1] >> 62))
                .wrapping_add(index as u64);
        }
        Self { state, index: MT_N }
    }

    fn unit_f64(&mut self) -> f64 {
        self.next_u64() as f64 / 18_446_744_073_709_551_616.0
    }

    fn next_u64(&mut self) -> u64 {
        if self.index == MT_N {
            self.twist();
        }
        let mut value = self.state[self.index];
        self.index += 1;
        value ^= (value >> 29) & 0x5555_5555_5555_5555;
        value ^= (value << 17) & 0x71d6_7fff_eda6_0000;
        value ^= (value << 37) & 0xfff7_eee0_0000_0000;
        value ^ (value >> 43)
    }

    fn twist(&mut self) {
        for index in 0..MT_N {
            let joined =
                (self.state[index] & UPPER_MASK) | (self.state[(index + 1) % MT_N] & LOWER_MASK);
            let matrix = if joined & 1 == 0 { 0 } else { MATRIX_A };
            self.state[index] = self.state[(index + MT_M) % MT_N] ^ (joined >> 1) ^ matrix;
        }
        self.index = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::Mt19937_64;

    #[test]
    fn mt19937_64_matches_reference_sequence() {
        let mut random = Mt19937_64::new(5489);
        assert_eq!(random.next_u64(), 14_518_275_388_795_819_859);
        assert_eq!(random.next_u64(), 4_622_054_450_568_416_318);
        assert_eq!(random.next_u64(), 13_109_581_317_958_384_263);
    }
}
