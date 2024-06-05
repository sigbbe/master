use get_size::GetSize;

use super::mart_vec::MartVec;
use super::traits::MartEmptyNodeStack;
use super::traits::MartOffsets;
use super::ByteLabels;
use super::MartCursor;
use super::MartPointer;
use super::MartTrie;
use super::RawMartPointer;
use super::TrieCounts;
use crate::dyft::InNodeStatistics;
use crate::dyft::InNodeStats;
use crate::dyft::MartNodeId;
use crate::dyft::PopulationStatistics;
use crate::dyft::PopulationStats;
use std::array::TryFromSliceError;
use std::collections::HashSet;
use std::convert::AsRef;
use std::ops::Range;

/// Wraps a MartByteVec and provides it with a stack of empty nodes.
/// This struct provides the empty_node_push and empty_node_pop methods,
/// such that MartSparse and MartDense can push a node to the empty stack
/// when extracting, and pop from the empty stack when inserting. The
/// stack's data is kept as a doubly linked list in the underlying MartByteVec.
///
#[derive(GetSize)]
pub struct EmptyNodes<T> {
    // head node id
    m_head_nid: Option<u32>,
    // number of empty nodes
    m_num_empty: usize,
    // phantom data for type of node
    pub(crate) inner: T,
}

impl<T> EmptyNodes<ByteLabels<MartVec<T>>>
where
    T: MartOffsets, // AsRef<MartVec<T>> + AsMut<MartVec<T>>,
{
    pub fn iter_empty(&self) -> impl Iterator<Item = MartNodeId> {
        let mut v = HashSet::new();
        if let Some(head) = self.head() {
            v.insert(head);
            while let Some(next) = self.next_ref(head) {
                if v.contains(&next) {
                    break;
                }
                v.insert(next);
            }
        }
        v.into_iter()
    }
}

impl<T> EmptyNodes<T> {
    pub fn new(inner: T) -> Self {
        Self {
            m_head_nid: None,
            m_num_empty: 0,
            inner,
        }
    }
    pub fn head(&self) -> Option<u32> {
        self.m_head_nid
    }
    pub fn num_empty(&self) -> usize {
        self.m_num_empty
    }
}

impl<T: MartOffsets> MartOffsets for EmptyNodes<T> {
    const K: usize = T::K;
    const PTRS_OFFSET: usize = T::PTRS_OFFSET;
    const BYTES: usize = T::BYTES;
}

impl<T> EmptyNodes<ByteLabels<MartVec<T>>>
where
    T: MartOffsets,
{
    fn map_node_ref(
        &self,
        node_id_ref: Result<u32, TryFromSliceError>,
        node_id: u32,
    ) -> Option<u32> {
        match node_id_ref {
            Ok(nid) if nid == node_id => None,
            Ok(nid) => Some(nid),
            Err(_) => None,
        }
    }
    fn node_ref(
        &self,
        node_id: MartNodeId,
        idx: Range<usize>,
    ) -> Result<MartNodeId, TryFromSliceError> {
        self.inner.as_ref().node(node_id)[idx]
            .try_into()
            .map(|bytes| u32::from_le_bytes(bytes))
    }

    fn set_node_ref(&mut self, node_id: MartNodeId, idx: Range<usize>, value: MartNodeId) {
        let node = self.inner.as_mut().node_mut(node_id);
        node[idx].copy_from_slice(&value.to_le_bytes());
    }

    fn prev_ref(&self, node_id: MartNodeId) -> Option<MartNodeId> {
        self.map_node_ref(self.node_ref(node_id, 0..4), node_id)
    }
    fn next_ref(&self, node_id: MartNodeId) -> Option<MartNodeId> {
        self.map_node_ref(self.node_ref(node_id, 4..8), node_id)
    }
    fn set_prev_ref(&mut self, node_id: MartNodeId, prev_node_id: MartNodeId) {
        self.set_node_ref(node_id, 0..4, prev_node_id);
    }
    fn set_next_ref(&mut self, node_id: MartNodeId, next_node_id: MartNodeId) {
        self.set_node_ref(node_id, 4..8, next_node_id)
    }
    #[allow(dead_code)]
    fn reset_node(&mut self, node_id: u32) {
        self.inner.as_mut().node_mut(node_id).fill(u8::MAX);
    }

    pub fn empty_nodes_iter(&self) -> impl Iterator<Item = MartNodeId> {
        let mut iter = vec![];
        if let Some(head) = self.head() {
            iter.push(head * Self::BYTES as MartNodeId);
            let mut next = self.next_ref(head);
            while let Some(n) = next {
                if n != head {
                    iter.push(n * Self::BYTES as MartNodeId);
                    next = self.next_ref(n);
                } else {
                    break;
                }
            }
        }
        iter.into_iter()
    }
}

