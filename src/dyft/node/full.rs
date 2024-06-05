// https://github.com/kampersanda/dyft/blob/master/include/mart_array_full.hpp

use super::common::MartCursor;
use super::common::MartEdge;
use super::common::MartInsertFlags;
use super::common::MartPointer;
use super::mart_vec::MartVec;
use super::node_types::IntoMartNode;
use super::offsets::MartFullOffsets;
use super::InnerMartBruteForce;
use super::MartArray;
use super::MartBruteForce;
use super::MartChildren;
use super::MartFind;
use super::MartTrie;
use super::TrieCounts;
use crate::dyft::BitPositionsEntry;
use crate::dyft::HamTableEntry;
use crate::dyft::InNodeStatistics;
use crate::dyft::InNodeStats;
use crate::dyft::MartNodeId;
use get_size::GetSize;

/// Node Full is a data structure for very large k and consists of pointer array Ptr of
/// length 256 such that Ptr[b] stores the child pointer with edge label b. The data structure
/// is identical to the array form.
#[derive(GetSize)]
pub struct MartFull<T> {
    inner: MartVec<MartFullOffsets<T>>,
}

impl<'a, T: IntoMartNode> Default for MartFull<T> {
    fn default() -> Self {
        Self {
            inner: MartVec::new(T::BYTES),
        }
    }
}

impl<'a, T: IntoMartNode> MartFull<T> {
    pub fn internals(&self) -> &MartVec<MartFullOffsets<T>> {
        &self.inner
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }
    pub fn header(&self, node_id: MartNodeId) -> u8 {
        self.inner.header(node_id)
    }
}

impl<'a, T: IntoMartNode> MartArray<T> for MartFull<T> {
    /// Current implementation of MartArrayFull does not include a single-byte header
    /// in the byte vector for storing the value of k. Therefore PTRS_OFFSET is set to 0.
    /// In the implementation we therefore skip adding PTRS_OFFSET to the index when
    /// accessing the byte vector.
    fn insert_ptr(
        &mut self,
        mc: &mut MartCursor,
        label: u8,
        new_ptr: &MartPointer,
    ) -> MartInsertFlags {
        assert_eq!(mc.ntype(), T::TYPE_ID);
        let node_id = mc.nptr.nid();
        let offset: usize = label.into();
        if let Some(mptr) = self.inner.ptr(node_id, offset) {
            // found
            mc.update(offset, &mptr);
            MartInsertFlags::MartFound
        } else {
            // not found
            self.inner.header_inc(node_id);
            self.inner.set_ptr(node_id, offset, new_ptr);
            mc.update(offset, new_ptr);
            MartInsertFlags::MartInserted
        }
    }

    fn make_node(&mut self) -> MartPointer {
        let node_id = self.inner.next_node_id();
        self.inner.create_node();
        MartPointer::new(node_id, T::TYPE_ID)
    }

    fn make_node_with_edges<E>(&mut self, edges: E) -> MartPointer
    where
        E: AsRef<[MartEdge]>,
    {
        let ptr = self.make_node();
        let node_id = ptr.nid();
        let header = edges.as_ref().len() as u8;
        *self.inner.header_mut(node_id) = header;

        for edge in edges.as_ref() {
            let offset: usize = edge.label.into();
            self.inner.set_ptr(node_id, offset, &edge.ptr);
        }

        ptr
    }

    fn find_child(&self, ptr: MartPointer, label: u8) -> Option<MartPointer> {
        assert_eq!(ptr.ntype(), T::TYPE_ID);
        self.inner.ptr(ptr.nid(), label.into())
    }
    fn update_srcptr(&mut self, mc: &MartCursor) {
        self.inner.update_src(mc);
    }
}

impl<T: IntoMartNode> MartBruteForce for MartFull<T> {
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
impl<T: IntoMartNode> MartFind for &MartFull<T> {
    fn find(&self, ptr: MartNodeId, label: u8) -> Option<MartPointer> {
        self.inner.ptr(ptr, label.into())
    }
}

impl<T: IntoMartNode> MartChildren for MartFull<T> {
    fn children(&self, ptr: &MartPointer) -> impl Iterator<Item = MartEdge> {
        assert_eq!(ptr.ntype(), T::TYPE_ID);
        self.inner
            .ptr_iter(ptr.nid(), 0..256)
            .enumerate()
            .map(|(i, nptr)| MartEdge {
                label: i.try_into().expect("overflow in MartFull::children"),
                ptr: nptr,
            })
    }
}

impl<T: IntoMartNode> TrieCounts for MartFull<T> {
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

impl<T: IntoMartNode> InNodeStatistics for MartFull<T> {
    fn innode_stats(&self) -> InNodeStats {
        self.inner.innode_stats()
    }
}
