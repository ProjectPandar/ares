#[derive(Clone, Copy, Debug)]
pub(super) struct EndPoint {
    pub(super) candidate: Option<usize>,
    pub(super) distance: f64,
    pub(super) heap_index: usize,
    pub(super) chain_id: usize,
    pub(super) edge_out: Option<usize>,
}

impl EndPoint {
    pub(super) fn new() -> Self {
        Self {
            candidate: None,
            distance: f64::MAX,
            heap_index: usize::MAX,
            chain_id: 0,
            edge_out: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Chain {
    segment_count: usize,
    cost: f64,
    flipped_cost: f64,
    begin: usize,
    end: usize,
    equivalent_with: usize,
}

pub(super) struct Chains {
    chains: Vec<Chain>,
    last_id: usize,
}

impl Chains {
    pub(super) fn new(capacity: usize) -> Self {
        let mut chains = Vec::with_capacity(capacity + 1);
        chains.push(Chain::default());
        Self { chains, last_id: 0 }
    }

    pub(super) fn next_id(&mut self) -> usize {
        self.last_id += 1;
        self.chains.push(Chain {
            equivalent_with: self.last_id,
            ..Chain::default()
        });
        self.last_id
    }

    pub(super) fn equivalent(&mut self, mut id: usize) -> usize {
        if id == 0 {
            return 0;
        }
        let original = id;
        loop {
            let lower = self.chains[id].equivalent_with;
            if lower == id {
                self.chains[original].equivalent_with = lower;
                return lower;
            }
            id = lower;
        }
    }

    pub(super) fn merge(&mut self, first: usize, second: usize) -> usize {
        let id = match (first, second) {
            (0, 0) => return self.next_id(),
            (0, second) => second,
            (first, 0) => first,
            (first, second) => first.min(second),
        };
        self.chains[first].equivalent_with = id;
        self.chains[second].equivalent_with = id;
        id
    }

    pub(super) fn flip_penalty(&mut self, id: usize) -> f64 {
        let id = self.equivalent(id);
        self.chains[id].flipped_cost - self.chains[id].cost
    }

    pub(super) fn begin(&self, id: usize) -> usize {
        self.chains[id].begin
    }

    pub(super) fn end(&self, id: usize) -> usize {
        self.chains[id].end
    }

    pub(super) fn assign(
        &mut self,
        id: usize,
        chain_ids: (Option<usize>, Option<usize>),
        endpoints: (usize, usize),
        positions: &[[f64; 2]],
    ) {
        let (first, second) = endpoints;
        let first_chain = chain_ids.0.map(|id| self.chains[id]);
        let second_chain = chain_ids.1.map(|id| self.chains[id]);
        let begin = first_chain.map_or(first ^ 1, |chain| {
            if chain.begin == first {
                chain.end
            } else {
                chain.begin
            }
        });
        let end = second_chain.map_or(second ^ 1, |chain| {
            if chain.begin == second {
                chain.end
            } else {
                chain.begin
            }
        });
        self.chains[id] = Chain {
            segment_count: first_chain.map_or(1, |chain| chain.segment_count)
                + second_chain.map_or(1, |chain| chain.segment_count),
            cost: first_chain.map_or(0.0, |chain| chain.cost)
                + second_chain.map_or(0.0, |chain| chain.cost)
                + distance(positions[first], positions[second]),
            flipped_cost: first_chain.map_or(0.0, |chain| chain.flipped_cost)
                + second_chain.map_or(0.0, |chain| chain.flipped_cost)
                + distance(positions[first ^ 1], positions[second ^ 1]),
            begin,
            end,
            equivalent_with: id,
        };
    }

    pub(super) fn flip(&mut self, id: usize, endpoints: &mut [EndPoint]) {
        let chain = &mut self.chains[id];
        let old_begin = chain.begin;
        let old_end = chain.end;
        let mut endpoint = old_begin;
        let mut previous = None;
        loop {
            let endpoint_end = endpoint ^ 1;
            let next = endpoints[endpoint_end].edge_out;
            endpoints[endpoint_end].edge_out = previous;
            if let Some(previous) = previous {
                endpoints[previous].edge_out = Some(endpoint_end);
            }
            previous = Some(endpoint);
            let Some(next) = next else {
                break;
            };
            endpoint = next;
        }
        endpoints[previous.expect("chain is nonempty")].edge_out = None;
        std::mem::swap(&mut chain.cost, &mut chain.flipped_cost);
        let new_begin = old_begin ^ 1;
        let new_end = old_end ^ 1;
        let begin_id = endpoints[old_begin].chain_id;
        endpoints[old_begin].chain_id = endpoints[new_begin].chain_id;
        endpoints[new_begin].chain_id = begin_id;
        let end_id = endpoints[old_end].chain_id;
        endpoints[old_end].chain_id = endpoints[new_end].chain_id;
        endpoints[new_end].chain_id = end_id;
        chain.begin = new_begin;
        chain.end = new_end;
    }
}

pub(super) fn distance(first: [f64; 2], second: [f64; 2]) -> f64 {
    let x = second[0] - first[0];
    let y = second[1] - first[1];
    x.hypot(y)
}
