use super::light::{self, Intensity};
use crate::ThumbnailData;

pub(super) const SUBPIXEL: i64 = 1 << 8;

#[derive(Clone, Copy)]
pub(super) struct Vertex {
    pub(super) xy: [i64; 2],
    pub(super) depth: i64,
    pub(super) world_z: i64,
}

pub(super) struct Raster {
    dimensions: [u32; 2],
    depth: Vec<i64>,
    pixels: Vec<u8>,
    visible: bool,
}

impl Raster {
    pub(super) fn new(dimensions: [u32; 2]) -> Self {
        let count = dimensions[0] as usize * dimensions[1] as usize;
        Self {
            dimensions,
            depth: vec![i64::MIN; count],
            pixels: vec![0; count * 4],
            visible: false,
        }
    }

    pub(super) fn finish(self) -> Option<ThumbnailData> {
        self.visible.then(|| {
            ThumbnailData::new(self.dimensions[0], self.dimensions[1], self.pixels)
                .expect("raster dimensions match the RGBA buffer")
        })
    }

    pub(super) fn triangle(
        &mut self,
        vertices: [Vertex; 3],
        lights: [Intensity; 3],
        color: [u8; 4],
        depth_range: [i64; 2],
    ) {
        let [a, b, c] = vertices.map(|v| v.xy);
        let edges = [Edge::new(b, c), Edge::new(c, a), Edge::new(a, b)];
        let area = edges[0].distance(a);
        // 3DScene.cpp:708-713: cull back faces; caller accounts for mirrored instances.
        if area <= 0 {
            return;
        }
        let [width, height] = self.dimensions.map(i64::from);
        let [x0, y0] = std::array::from_fn(|axis| {
            ([a[axis], b[axis], c[axis]].into_iter().min().unwrap() + SUBPIXEL / 2 - 1)
                .div_euclid(SUBPIXEL)
                .max(0)
        });
        let [x1, y1] = std::array::from_fn(|axis| {
            ([a[axis], b[axis], c[axis]].into_iter().max().unwrap() - SUBPIXEL / 2)
                .div_euclid(SUBPIXEL)
        });
        let triangle = Triangle {
            edges,
            area,
            depths: vertices.map(|v| v.depth),
            sinking_z: vertices
                .iter()
                .any(|v| v.world_z < 0)
                .then(|| vertices.map(|v| v.world_z)),
            lights: std::array::from_fn(|channel| lights.map(|v| v[channel])),
            color,
            depth_range,
        };
        let mut row = triangle.edges.each_ref().map(|edge| {
            edge.distance([x0 * SUBPIXEL + SUBPIXEL / 2, y0 * SUBPIXEL + SUBPIXEL / 2])
        });
        for y in y0..=y1.min(height - 1) {
            let mut weights = row;
            for x in x0..=x1.min(width - 1) {
                self.fragment((y * width + x) as usize, weights, &triangle);
                weights = std::array::from_fn(|i| weights[i] + triangle.edges[i].dx * SUBPIXEL);
            }
            row = std::array::from_fn(|i| row[i] + triangle.edges[i].dy * SUBPIXEL);
        }
    }

    fn fragment(&mut self, at: usize, weights: [i64; 3], triangle: &Triangle) {
        if (0..3).any(|i| weights[i] < 0 || weights[i] == 0 && !triangle.edges[i].inclusive)
            || triangle.sinking_z.is_some_and(|z| weighted(weights, z) < 0)
        {
            return;
        }
        let depth = (weighted(weights, triangle.depths) / i128::from(triangle.area)) as i64;
        if depth <= self.depth[at]
            || !(triangle.depth_range[0]..=triangle.depth_range[1]).contains(&depth)
        {
            return;
        }
        let intensity = triangle
            .lights
            .map(|values| interpolate(weights, values, triangle.area));
        blend(
            &mut self.pixels[4 * at..][..4],
            light::shade(triangle.color, intensity),
        );
        self.depth[at] = depth;
        self.visible = true;
    }
}

struct Triangle {
    edges: [Edge; 3],
    area: i64,
    depths: [i64; 3],
    sinking_z: Option<[i64; 3]>,
    lights: [[i64; 3]; 2],
    color: [u8; 4],
    depth_range: [i64; 2],
}

struct Edge {
    from: [i64; 2],
    dx: i64,
    dy: i64,
    inclusive: bool,
}

impl Edge {
    fn new(a: [i64; 2], b: [i64; 2]) -> Self {
        let (dx, dy) = (a[1] - b[1], b[0] - a[0]);
        Self {
            from: a,
            dx,
            dy,
            inclusive: dx > 0 || dx == 0 && dy < 0,
        }
    }

    fn distance(&self, [x, y]: [i64; 2]) -> i64 {
        self.dx * (x - self.from[0]) + self.dy * (y - self.from[1])
    }
}

fn interpolate([a, b, c]: [i64; 3], [x, y, z]: [i64; 3], area: i64) -> i64 {
    (a * x + b * y + c * z) / area
}

fn weighted(weights: [i64; 3], values: [i64; 3]) -> i128 {
    (0..3)
        .map(|i| i128::from(weights[i]) * i128::from(values[i]))
        .sum()
}

fn blend(dst: &mut [u8], src: [u8; 4]) {
    if src[3] == 255 || dst[3] == 0 {
        dst.copy_from_slice(&src);
        return;
    }
    let alpha = u32::from(src[3]);
    let front = alpha * 255;
    let back = (255 - alpha) * u32::from(dst[3]);
    let combined = front + back;
    for channel in 0..3 {
        dst[channel] =
            ((front * u32::from(src[channel]) + back * u32::from(dst[channel]) + combined / 2)
                / combined) as u8;
    }
    dst[3] = ((combined + 127) / 255) as u8;
}
