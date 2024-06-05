// https://github.com/kampersanda/dyft/blob/9c675b60817542a3fccbfb76567f9b6a4ec935a9/include/mart_array_sparse.hpp

use super::common::MartCursor;
use super::common::MartEdge;
use super::common::MartInsertFlags;
use super::common::MartPointer;
use super::mart_vec::MartVec;
use super::node_types::IntoMartNode;
use super::offsets::MartSparseOffsets;
use super::traits::MartEmptyNodeStack;
use super::traits::MartOffsets;
use super::traits::MartTrie;
use super::ByteLabels;
use super::EmptyNodes;
use super::MartArray;
use super::MartArrayExtractable;
use super::MartByteLabels;
use super::MartChildren;
use super::MartLinearScan;
use super::TrieCounts;
use crate::dyft::InNodeStatistics;
use crate::dyft::InNodeStats;
use crate::dyft::MartNodeId;
use crate::dyft::PopulationStatistics;
use crate::dyft::PopulationStats;
use core::simd::Simd;
use get_size::GetSize;
use nalgebra::SimdBool;
use num_traits::Zero;
use std::simd::LaneCount;
use std::simd::SupportedLaneCount;

#[derive(GetSize)]
pub struct MartSparse<T>
where
    T: IntoMartNode,
{
    inner: EmptyNodes<ByteLabels<MartVec<MartSparseOffsets<T>>>>,
}

impl<'a, T: IntoMartNode> Default for MartSparse<T> {
    fn default() -> Self {
        assert!(2 <= T::BYTES && T::BYTES <= 255, "K must be in [2, 255]");
        Self {
            inner: EmptyNodes::new(ByteLabels::new(MartVec::new(MartSparseOffsets::<T>::BYTES))),
        }
    }
}

impl<T: IntoMartNode> MartSparse<T>
where
    LaneCount<{ T::BYTES }>: SupportedLaneCount,
{
    pub fn internals(&self) -> &MartVec<MartSparseOffsets<T>> {
        &self.inner.inner.inner
    }

    #[allow(dead_code)]
    fn adaptive_search(&self, node_id: MartNodeId, label: u8) -> Option<MartPointer> {
        let m_num: usize = self.inner.header(node_id).into();
        self.inner
            .label_iter(node_id)
            .take(m_num)
            .position(|byte_label| byte_label == label)
            .map(|i| self.inner.ptr(node_id, i))
            .unwrap_or(None)
    }

    fn adaptive_search_simd(&self, node_id: MartNodeId, label: u8) -> Option<MartPointer> {
        let m_num: usize = self.inner.header(node_id).into();
        let simd =
            Simd::<u8, { T::BYTES }>::from_array(self.inner.inner.inner.byte_labels_slice(node_id));

        let mask = (simd == Simd::<u8, { T::BYTES }>::splat(label))
            .bitmask()
            .trailing_zeros()
            .try_into()
            .expect("MartSparse::adaptive_search: mask is too large");

        if mask < m_num {
            self.inner.ptr(node_id, mask)
        } else {
            None
        }
    }
}

