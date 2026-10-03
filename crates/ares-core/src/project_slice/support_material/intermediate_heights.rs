//! `SupportMaterial.cpp::raft_and_intermediate_support_layers`.

mod independent;
mod synchronized;

use super::{
    bottom_contacts::BottomContactLayer, contact_layer::TopContactLayer,
    contact_placement::ContactPlacement,
};
use crate::geometry::Polygon;

const EPSILON: f64 = 1e-4;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum SupportLayerType {
    Intermediate,
    Base,
    TopInterface,
    BottomInterface,
}

pub(super) struct IntermediateLayer {
    pub(super) layer_type: SupportLayerType,
    pub(super) print_z: f64,
    pub(super) bottom_z: f64,
    pub(super) height: f64,
    pub(super) polygons: Vec<Polygon>,
}

#[derive(Clone, Copy)]
enum Extreme {
    Top(usize),
    Bottom(usize),
}

impl Extreme {
    fn z(self, tops: &[TopContactLayer], bottoms: &[BottomContactLayer]) -> f64 {
        match self {
            Self::Top(index) => tops[index].bottom_z,
            Self::Bottom(index) => bottoms[index].print_z,
        }
    }
}

struct Interval {
    lower: Option<Extreme>,
    upper: Extreme,
    lower_z: f64,
    upper_z: f64,
}

struct HeightPlanning<'a> {
    placement: &'a ContactPlacement<'a>,
    tops: &'a mut [TopContactLayer],
    bottoms: &'a [BottomContactLayer],
    layers: Vec<IntermediateLayer>,
    object_index: usize,
}

pub(super) fn raft_and_intermediate_support_layers(
    placement: &ContactPlacement<'_>,
    bottom_contacts: &[BottomContactLayer],
    top_contacts: &mut [TopContactLayer],
) -> Vec<IntermediateLayer> {
    let mut extremes = (0..top_contacts.len())
        .map(Extreme::Top)
        .chain((0..bottom_contacts.len()).map(Extreme::Bottom))
        .collect::<Vec<_>>();
    extremes.sort_unstable_by(|left, right| {
        let left_z = left.z(top_contacts, bottom_contacts);
        let right_z = right.z(top_contacts, bottom_contacts);
        if left_z == right_z {
            matches!(left, Extreme::Bottom(_)).cmp(&matches!(right, Extreme::Bottom(_)))
        } else {
            left_z.total_cmp(&right_z)
        }
    });
    let parameters = placement.parameters;
    let start = usize::from(extremes.first().is_some_and(|extreme| {
        (extreme.z(top_contacts, bottom_contacts) - parameters.raft_interface_top_z).abs() < EPSILON
    }));
    let mut planning = HeightPlanning {
        placement,
        tops: top_contacts,
        bottoms: bottom_contacts,
        layers: Vec::new(),
        object_index: 0,
    };
    for index in start..extremes.len() {
        let upper = extremes[index];
        let upper_z = upper.z(planning.tops, planning.bottoms);
        if (upper_z - parameters.first_print_layer_height).abs() < EPSILON {
            if planning
                .layers
                .last()
                .is_none_or(|layer| layer.print_z < parameters.first_print_layer_height)
            {
                planning.append_first_layer();
            }
            continue;
        }
        let lower = (index > start).then(|| extremes[index - 1]);
        let mut lower_z = lower.map_or(parameters.raft_interface_top_z, |extreme| {
            extreme.z(planning.tops, planning.bottoms)
        });
        if lower_z.abs() < EPSILON {
            planning.append_first_layer();
            lower_z = parameters.first_print_layer_height;
        }
        if upper_z - lower_z == 0.0 {
            continue;
        }
        let interval = Interval {
            lower,
            upper,
            lower_z,
            upper_z,
        };
        if placement.independent_support_layer_height {
            planning.independent(interval);
        } else {
            planning.synchronized(interval);
        }
    }
    planning.layers
}

impl HeightPlanning<'_> {
    fn append_first_layer(&mut self) {
        let height = self.placement.parameters.first_print_layer_height;
        self.layers.push(IntermediateLayer {
            layer_type: SupportLayerType::Intermediate,
            print_z: height,
            bottom_z: 0.0,
            height,
            polygons: Vec::new(),
        });
    }
}
