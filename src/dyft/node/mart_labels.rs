use super::mart_vec::MartVec;
use super::offsets::MartDenseOffsets;
use super::offsets::MartSparseOffsets;
use super::EmptyNodes;
use super::IntoMartNode;
use super::MartByteLabels;
use super::MartCursor;
use super::MartDense;
use super::MartOffsets;
use super::MartPointer;
use super::MartTrie;
use super::RawMartPointer;
use super::TrieCounts;
use crate::dyft::*;
use std::convert::AsMut;
use std::convert::AsRef;

/// Wraps a MartByteVec and provides it with a MartByteLabel implementation
#[derive(GetSize)]
pub struct ByteLabels<T> {
    pub(crate) inner: T,
}

impl<T> ByteLabels<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: MartOffsets> ByteLabels<MartVec<T>> {
    fn label_offset(&self, node_id: MartNodeId, idx: MartPointerOffset) -> MartPointerOffset {
        self.inner.header_index(node_id) + 1 + idx
    }

    fn label_impl(&self, node_id: MartNodeId, idx: MartPointerOffset) -> Option<MartByteLabel> {
        self.inner
            .m_nodes
            .get(self.label_offset(node_id, idx))
            .map(|&n| n)
    }

    fn set_label_impl(
        &mut self,
        node_id: MartNodeId,
        idx: MartPointerOffset,
        value: MartByteLabel,
    ) {
        let offset = self.label_offset(node_id, idx);
        self.inner.m_nodes.get_mut(offset).map(|n| *n = value);
    }
}

// the labels are values
// the indices are offsets to the next node
impl<T: IntoMartNode> MartByteLabels for ByteLabels<MartVec<MartSparseOffsets<T>>> {
    fn ptr_offset_iter(&self, node_id: MartNodeId) -> impl Iterator<Item = MartPointerOffset> {
        0..self.inner.header(node_id).into()
    }

    fn ptr_offset(&self, node_id: MartNodeId, idx: MartByteLabel) -> Option<MartPointerOffset> {
        self.inner
            .node_without_header(node_id)
            .iter()
            .take(self.inner.header(node_id).into())
            .position(|&offset| offset == idx)
    }

    fn label_iter(&self, node_id: MartNodeId) -> impl Iterator<Item = MartByteLabel> {
        self.inner
            .node_without_header(node_id)
            .iter()
            .take(self.inner.header(node_id).into())
            .copied()
    }

    fn label(&self, node_id: MartNodeId, offset: MartPointerOffset) -> Option<MartByteLabel> {
        self.label_impl(node_id, offset)
    }

    fn set_label(&mut self, node_id: MartNodeId, idx: MartPointerOffset, value: MartByteLabel) {
        self.set_label_impl(node_id, idx, value)
    }
}

// the labels are indices
// values are offsets to the next node
impl<T: IntoMartNode> MartByteLabels for ByteLabels<MartVec<MartDenseOffsets<T>>> {
    fn ptr_offset_iter(&self, node_id: u32) -> impl Iterator<Item = MartPointerOffset> {
        let node = self.inner.node_without_header(node_id);
        self.label_iter(node_id)
            .map(move |label| MartPointerOffset::from(node[MartPointerOffset::from(label)]))
    }

    fn ptr_offset(&self, node_id: MartNodeId, idx: MartByteLabel) -> Option<MartPointerOffset> {
        self.inner
            .m_nodes
            .get(self.label_offset(node_id, idx.into()))
            .filter(|&&offset| offset != MartDense::<T>::NIL_IDX)
            .map(|&n| n.into())
    }

    fn label_iter(&self, node_id: MartNodeId) -> impl Iterator<Item = MartByteLabel> {
        let node = self.inner.node_without_header(node_id);
        (0..MartDense::<T>::NIL_IDX).filter(|&i| node[MartPointerOffset::from(i)] != MartDense::<T>::NIL_IDX)
    }

    fn label(&self, node_id: u32, idx: MartPointerOffset) -> Option<MartByteLabel> {
        self.label_impl(node_id, idx)
    }

    fn set_label(&mut self, node_id: u32, idx: MartPointerOffset, value: MartByteLabel) {
        self.set_label_impl(node_id, idx, value)
    }
}

impl<T: MartByteLabels> MartByteLabels for EmptyNodes<T> {
    fn ptr_offset_iter(&self, node_id: u32) -> impl Iterator<Item = MartPointerOffset> {
        self.inner.ptr_offset_iter(node_id)
    }

    fn ptr_offset(&self, node_id: MartNodeId, idx: MartByteLabel) -> Option<MartPointerOffset> {
        self.inner.ptr_offset(node_id, idx)
    }

    fn label_iter(&self, node_id: MartNodeId) -> impl Iterator<Item = MartByteLabel> {
        self.inner.label_iter(node_id)
    }

    fn label(&self, node_id: MartNodeId, idx: MartPointerOffset) -> Option<MartByteLabel> {
        self.inner.label(node_id, idx)
    }

    fn set_label(&mut self, ptr: MartNodeId, idx: MartPointerOffset, value: MartByteLabel) {
        self.inner.set_label(ptr, idx, value)
    }
}


impl<T: MartOffsets> MartOffsets for ByteLabels<T> {
    const K: usize = T::K;
    const PTRS_OFFSET: usize = T::PTRS_OFFSET;
    const BYTES: usize = T::BYTES;
}

impl<T: MartTrie> MartTrie for ByteLabels<T> {
    fn create_node(&mut self) -> MartNodeId {
        self.inner.create_node()
    }

    fn next_node_id(&self) -> MartNodeId {
        self.inner.next_node_id()
    }

    fn header(&self, node_id: MartNodeId) -> u8 {
        self.inner.header(node_id)
    }

    fn header_mut(&mut self, node_id: MartNodeId) -> &mut u8 {
        self.inner.header_mut(node_id)
    }

    fn ptr(&self, node_id: MartNodeId, idx: usize) -> Option<MartPointer> {
        self.inner.ptr(node_id, idx)
    }

    fn ptr_slice_mut(&mut self, node_id: MartNodeId, idx: usize) -> Option<&mut RawMartPointer> {
        self.inner.ptr_slice_mut(node_id, idx)
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

impl<T: MartOffsets> TrieCounts for ByteLabels<MartVec<T>> {
    fn num_nodes(&self) -> usize {
        self.inner.num_nodes()
    }

    fn num_edges(&self) -> usize {
        self.inner.num_edges()
    }

    fn num_empty(&self) -> usize {
        self.inner.num_empty()
    }
}

impl<T> AsRef<MartVec<T>> for ByteLabels<MartVec<T>>
where
    T: MartOffsets,
{
    fn as_ref(&self) -> &MartVec<T> {
        self.inner.as_ref()
    }
}

impl<T> AsMut<MartVec<T>> for ByteLabels<MartVec<T>>
where
    T: MartOffsets,
{
    fn as_mut(&mut self) -> &mut MartVec<T> {
        self.inner.as_mut()
    }
}
