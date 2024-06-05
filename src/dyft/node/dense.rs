// https://github.com/kampersanda/dyft/blob/master/include/mart_array_dense.hpp

use super::common::MartCursor;
use super::common::MartEdge;
use super::common::MartInsertFlags;
use super::common::MartPointer;
use super::mart_vec::MartVec;
use super::node_types::IntoMartNode;
use super::offsets::MartDenseOffsets;
use super::traits::MartByteLabels;
use super::traits::MartOffsets;
use super::ByteLabels;
use super::EmptyNodes;
use super::InnerMartBruteForce;
use super::MartArray;
use super::MartArrayExtractable;
use super::MartBruteForce;
use super::MartChildren;
use super::MartFind;
use super::MartTrie;
use super::TrieCounts;
use crate::dyft::BitPositionsEntry;
use crate::dyft::HamTableEntry;
use crate::dyft::InNodeStatistics;
use crate::dyft::InNodeStats;
use crate::dyft::MartEmptyNodeStack;
use crate::dyft::MartNodeId;
use crate::dyft::PopulationStatistics;
use crate::dyft::PopulationStats;
use get_size::GetSize;
use num_traits::Zero;
use std::convert::AsRef;

#[derive(GetSize)]
pub struct MartDense<T> {
    inner: EmptyNodes<ByteLabels<MartVec<MartDenseOffsets<T>>>>,
}

impl<T: IntoMartNode + Default> Default for MartDense<T> {
    fn default() -> Self {
        assert!(T::BYTES >= 2 && T::BYTES < 255);
        Self {
            inner: EmptyNodes::new(ByteLabels::new(MartVec::new(MartDenseOffsets::<T>::BYTES))),
        }
    }
}

impl<T: IntoMartNode> MartDense<T> {
    pub const NIL_IDX: u8 = u8::MAX;

    pub fn internals(&self) -> &MartVec<MartDenseOffsets<T>> {
        &self.inner.inner.inner
    }
}

impl<T: IntoMartNode> MartArray<T> for MartDense<T> {
    fn insert_ptr(
        &mut self,
        mc: &mut MartCursor,
        label: u8,
        new_ptr: &MartPointer,
    ) -> MartInsertFlags {
        assert_eq!(mc.ntype(), T::TYPE_ID);
        let node_id = mc.nptr().nid();
        let header: usize = self.inner.header(node_id).into();
        assert!(
            header <= T::BYTES,
            "MartArrayDense::insert_ptr: header must be less than K, header={} & K={}",
            header,
            T::BYTES
        );
        if let Some(offset) = self.inner.ptr_offset(node_id, label) {
            let ptr = self
                .inner
                .ptr(node_id, offset)
                .expect("MartArrayDense::insert_ptr: label exists but ptr is None");
            mc.update(offset, &ptr);
            MartInsertFlags::MartFound
        } else if header < T::BYTES {
            self.inner.header_inc(node_id);
            self.inner.set_label(node_id, label.into(), header as u8);
            self.inner.set_ptr(node_id, header, new_ptr);
            mc.update(header, new_ptr);
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
            "MartArrayDense::make_node_with_edges: edges.len() must be less than K"
        );
        let ptr = self.make_node();
        let node_id = ptr.nid();
        *self.inner.header_mut(node_id) = header as u8;
        for (idx, MartEdge { label, ptr }) in edges.as_ref().iter().enumerate() {
            let offset = usize::from(*label);
            let label = idx
                .try_into()
                .expect("overflow in MartDense::make_node_with_edges");
            self.inner.set_label(node_id, offset, label);
            self.inner.set_ptr(node_id, idx, ptr);
        }
        ptr
    }

    fn find_child(&self, nptr: MartPointer, label: u8) -> Option<MartPointer> {
        assert_eq!(nptr.ntype(), T::TYPE_ID);
        let node_id = nptr.nid();
        self.inner
            .ptr_offset(node_id, label)
            .map(|offset| self.inner.ptr(node_id, offset))
            .flatten()
    }

    fn update_srcptr(&mut self, mc: &MartCursor) {
        self.inner.update_src(mc);
    }
}

impl<T: IntoMartNode> MartArrayExtractable<T> for MartDense<T> {
    fn append_ptr(
        &mut self,
        mc: &mut MartCursor,
        label: u8,
        nptr: &MartPointer,
    ) -> MartInsertFlags {
        assert_eq!(mc.ntype(), T::TYPE_ID);
        let node_id = mc.nptr.nid();
        let header = self.inner.header(node_id);

        assert!(
            T::BYTES > usize::from(header),
            "MartArrayDense::<{:?}>::append_ptr: cannot have {} children",
            T::TYPE_ID,
            header + 1
        );

        assert_eq!(
            self.inner.label(node_id, label.into()),
            Some(Self::NIL_IDX),
            "MartArrayDense::<{:?}>::append_ptr: node {}'s label at offset {:?} is {:?}, should have been None\nnode[{node_id}]: {:?}",
            T::TYPE_ID,
            node_id,
            label,
            self.inner.label(node_id, label.into()),
            self.internals().node(node_id)
        );

        // not searched
        self.inner.header_inc(node_id);
        self.inner.set_label(node_id, label.into(), header);
        self.inner.set_ptr(node_id, header.into(), nptr);
        mc.update(header.into(), nptr);

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
impl<T: IntoMartNode> MartBruteForce for MartDense<T> {
    fn brute_force(
        &self,
        ptr: MartNodeId,
        radius: u8,
        lookup: &HamTableEntry,
        hamming_distance: &HamTableEntry,
        byte_positions: &BitPositionsEntry,
    ) -> Vec<MartEdge> {
        InnerMartBruteForce::new(self).brute_force(
            ptr,
            radius,
            lookup,
            hamming_distance,
            byte_positions,
        )
    }
}

impl<T: IntoMartNode> MartFind for &MartDense<T> {
    fn find(&self, ptr: MartNodeId, label: u8) -> Option<MartPointer> {
        self.find_child(MartPointer::new(ptr, T::TYPE_ID), label)
    }
}

impl<T: IntoMartNode> MartChildren for MartDense<T> {
    fn children(&self, ptr: &MartPointer) -> impl Iterator<Item = MartEdge> {
        assert_eq!(ptr.ntype(), T::TYPE_ID);
        self.inner.children(ptr)
    }
}

impl<T: IntoMartNode> TrieCounts for MartDense<T> {
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

impl<T: IntoMartNode> PopulationStatistics for MartDense<T> {
    fn population_stats(&self) -> PopulationStats {
        self.inner.population_stats()
    }
}

impl<T: IntoMartNode> InNodeStatistics for MartDense<T> {
    fn innode_stats(&self) -> InNodeStats {
        self.inner.innode_stats()
    }
}
