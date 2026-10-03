//! `ToolOrdering::initialize_layers`, `mark_skirt_layers` and skirt_done.

const EPSILON: f64 = 1e-4;

pub(super) struct Layer {
    pub(super) print_z: f64,
    pub(super) has_object: bool,
    pub(super) has_extrusions: bool,
    has_skirt: bool,
}

impl Layer {
    pub(super) fn new(print_z: f64, has_object: bool, has_extrusions: bool) -> Self {
        Self {
            print_z,
            has_object,
            has_extrusions,
            has_skirt: false,
        }
    }
}

pub(super) struct SkirtSchedule {
    layers: Vec<Layer>,
    done: usize,
    last_z: f64,
}

impl SkirtSchedule {
    pub(super) fn new(mut raw: Vec<Layer>, max_layer_height: f64) -> Self {
        raw.sort_by(|a, b| a.print_z.total_cmp(&b.print_z));
        let mut layers = Vec::new();
        let mut begin = 0;
        while begin < raw.len() {
            let mut end = begin + 1;
            while end < raw.len() && raw[end].print_z <= raw[begin].print_z + EPSILON {
                end += 1;
            }
            layers.push(Layer::new(
                0.5 * (raw[begin].print_z + raw[end - 1].print_z),
                false,
                false,
            ));
            begin = end;
        }
        for layer in raw {
            let index = tools_for_layer(&layers, layer.print_z);
            layers[index].has_object |= layer.has_object;
            layers[index].has_extrusions |= layer.has_extrusions;
        }
        if layers.first().is_some_and(|layer| layer.has_extrusions) {
            let mut i = 0;
            loop {
                layers[i].has_skirt = true;
                let Some(j) = (i + 1..layers.len()).find(|&j| layers[j].has_object) else {
                    break;
                };
                let mut last_z = layers[i].print_z;
                let mut k = i + 1;
                while k < j {
                    if layers[k + 1].print_z - last_z > max_layer_height + EPSILON {
                        while !layers[k].has_extrusions {
                            k -= 1;
                        }
                        if layers[k].has_skirt {
                            break;
                        }
                        layers[k].has_skirt = true;
                        last_z = layers[k].print_z;
                    }
                    k += 1;
                }
                i = j;
            }
        }
        Self {
            layers,
            done: 0,
            last_z: 0.0,
        }
    }

    pub(super) fn next(&mut self, print_z: f64, limit: usize) -> Option<(usize, f64)> {
        let index = tools_for_layer(&self.layers, print_z);
        let layer = &self.layers[index];
        if !layer.has_skirt
            || self.done >= limit
            || (self.done > 0 && self.last_z >= layer.print_z - EPSILON)
        {
            return None;
        }
        let result = (self.done, f64::from((layer.print_z - self.last_z) as f32));
        self.done += 1;
        self.last_z = layer.print_z;
        Some(result)
    }
}

fn tools_for_layer(layers: &[Layer], print_z: f64) -> usize {
    let mut index = layers.partition_point(|layer| layer.print_z < print_z - EPSILON);
    while index + 1 < layers.len()
        && (layers[index + 1].print_z - print_z).abs() < (layers[index].print_z - print_z).abs()
    {
        index += 1;
    }
    index
}