impl<T: IntoMartNode> MartArray<T> for MartSparse<T>
where
    LaneCount<{ T::BYTES }>: SupportedLaneCount,
{
    fn insert_ptr(
        &mut self,
        mc: &mut MartCursor,
        label: u8,
        new_ptr: &MartPointer,
    ) -> MartInsertFlags {
        assert_eq!(mc.ntype(), T::TYPE_ID);
        let node_id = mc.nptr.nid();
        let header = self.inner.header(node_id).into();
        assert!(
            T::BYTES >= header,
            "MartNode{} cannot have {header} children",
            T::BYTES
        );
        if let Some(offset) = self.inner.ptr_offset(node_id, label) {
            let ptr = self
                .inner
                .ptr(node_id, offset)
                .expect("MartArraySparse::insert_ptr: label exists but ptr is None");
            mc.update(offset, &ptr);
            MartInsertFlags::MartFound
        } else if header < T::BYTES {
            self.inner.header_inc(node_id);
            self.inner.set_label(node_id, header, label);
            self.inner.set_ptr(node_id, header, new_ptr);
            mc.update(header, &new_ptr);
            MartInsertFlags::MartInserted
        } else {
            MartInsertFlags::MartNeededToExpand
        }
    }

    fn make_node(&mut self) -> MartPointer {
        // reuse empty node
        if let Some(head) = self.inner.empty_node_pop() {
            self.inner.header_mut(head).set_zero();
            MartPointer::new(head, T::TYPE_ID)
        } else {
            MartPointer::new(self.inner.create_node(), T::TYPE_ID)
        }
    }

    fn make_node_with_edges<E>(&mut self, edges: E) -> MartPointer
    where
        E: AsRef<[MartEdge]>,
    {
        let header = edges.as_ref().len();
        assert!(
            header <= T::BYTES,
            "MartArraySparse::make_node_with_edges: edges.len() must be less than K"
        );
        let ptr = self.make_node();
        let node_id = ptr.nid();
        *self.inner.header_mut(node_id) = header as u8;
        for (idx, MartEdge { label, ptr }) in edges.as_ref().iter().enumerate() {
            self.inner.set_label(node_id, idx, *label);
            self.inner.set_ptr(node_id, idx, ptr);
        }
        ptr
    }

    fn find_child(&self, ptr: MartPointer, label: u8) -> Option<MartPointer> {
        assert_eq!(ptr.ntype(), T::TYPE_ID);
        self.adaptive_search_simd(ptr.nid(), label)
    }

    fn update_srcptr(&mut self, mc: &MartCursor) {
        self.inner.update_src(mc);
    }
}

impl<T: IntoMartNode> MartArrayExtractable<T> for MartSparse<T>
where
    LaneCount<{ T::BYTES }>: SupportedLaneCount,
{
    fn append_ptr(
        &mut self,
        mc: &mut MartCursor,
        label: u8,
        nptr: &MartPointer,
    ) -> MartInsertFlags {
        assert_eq!(mc.ntype(), T::TYPE_ID);
        let node_id = mc.nptr.nid();
        let header = self.inner.header(node_id).into();
        assert!(
            T::BYTES > header,
            "MartArraySparse::<{:?}>::append_ptr: cannot have {} children",
            T::TYPE_ID,
            header + 1
        );
        // not searched
        self.inner.header_inc(node_id);
        self.inner.set_label(node_id, header, label);
        self.inner.set_ptr(node_id, header, nptr);
        mc.update(header, nptr);

        MartInsertFlags::MartInserted
    }

    fn extract_edges(&mut self, mc: &MartCursor) -> Vec<MartEdge> {
        assert_eq!(mc.ntype(), T::TYPE_ID);
        let node_id = mc.nptr.nid();
        let extract = self.inner.edge_iter(node_id).collect();
        self.inner.empty_node_push(node_id);
        extract
    }
}

impl<'a, T: IntoMartNode> MartLinearScan<'a> for MartSparse<T> {
    fn linear_scan(&'a self, ptr: MartNodeId) -> Vec<MartEdge> {
        Vec::from_iter((0..self.inner.header(ptr).into()).filter_map(move |idx| {
            match (self.inner.label(ptr, idx), self.inner.ptr(ptr, idx)) {
                (Some(label), Some(ptr)) => Some(MartEdge { label, ptr }),
                (None, Some(_)) => panic!("MartSparse::linear_scan: label is None"),
                (Some(_), None) => panic!("MartSparse::linear_scan: ptr is None"),
                _ => None,
            }
        }))
    }
}

impl<T: IntoMartNode> MartChildren for MartSparse<T> {
    fn children(&self, ptr: &MartPointer) -> impl Iterator<Item = MartEdge> {
        assert_eq!(ptr.ntype(), T::TYPE_ID);
        self.inner.children(ptr)
    }
}

impl<T: IntoMartNode> TrieCounts for MartSparse<T> {
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

impl<T: IntoMartNode> PopulationStatistics for MartSparse<T> {
    fn population_stats(&self) -> PopulationStats {
        self.inner.population_stats()
    }
}

impl<T: IntoMartNode> InNodeStatistics for MartSparse<T> {
    fn innode_stats(&self) -> InNodeStats {
        self.inner.innode_stats()
    }
}
