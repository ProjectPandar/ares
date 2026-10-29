use super::{Issue, State};
use boostvoronoi::{
    BvError,
    prelude::{Cell, CellIndex, Edge, EdgeIndex, VertexIndex},
};

// Moving indexed topology preserves categories/anchors/flags without rebasing
// C++ pointers. Only a repaired diagram needs separately owned coordinates.
pub(crate) struct Diagram {
    topology: boostvoronoi::prelude::Diagram,
    pub(super) local: Option<Vec<[f64; 2]>>,
    pub(super) issue: Issue,
    pub(super) state: State,
}

pub(crate) struct Vertex<'a> {
    pub(in crate::geometry) metadata: &'a boostvoronoi::prelude::Vertex,
    position: [f64; 2],
}

impl Vertex<'_> {
    pub(crate) fn x(&self) -> f64 {
        self.position[0]
    }
    pub(crate) fn y(&self) -> f64 {
        self.position[1]
    }
    pub(crate) fn get_incident_edge(&self) -> Result<EdgeIndex, BvError> {
        self.metadata.get_incident_edge()
    }
}

impl Diagram {
    pub(super) fn new(topology: boostvoronoi::prelude::Diagram, modified: bool) -> Self {
        let local = modified.then(|| topology.vertices().iter().map(|v| [v.x(), v.y()]).collect());
        Self {
            topology,
            local,
            issue: Issue::Unknown,
            state: State::Unknown,
        }
    }
    pub(crate) fn is_valid(&self) -> bool {
        self.state != State::Unsuccessful
    }
    pub(crate) fn cells(&self) -> &[Cell] {
        self.topology.cells()
    }
    pub(crate) fn edges(&self) -> &[Edge] {
        self.topology.edges()
    }
    pub(crate) fn num_edges(&self) -> usize {
        self.topology.num_edges()
    }
    pub(crate) fn num_cells(&self) -> usize {
        self.topology.num_cells()
    }
    pub(crate) fn num_vertices(&self) -> usize {
        self.topology.num_vertices()
    }
    pub(crate) fn edge_is_finite(&self, id: EdgeIndex) -> Result<bool, BvError> {
        self.topology.edge_is_finite(id)
    }
    pub(crate) fn vertex(&self, id: VertexIndex) -> Result<Vertex<'_>, BvError> {
        let original = self.topology.vertex(id)?;
        let position = self
            .local
            .as_ref()
            .map_or_else(|| [original.x(), original.y()], |v| v[id.usize()]);
        Ok(Vertex {
            metadata: original,
            position,
        })
    }
    pub(crate) fn vertices(&self) -> impl ExactSizeIterator<Item = Vertex<'_>> {
        self.topology
            .vertices()
            .iter()
            .map(|v| self.vertex(v.get_id()).unwrap())
    }
    pub(crate) fn edge(&self, id: EdgeIndex) -> Result<&Edge, BvError> {
        self.topology.edge(id)
    }
    pub(crate) fn cell(&self, id: CellIndex) -> Result<&Cell, BvError> {
        self.topology.cell(id)
    }
    pub(crate) fn edge_index_unchecked(&self, id: usize) -> EdgeIndex {
        self.topology.edge_index_unchecked(id)
    }
    pub(crate) fn edge_get_cell(&self, id: EdgeIndex) -> Result<CellIndex, BvError> {
        self.topology.edge_get_cell(id)
    }
    pub(crate) fn edge_get_twin(&self, id: EdgeIndex) -> Result<EdgeIndex, BvError> {
        self.topology.edge_get_twin(id)
    }
    pub(crate) fn edge_get_next(&self, id: EdgeIndex) -> Result<EdgeIndex, BvError> {
        self.topology.edge_get_next(id)
    }
    pub(crate) fn edge_get_vertex0(&self, id: EdgeIndex) -> Result<Option<VertexIndex>, BvError> {
        self.topology.edge_get_vertex0(id)
    }
    pub(crate) fn edge_get_vertex1(&self, id: EdgeIndex) -> Result<Option<VertexIndex>, BvError> {
        self.topology.edge_get_vertex1(id)
    }
    pub(crate) fn edge_rot_next(&self, id: EdgeIndex) -> Option<EdgeIndex> {
        self.topology.edge_rot_next(id)
    }
    pub(crate) fn edge_as_line(&self, id: EdgeIndex) -> Result<[f64; 4], BvError> {
        let a = self.vertex(
            self.edge_get_vertex0(id)?
                .ok_or_else(|| BvError::IdError("infinite edge".into()))?,
        )?;
        let b = self.vertex(
            self.edge_get_vertex1(id)?
                .ok_or_else(|| BvError::IdError("infinite edge".into()))?,
        )?;
        Ok([a.x(), a.y(), b.x(), b.y()])
    }
    pub(super) fn position(&self, id: VertexIndex) -> [f64; 2] {
        let v = self.vertex(id).unwrap();
        [v.x(), v.y()]
    }
}
