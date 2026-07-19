use super::ClosedClipper;
use super::types::{EdgeId, Join, OutputIndex};
use crate::geometry::Point;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MaximaCursor {
    Forward(usize),
    Reverse(usize),
    Exhausted,
}

impl MaximaCursor {
    pub(crate) fn left_to_right(maxima: &[i64], bottom_x: i64, final_top_x: i64) -> Self {
        let index = maxima.partition_point(|&x| x <= bottom_x);
        if maxima.get(index).is_some_and(|&x| x < final_top_x) {
            Self::Forward(index)
        } else {
            Self::Exhausted
        }
    }

    pub(crate) fn right_to_left(maxima: &[i64], bottom_x: i64, final_top_x: i64) -> Self {
        let Some(index) = maxima.partition_point(|&x| x <= bottom_x).checked_sub(1) else {
            return Self::Exhausted;
        };
        if maxima[index] > final_top_x {
            Self::Reverse(index)
        } else {
            Self::Exhausted
        }
    }

    pub(crate) fn pop_before(&mut self, maxima: &[i64], crossing_x: i64) -> Option<i64> {
        match *self {
            Self::Forward(index) if maxima[index] < crossing_x => {
                let x = maxima[index];
                *self = if index + 1 == maxima.len() {
                    Self::Exhausted
                } else {
                    Self::Forward(index + 1)
                };
                Some(x)
            }
            Self::Reverse(index) if maxima[index] > crossing_x => {
                let x = maxima[index];
                *self = index.checked_sub(1).map_or(Self::Exhausted, Self::Reverse);
                Some(x)
            }
            _ => None,
        }
    }
}

impl ClosedClipper {
    pub(super) fn collect_strict_maximum(&mut self, edge: EdgeId) {
        if self.options.strictly_simple {
            let x = self.edges.edge(edge).top.x();
            self.maxima.push(x);
            #[cfg(test)]
            self.collected_maxima_for_test.push(x);
        }
    }

    pub(super) fn prepare_strict_maxima(&mut self) {
        self.maxima.sort_unstable();
    }

    pub(super) fn clear_strict_maxima(&mut self) {
        self.maxima.clear();
    }

    pub(super) fn insert_strict_horizontal_maxima(
        &mut self,
        horizontal: EdgeId,
        cursor: &mut MaximaCursor,
        crossing_x: i64,
    ) {
        while let Some(x) = cursor.pop_before(&self.maxima, crossing_x) {
            let edge = *self.edges.edge(horizontal);
            if matches!(edge.output, OutputIndex::Assigned(_)) && edge.wind_delta != 0 {
                self.add_out_point(horizontal, Point::new(x, edge.bottom.y()));
            }
        }
    }

    pub(super) fn join_strict_top_touch(&mut self, edge: EdgeId) {
        if !self.options.strictly_simple {
            return;
        }
        let current = *self.edges.edge(edge);
        let Some(previous) = current.previous_in_ael else {
            return;
        };
        let previous_edge = *self.edges.edge(previous);
        if matches!(current.output, OutputIndex::Assigned(_))
            && current.wind_delta != 0
            && matches!(previous_edge.output, OutputIndex::Assigned(_))
            && previous_edge.current.x() == current.current.x()
            && previous_edge.wind_delta != 0
        {
            let point = current.current;
            let first = self.add_out_point(previous, point);
            let second = self.add_out_point(edge, point);
            self.joins.push(Join {
                first,
                second,
                offset: point,
            });
        }
    }

    #[cfg(test)]
    pub(crate) fn seed_strict_maxima_for_test(&mut self, maxima: &[i64]) {
        self.maxima.extend_from_slice(maxima);
    }

    #[cfg(test)]
    pub(crate) fn strict_maxima_for_test(&self) -> &[i64] {
        &self.maxima
    }

    #[cfg(test)]
    pub(crate) fn collected_strict_maxima_for_test(&self) -> &[i64] {
        &self.collected_maxima_for_test
    }
}
