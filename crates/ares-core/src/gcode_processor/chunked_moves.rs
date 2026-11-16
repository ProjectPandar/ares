use super::MoveVertex;
use std::ops::{Index, IndexMut, Range};

const CHUNK_LEN: usize = 1_048_576;

/// Processor moves in global record order, with independently owned fixed-capacity chunks.
#[derive(Debug, Default, PartialEq)]
pub struct ChunkedMoves {
    chunks: Vec<Vec<MoveVertex>>,
    len: usize,
}

impl ChunkedMoves {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn get(&self, index: usize) -> Option<&MoveVertex> {
        self.chunks.get(index / CHUNK_LEN)?.get(index % CHUNK_LEN)
    }
    pub fn get_mut(&mut self, index: usize) -> Option<&mut MoveVertex> {
        self.chunks
            .get_mut(index / CHUNK_LEN)?
            .get_mut(index % CHUNK_LEN)
    }
    pub fn first(&self) -> Option<&MoveVertex> {
        self.get(0)
    }
    pub fn last(&self) -> Option<&MoveVertex> {
        self.chunks.last()?.last()
    }
    pub fn push(&mut self, vertex: MoveVertex) {
        if self.len.is_multiple_of(CHUNK_LEN) {
            self.chunks.push(Vec::with_capacity(CHUNK_LEN));
        }
        self.chunks.last_mut().unwrap().push(vertex);
        self.len += 1;
    }
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &MoveVertex> + ExactSizeIterator {
        (0..self.len).map(|index| &self[index])
    }
    pub fn chunks(&self) -> impl Iterator<Item = &[MoveVertex]> {
        self.chunks.iter().map(Vec::as_slice)
    }
    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut MoveVertex> {
        self.chunks.iter_mut().flatten()
    }
    pub fn pairs(&self) -> impl Iterator<Item = [&MoveVertex; 2]> {
        self.iter().zip(self.iter().skip(1)).map(|(a, b)| [a, b])
    }
    pub fn range(&self, range: Range<usize>) -> impl Iterator<Item = &MoveVertex> {
        let start = range.start;
        self.chunks[start / CHUNK_LEN..]
            .iter()
            .enumerate()
            .flat_map(move |(i, chunk)| chunk[if i == 0 { start % CHUNK_LEN } else { 0 }..].iter())
            .take(range.end - start)
    }
    pub fn partition_point(&self, mut predicate: impl FnMut(&MoveVertex) -> bool) -> usize {
        self.partition_range(0..self.len, &mut predicate)
    }
    pub fn partition_range(
        &self,
        range: Range<usize>,
        mut predicate: impl FnMut(&MoveVertex) -> bool,
    ) -> usize {
        let mut left = range.start;
        let mut right = range.end;
        while left < right {
            let middle = left + (right - left) / 2;
            if predicate(&self[middle]) {
                left = middle + 1;
            } else {
                right = middle;
            }
        }
        left - range.start
    }
    pub fn remove(&mut self, index: usize) -> MoveVertex {
        let removed = self[index];
        let first = index / CHUNK_LEN;
        for chunk in first..self.chunks.len() {
            let start = if chunk == first { index % CHUNK_LEN } else { 0 };
            let carry = self.chunks.get(chunk + 1).map(|next| next[0]);
            self.chunks[chunk].copy_within(start + 1.., start);
            if let Some(carry) = carry {
                *self.chunks[chunk].last_mut().unwrap() = carry;
            }
        }
        self.chunks.last_mut().unwrap().pop();
        self.len -= 1;
        if self.chunks.last().unwrap().is_empty() {
            self.chunks.pop();
        }
        removed
    }
    pub fn resize(&mut self, len: usize, value: MoveVertex) {
        while self.len < len {
            self.push(value);
        }
        while self.len > len {
            self.chunks.last_mut().unwrap().pop();
            self.len -= 1;
            if self.chunks.last().unwrap().is_empty() {
                self.chunks.pop();
            }
        }
    }
    /// Back-to-front suffix movement used by actual-speed insertion.
    pub(crate) fn copy_backwards(&mut self, range: Range<usize>, destination: usize) {
        let mut end = range.end;
        let mut to = destination + range.len();
        while end > range.start {
            let source_chunk = (end - 1) / CHUNK_LEN;
            let target_chunk = (to - 1) / CHUNK_LEN;
            let source_end = (end - 1) % CHUNK_LEN + 1;
            let target_end = (to - 1) % CHUNK_LEN + 1;
            let count = (end - range.start).min(source_end).min(target_end);
            if source_chunk == target_chunk {
                self.chunks[source_chunk]
                    .copy_within(source_end - count..source_end, target_end - count);
            } else {
                let (source, target) = self.chunks.split_at_mut(target_chunk);
                target[0][target_end - count..target_end]
                    .copy_from_slice(&source[source_chunk][source_end - count..source_end]);
            }
            end -= count;
            to -= count;
        }
    }
    pub fn into_chunks(self) -> impl Iterator<Item = Vec<MoveVertex>> {
        self.chunks.into_iter()
    }
}

impl Clone for ChunkedMoves {
    fn clone(&self) -> Self {
        self.iter().copied().collect()
    }
}

impl Index<usize> for ChunkedMoves {
    type Output = MoveVertex;
    fn index(&self, index: usize) -> &Self::Output {
        &self.chunks[index / CHUNK_LEN][index % CHUNK_LEN]
    }
}
impl IndexMut<usize> for ChunkedMoves {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.chunks[index / CHUNK_LEN][index % CHUNK_LEN]
    }
}
impl FromIterator<MoveVertex> for ChunkedMoves {
    fn from_iter<I: IntoIterator<Item = MoveVertex>>(iter: I) -> Self {
        let mut out = Self::default();
        for vertex in iter {
            out.push(vertex);
        }
        out
    }
}
impl From<Vec<MoveVertex>> for ChunkedMoves {
    fn from(values: Vec<MoveVertex>) -> Self {
        values.into_iter().collect()
    }
}
impl<'a> IntoIterator for &'a ChunkedMoves {
    type Item = &'a MoveVertex;
    type IntoIter = std::iter::Flatten<std::slice::Iter<'a, Vec<MoveVertex>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.chunks.iter().flatten()
    }
}
impl<'a> IntoIterator for &'a mut ChunkedMoves {
    type Item = &'a mut MoveVertex;
    type IntoIter = std::iter::Flatten<std::slice::IterMut<'a, Vec<MoveVertex>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.chunks.iter_mut().flatten()
    }
}
impl IntoIterator for ChunkedMoves {
    type Item = MoveVertex;
    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Vec<MoveVertex>>>;
    fn into_iter(self) -> Self::IntoIter {
        self.chunks.into_iter().flatten()
    }
}
