use super::raster::SUBPIXEL;

pub(super) type Vector = [i64; 3];
pub(super) const WORLD_UNIT: i64 = 10_000;
const ROTATION_UNIT: i64 = 1 << 20;
// Camera.cpp:739-748: Rx(-45°) * Rz(45°), with sqrt(1/2) rounded to Q20.
const DIAGONAL: i64 = 741_455;

pub(super) fn world(value: f64) -> i64 {
    (value * WORLD_UNIT as f64).round() as i64
}

pub(super) fn rotate([x, y, z]: Vector) -> Vector {
    let half = ROTATION_UNIT / 2;
    [
        (DIAGONAL * (x - y)) / ROTATION_UNIT,
        (half * (x + y) + DIAGONAL * z) / ROTATION_UNIT,
        (-half * (x + y) + DIAGONAL * z) / ROTATION_UNIT,
    ]
}

#[derive(Clone, Copy)]
pub(super) struct Bounds {
    pub(super) min: Vector,
    pub(super) max: Vector,
}

impl Bounds {
    pub(super) fn points(points: &[Vector]) -> Option<Self> {
        let (&first, rest) = points.split_first()?;
        let mut bounds = Self {
            min: first,
            max: first,
        };
        for &point in rest {
            bounds.include(Self {
                min: point,
                max: point,
            });
        }
        Some(bounds)
    }

    pub(super) fn include(&mut self, other: Self) {
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(other.min[axis]);
            self.max[axis] = self.max[axis].max(other.max[axis]);
        }
    }

    fn corners(self) -> [Vector; 8] {
        let extrema = [self.min, self.max];
        std::array::from_fn(|corner| {
            std::array::from_fn(|axis| extrema[(corner >> axis) & 1][axis])
        })
    }
}

pub(super) struct Camera {
    center: Vector,
    spans: [i64; 2],
    distance: i64,
    pub(super) depth_range: [i64; 2],
}

impl Camera {
    pub(super) fn fit(mut objects: Bounds, plate: Bounds) -> Self {
        // GLCanvas3D.cpp:7008-7022: fit the selected geometry down to the bed.
        objects.min[2] = -1;
        for axis in 0..3 {
            let margin = (objects.max[axis] - objects.min[axis]) / if axis == 2 { 50 } else { 100 };
            objects.min[axis] -= margin;
            objects.max[axis] += margin;
        }
        let center =
            std::array::from_fn(|axis| i64::midpoint(objects.min[axis], objects.max[axis]));
        let project =
            |point: Vector| rotate(std::array::from_fn(|axis| point[axis] - center[axis]));
        let span = Bounds::points(&objects.corners().map(project)).unwrap();
        // Camera.cpp:31-36,524-548,561-613: viewport margin and depth planes around the plate.
        let spans = std::array::from_fn(|axis| (span.max[axis] - span.min[axis]) * 41 / 40);
        let plate_view = Bounds::points(&plate.corners().map(project)).unwrap();
        let z_margin = 10 * WORLD_UNIT;
        let mid = i64::midpoint(plate_view.min[2], plate_view.max[2]);
        let far = (plate_view.min[2] - z_margin).min(mid - 25 * WORLD_UNIT);
        let near = (plate_view.max[2] + z_margin).max(mid + 25 * WORLD_UNIT);
        let distance = (1000 * WORLD_UNIT).max(near + 100 * WORLD_UNIT);
        Self {
            center,
            spans,
            distance,
            depth_range: [far, near],
        }
    }

    pub(super) fn view(&self, point: Vector) -> Vector {
        rotate(std::array::from_fn(|axis| point[axis] - self.center[axis]))
    }

    pub(super) fn toward_eye(&self, [x, y, z]: Vector) -> Vector {
        [-x, -y, self.distance - z]
    }

    pub(super) fn screen(&self, dimensions: [u32; 2]) -> impl Fn(Vector) -> [i64; 2] {
        let [width, height] = dimensions.map(i64::from);
        let [span_x, span_y] = self.spans;
        let (size, span) = if width * span_y < height * span_x {
            (width, span_x)
        } else {
            (height, span_y)
        };
        move |[x, y, _]| {
            [
                width * SUBPIXEL / 2 + x * size * SUBPIXEL / span,
                height * SUBPIXEL / 2 + y * size * SUBPIXEL / span,
            ]
        }
    }
}
