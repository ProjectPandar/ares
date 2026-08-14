use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::super::{beading::base::Beading, extrusion_line::ExtrusionJunction};

pub(crate) type Shared<T> = Rc<RefCell<Vec<T>>>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TransitionMiddle {
    pub(crate) pos: i64,
    pub(crate) lower_bead_count: i32,
    pub(crate) feature_radius: i64,
}

impl TransitionMiddle {
    pub(crate) const fn new(pos: i64, lower_bead_count: i32, feature_radius: i64) -> Self {
        Self {
            pos,
            lower_bead_count,
            feature_radius,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TransitionEnd {
    pub(crate) pos: i64,
    pub(crate) lower_bead_count: i32,
    pub(crate) is_lower_end: bool,
}

impl TransitionEnd {
    pub(crate) const fn new(pos: i64, lower_bead_count: i32, is_lower_end: bool) -> Self {
        Self {
            pos,
            lower_bead_count,
            is_lower_end,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum EdgeType {
    #[default]
    Normal,
    ExtraVoronoi,
    TransitionEnd,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Central {
    Unknown,
    No,
    Yes,
}

#[derive(Debug)]
pub(crate) struct SkeletalEdge {
    pub(crate) edge_type: EdgeType,
    central: Central,
    transitions: Weak<RefCell<Vec<TransitionMiddle>>>,
    transition_ends: Weak<RefCell<Vec<TransitionEnd>>>,
    extrusion_junctions: Weak<RefCell<Vec<ExtrusionJunction>>>,
}

impl Default for SkeletalEdge {
    fn default() -> Self {
        Self::new(EdgeType::Normal)
    }
}

impl SkeletalEdge {
    pub(crate) fn new(edge_type: EdgeType) -> Self {
        Self {
            edge_type,
            central: Central::Unknown,
            transitions: Weak::new(),
            transition_ends: Weak::new(),
            extrusion_junctions: Weak::new(),
        }
    }

    pub(crate) fn is_central(&self) -> bool {
        match self.central {
            Central::Yes => true,
            Central::No => false,
            Central::Unknown => panic!("central state must be set before querying it"),
        }
    }

    pub(crate) fn set_is_central(&mut self, central: bool) {
        self.central = if central { Central::Yes } else { Central::No };
    }

    pub(crate) fn central_is_set(&self) -> bool {
        self.central != Central::Unknown
    }

    pub(crate) fn set_transitions(&mut self, storage: &Shared<TransitionMiddle>) {
        self.transitions = Rc::downgrade(storage);
    }

    pub(crate) fn transitions(&self) -> Option<Shared<TransitionMiddle>> {
        self.transitions.upgrade()
    }

    pub(crate) fn has_transitions(&self, ignore_empty: bool) -> bool {
        self.transitions()
            .is_some_and(|storage| ignore_empty || !storage.borrow().is_empty())
    }

    pub(crate) fn set_transition_ends(&mut self, storage: &Shared<TransitionEnd>) {
        self.transition_ends = Rc::downgrade(storage);
    }

    pub(crate) fn transition_ends(&self) -> Option<Shared<TransitionEnd>> {
        self.transition_ends.upgrade()
    }

    pub(crate) fn has_transition_ends(&self, ignore_empty: bool) -> bool {
        self.transition_ends()
            .is_some_and(|storage| ignore_empty || !storage.borrow().is_empty())
    }

    pub(crate) fn set_extrusion_junctions(&mut self, storage: &Shared<ExtrusionJunction>) {
        self.extrusion_junctions = Rc::downgrade(storage);
    }

    pub(crate) fn extrusion_junctions(&self) -> Option<Shared<ExtrusionJunction>> {
        self.extrusion_junctions.upgrade()
    }

    pub(crate) fn has_extrusion_junctions(&self, ignore_empty: bool) -> bool {
        self.extrusion_junctions()
            .is_some_and(|storage| ignore_empty || !storage.borrow().is_empty())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BeadingPropagation {
    pub(crate) beading: Beading,
    pub(crate) dist_to_bottom_source: i64,
    pub(crate) dist_from_top_source: i64,
    pub(crate) is_upward_propagated_only: bool,
}

impl BeadingPropagation {
    pub(crate) fn new(beading: Beading) -> Self {
        Self {
            beading,
            dist_to_bottom_source: 0,
            dist_from_top_source: 0,
            is_upward_propagated_only: false,
        }
    }
}

#[derive(Debug)]
pub(crate) struct SkeletalJoint {
    pub(crate) distance_to_boundary: i64,
    pub(crate) bead_count: i64,
    pub(crate) transition_ratio: f32,
    beading: Weak<RefCell<BeadingPropagation>>,
}

impl Default for SkeletalJoint {
    fn default() -> Self {
        Self {
            distance_to_boundary: -1,
            bead_count: -1,
            transition_ratio: 0.0,
            beading: Weak::new(),
        }
    }
}

impl SkeletalJoint {
    pub(crate) fn set_beading(&mut self, storage: &Rc<RefCell<BeadingPropagation>>) {
        self.beading = Rc::downgrade(storage);
    }

    pub(crate) fn beading(&self) -> Option<Rc<RefCell<BeadingPropagation>>> {
        self.beading.upgrade()
    }

    pub(crate) fn has_beading(&self) -> bool {
        self.beading.strong_count() > 0
    }
}