impl<T> MartEmptyNodeStack<u32> for EmptyNodes<ByteLabels<MartVec<T>>>
where
    T: MartOffsets,
{
    fn empty_node_push(&mut self, value: u32) {
        if let Some(head) = self.m_head_nid {
            let prev = self.prev_ref(head).unwrap_or(head);
            self.set_prev_ref(value, prev);
            self.set_next_ref(value, head);
            self.set_next_ref(prev, value);
            self.set_prev_ref(head, value)
        } else {
            self.set_prev_ref(value, value);
            self.set_next_ref(value, value);
            self.m_head_nid = Some(value);
        }
        self.m_num_empty += 1;
    }

    fn empty_node_pop(&mut self) -> Option<u32> {
        if let Some(head) = self.m_head_nid {
            // TODO: thread 'main' panicked at master/src/dyft/mart_vec/linked_list.rs:110:13:
            // attempt to subtract with overflow
            let prev = self.prev_ref(head);
            if let Some(next) = self.next_ref(head) {
                self.m_head_nid = Some(next);
                let prev = prev.unwrap_or(next);
                self.set_prev_ref(next, prev);
                self.set_next_ref(prev, next);
            } else {
                self.m_head_nid = None;
                self.m_num_empty = 0;
            }
            // reset rest of node
            // TODO: find out if we need to reset the node
            self.reset_node(head);
            Some(head)
        } else {
            None
        }
    }
}

impl<T: MartTrie> MartTrie for EmptyNodes<T> {
    fn ptr(&self, node_id: MartNodeId, idx: usize) -> Option<MartPointer> {
        self.inner.ptr(node_id, idx)
    }
    fn ptr_slice_mut(&mut self, node_id: MartNodeId, idx: usize) -> Option<&mut RawMartPointer> {
        self.inner.ptr_slice_mut(node_id, idx)
    }
    fn create_node(&mut self) -> MartNodeId {
        self.inner.create_node()
    }
    fn next_node_id(&self) -> u32 {
        self.inner.next_node_id()
    }
    fn header(&self, node_id: MartNodeId) -> u8 {
        self.inner.header(node_id)
    }
    fn header_mut(&mut self, node_id: MartNodeId) -> &mut u8 {
        self.inner.header_mut(node_id)
    }
    fn update_src(&mut self, mc: &MartCursor) {
        self.inner.update_src(mc)
    }

    fn ptr_iter<'a, I>(
        &'a self,
        node_id: MartNodeId,
        label_idxs: I,
    ) -> impl Iterator<Item = MartPointer> + 'a
    where
        I: IntoIterator<Item = usize> + 'a,
    {
        self.inner.ptr_iter(node_id, label_idxs)
    }
}

impl<T> TrieCounts for EmptyNodes<ByteLabels<MartVec<T>>>
where
    T: MartOffsets,
{
    fn num_nodes(&self) -> usize {
        self.inner.as_ref().num_nodes()
    }

    fn num_edges(&self) -> usize {
        let m_num: u32 = self
            .inner
            .as_ref()
            .num_nodes()
            .try_into()
            .expect("overflow in MartByteVec::num_edges_total");
        let empty = self.iter_empty().collect::<HashSet<_>>();
        let mut count = 0;
        for node_id in (0..m_num).into_iter().filter(|i| !empty.contains(i)) {
            count += <u8 as Into<usize>>::into(self.inner.as_ref().header(node_id));
        }
        count
    }

    fn num_empty(&self) -> usize {
        self.iter_empty().count()
    }
}

impl<T: MartOffsets> PopulationStatistics for EmptyNodes<ByteLabels<MartVec<T>>> {
    fn population_stats(&self) -> PopulationStats {
        let empty_nodes: HashSet<MartNodeId> = self.empty_nodes_iter().collect();
        let mut nodes = vec![0; T::BYTES + 1];

        for i in 0..self.num_nodes().try_into().unwrap() {
            if let Some(&node_id) = empty_nodes.get(&i) {
                let header: usize = self.inner.header(node_id).into();
                if header <= T::BYTES {
                    nodes[header] += 1;
                }
            }
        }

        let sum = nodes.iter().sum();

        PopulationStats {
            k: T::K,
            sum,
            nodes: nodes.into_iter().filter(|&v| v != 0).collect(),
        }
    }
}

impl<T: MartOffsets> InNodeStatistics for EmptyNodes<ByteLabels<MartVec<T>>> {
    fn innode_stats(&self) -> InNodeStats {
        InNodeStats {
            k: T::K,
            num: self.num_nodes(),
            empty: self.num_empty(),
        }
    }
}
