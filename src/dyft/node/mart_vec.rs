use super::offsets::MartSparseOffsets;
use super::traits::MartOffsets;
use super::traits::MartTrie;
use super::IntoMartNode;
use super::MartCursor;
use super::MartPointer;
use super::RawMartPointer;
use super::TrieCounts;
use super::MART_NIL_LABEL;
use super::MART_PTR_SIZE;
use crate::dyft::InNodeStatistics;
use crate::dyft::InNodeStats;
use crate::dyft::MartNodeId;
use get_size::GetSize;
use num_traits::AsPrimitive;
use std::array::TryFromSliceError;
use std::convert::AsMut;
use std::convert::AsRef;
use std::marker::PhantomData;

const RAW_MART_NIL_PTR: RawMartPointer = [255; MART_PTR_SIZE];

/// A byte vector for storing nodes in a tree
#[derive(Debug, GetSize)]
pub struct MartVec<T> {
    pub m_nodes: Vec<u8>,
    t_phantom: PhantomData<T>,
}

impl<T: MartOffsets> MartOffsets for MartVec<T> {
    const K: usize = T::K;
    const PTRS_OFFSET: usize = T::PTRS_OFFSET;
    const BYTES: usize = T::BYTES;
}

impl<T: MartOffsets> MartVec<T> {
    pub fn new(n: usize) -> Self {
        Self {
            m_nodes: Vec::with_capacity(n),
            t_phantom: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.m_nodes.len() / T::BYTES
    }

    pub fn node(&self, node_id: MartNodeId) -> &[u8] {
        let start = self.header_index(node_id);
        &self.m_nodes[start..start + T::BYTES]
    }

    pub fn node_without_header(&self, node_id: MartNodeId) -> &[u8] {
        let start = self.header_index(node_id) + 1;
        &self.m_nodes[start..start + T::BYTES - 1]
    }

    pub fn node_mut(&mut self, node_id: MartNodeId) -> &mut [u8] {
        let start = self.header_index(node_id);
        &mut self.m_nodes[start..start + T::BYTES]
    }

    pub fn bytes(&self) -> &[u8] {
        &self.m_nodes
    }
}

impl<T: IntoMartNode> MartVec<MartSparseOffsets<T>> {
    pub fn byte_labels_slice(&self, node_id: MartNodeId) -> [u8; T::BYTES] {
        let start = self.header_index(node_id) + 1;
        self.m_nodes[start..start + T::BYTES]
            .try_into()
            .expect("byte_labels_slice")
    }
}

impl<T: MartOffsets> MartTrie for MartVec<T> {
    fn create_node(&mut self) -> MartNodeId {
        let node_id = self.next_node_id();
        self.m_nodes
            .extend_from_slice(vec![u8::MAX; Self::BYTES].as_slice());
        *self.header_mut(node_id) = 0;
        node_id
    }

    fn next_node_id(&self) -> u32 {
        (self.m_nodes.len() / T::BYTES).as_()
    }

    fn header(&self, node_id: MartNodeId) -> u8 {
        self.m_nodes[self.header_index(node_id)]
    }

    fn header_mut(&mut self, node_id: MartNodeId) -> &mut u8 {
        let idx = self.header_index(node_id);
        &mut self.m_nodes[idx]
    }

    fn ptr(&self, node_id: MartNodeId, idx: usize) -> Option<MartPointer> {
        let ptr = self.ptr_index(node_id, idx);
        match self.m_nodes[ptr..ptr + MART_PTR_SIZE].try_into() {
            Result::<RawMartPointer, TryFromSliceError>::Ok(ptr)
                if ptr[..4] != RAW_MART_NIL_PTR[..4] =>
            {
                Some(MartPointer::from(&ptr))
            }
            _ => None,
        }
    }

    fn ptr_slice_mut(&mut self, node_id: MartNodeId, idx: usize) -> Option<&mut RawMartPointer> {
        let ptr = self.ptr_index(node_id, idx);
        <&mut [u8] as TryInto<&mut RawMartPointer>>::try_into(
            &mut self.m_nodes[ptr..ptr + MART_PTR_SIZE],
        )
        .ok()
    }

    fn update_src(&mut self, mc: &MartCursor) {
        self.set_ptr(mc.pptr.nid(), mc.offset, mc.nptr());
    }

    fn ptr_iter<'a, I>(
        &'a self,
        node_id: MartNodeId,
        label_idxs: I,
    ) -> impl Iterator<Item = MartPointer> + 'a
    where
        I: IntoIterator<Item = usize> + 'a,
    {
        label_idxs
            .into_iter()
            .filter_map(move |i| self.ptr(node_id, i))
    }
}

impl<T: MartOffsets> TrieCounts for MartVec<T> {
    fn num_nodes(&self) -> usize {
        self.m_nodes.len() / T::BYTES
    }

    fn num_edges(&self) -> usize {
        self.m_nodes
            .iter()
            .step_by(T::BYTES)
            .map(|&node| <u8 as Into<usize>>::into(node))
            .sum()
    }

    fn num_empty(&self) -> usize {
        self.m_nodes
            .chunks(T::BYTES)
            .into_iter()
            .map(|v| {
                v.iter()
                    .skip(1)
                    .array_chunks::<MART_PTR_SIZE>()
                    .filter(|&ptr| ptr.into_iter().all(|&b| b == MART_NIL_LABEL))
                    .count()
            })
            .sum()
    }
}

impl<T: MartOffsets> InNodeStatistics for MartVec<T> {
    fn innode_stats(&self) -> InNodeStats {
        InNodeStats {
            k: T::K,
            num: self.num_nodes(),
            empty: self.num_empty(),
        }
    }
}

/// AsRef and AsMut trait implementations for MartByteVec
/// This allows for the use of Vec<u8> and &[u8] methods
/// for structs containing MartByteVec.
impl<T: MartOffsets> AsRef<Self> for MartVec<T> {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<T: MartOffsets> AsMut<Self> for MartVec<T> {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
