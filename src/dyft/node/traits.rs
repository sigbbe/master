use super::IntoMartNode;
use super::MartCursor;
use super::MartEdge;
use super::MartInsertFlags;
use super::MartNodeTypes;
use super::MartPointer;
use super::RawMartPointer;
use super::MART_PTR_SIZE;
use crate::dyft::BitPositionsEntry;
use crate::dyft::HamTableEntry;
use crate::dyft::MartByteLabel;
use crate::dyft::MartIndex;
use crate::dyft::MartNodeId;
use crate::dyft::MartPointerOffset;

/// This trait defines the offsets of the pointers in the node
/// and the size of the node in bytes. It is used to calculate
/// the offset of the pointers and labels in the node
pub trait MartOffsets {
    const K: usize;

    // offset of the pointers in the node in bytes
    const PTRS_OFFSET: usize;
    // size of the node in bytes
    const BYTES: usize;

    fn header_index(&self, node_id: MartNodeId) -> usize {
        Self::BYTES
            * <MartNodeId as TryInto<usize>>::try_into(node_id).expect("node_id is too large")
    }

    fn ptr_index(&self, node_id: MartNodeId, child: usize) -> usize {
        Self::PTRS_OFFSET + self.header_index(node_id) + child * MART_PTR_SIZE
    }
}

/// this trait is used to implement the basic operations of a trie
/// all operations defined here are common to MartSparse, MartDense, and MartFull
pub trait MartTrie: MartOffsets {
    // create a new node
    fn create_node(&mut self) -> MartNodeId;

    fn next_node_id(&self) -> MartNodeId;

    fn update_src(&mut self, mc: &MartCursor);

    // get the header of a node
    fn header(&self, node_id: MartNodeId) -> MartByteLabel;

    fn header_inc(&mut self, node_id: MartNodeId) {
        let header = self.header_mut(node_id);
        *header += 1;
    }

    // get a mutable reference to the header of a node
    fn header_mut(&mut self, node_id: MartNodeId) -> &mut u8;

    fn ptr(&self, node_id: MartNodeId, idx: usize) -> Option<MartPointer>;

    fn set_ptr(&mut self, node_id: MartNodeId, idx: usize, ptr: &MartPointer) {
        if let Some(raw) = self.ptr_slice_mut(node_id, idx) {
            let value: RawMartPointer = ptr.into();
            raw.copy_from_slice(&value[..])
        }
    }
    fn ptr_iter<'a, I>(
        &'a self,
        node_id: MartNodeId,
        label_idxs: I,
    ) -> impl Iterator<Item = MartPointer> + 'a
    where
        I: IntoIterator<Item = usize> + 'a;

    // the implementor should implement this method
    // consumers of the implementions should call set_ptr instead
    fn ptr_slice_mut(&mut self, node_id: MartNodeId, idx: usize) -> Option<&mut RawMartPointer>;
}

pub trait MartArray<T: IntoMartNode> {
    fn insert_ptr(&mut self, mc: &mut MartCursor, label: u8, ptr: &MartPointer) -> MartInsertFlags;

    fn make_node(&mut self) -> MartPointer;

    fn make_node_with_edges<E>(&mut self, edges: E) -> MartPointer
    where
        E: AsRef<[MartEdge]>;

    fn update_srcptr(&mut self, mc: &MartCursor);

    fn find_child(&self, ptr: MartPointer, label: u8) -> Option<MartPointer>;
}

pub trait MartArrayExtractable<T: IntoMartNode>: MartArray<T> {
    fn append_ptr(&mut self, mc: &mut MartCursor, c: u8, nptr: &MartPointer) -> MartInsertFlags;

    fn extract_edges(&mut self, mc: &MartCursor) -> Vec<MartEdge>;
}

pub trait MartNodeSearcher<'a> {
    fn find(&self, label: u8) -> MartPointer;

    fn scan(&self) -> impl Iterator<Item = MartEdge>;
}

/// This trait is used for Mart nodes that have a byte label, i.e., MartSparse and MartDense
/// For MartSparse, the byte labels are stored in the the m_nodes slice, and the given byte label
/// is associated with the pointer at the same index in the m_ptrs slice.
/// For MartDense, it's the opposite, the indices are the byte labels, and the value of the byte
/// label is the offsets into the pointer associated with the label.
pub trait MartByteLabels: MartOffsets + MartTrie {
    // returns an iterator over the labels of a node
    fn label_iter(&self, node_id: MartNodeId) -> impl Iterator<Item = MartByteLabel>;

    // returns an iterator over the node's edges
    fn edge_iter(&self, node_id: MartNodeId) -> impl Iterator<Item = MartEdge> {
        self.label_iter(node_id)
            .zip(self.ptr_offset_iter(node_id))
            .filter_map(move |(label, ptr)| match self.ptr(node_id, ptr) {
                Some(ptr) => Some(MartEdge { label, ptr }),
                None => None,
            })
    }

    // returns an iterator over the offsets of the pointers in the node
    fn ptr_offset_iter(&self, node_id: MartNodeId) -> impl Iterator<Item = MartPointerOffset>;

    // takes a node_id and a label, and returns the offset of the associated pointer in the node
    fn ptr_offset(&self, node_id: MartNodeId, idx: MartByteLabel) -> Option<MartPointerOffset>;
    // fn label_offset(&self, node_id: MartNodeId, idx: MartByteLabel) -> Option<MartPointerOffset>;

    // returns the optional label at labels_offset(node_id) + idx
    fn label(&self, node_id: MartNodeId, idx: MartPointerOffset) -> Option<MartByteLabel>;

    fn label_none_if_unset(
        &self,
        node_id: MartNodeId,
        idx: MartPointerOffset,
    ) -> Option<MartByteLabel> {
        self.label(node_id, idx).filter(|&label| label != u8::MAX)
    }

    // sets the label at labels_offset(node_id) + idx to value
    fn set_label(&mut self, node_id: MartNodeId, idx: MartPointerOffset, value: MartByteLabel);
}

// This trait is used for Maintining a linked list of empty nodes.
// It is used in MartSparse and MartDense
pub trait MartEmptyNodeStack<T> {
    fn empty_node_push(&mut self, value: T);

    fn empty_node_pop(&mut self) -> Option<T>;
}

pub trait MartLinearScan<'a> {
    fn linear_scan(&'a self, ptr: MartNodeId) -> Vec<MartEdge>;
}

pub trait MartBruteForce {
    fn brute_force(
        &self,
        ptr: MartNodeId,
        radius: u8,
        lookup: &HamTableEntry,
        hamming_distance: &HamTableEntry,
        byte_positions: &BitPositionsEntry,
    ) -> Vec<MartEdge>;
}

pub trait MartFind {
    fn find(&self, ptr: MartNodeId, label: u8) -> Option<MartPointer>;
}

pub trait TrieCounts {
    fn num_nodes(&self) -> usize;

    fn num_empty(&self) -> usize;

    fn num_edges(&self) -> usize;
}

/// This trait is used to access the children of a given node in the trie
/// It is used in all the Mart node implementations.
pub trait MartChildren {
    fn children(&self, ptr: &MartPointer) -> impl Iterator<Item = MartEdge>;
}

impl<'a> MartChildren for MartIndex<'a> {
    fn children(&self, ptr: &MartPointer) -> impl Iterator<Item = MartEdge> {
        match ptr.ntype() {
            MartNodeTypes::Mart2Node => self.m_array_2.children(ptr).collect::<Vec<_>>(),
            MartNodeTypes::Mart4Node => self.m_array_4.children(ptr).collect::<Vec<_>>(),
            MartNodeTypes::Mart8Node => self.m_array_8.children(ptr).collect::<Vec<_>>(),
            MartNodeTypes::Mart16Node => self.m_array_16.children(ptr).collect::<Vec<_>>(),
            MartNodeTypes::Mart32Node => self.m_array_32.children(ptr).collect::<Vec<_>>(),
            MartNodeTypes::Mart64Node => self.m_array_64.children(ptr).collect::<Vec<_>>(),
            MartNodeTypes::Mart128Node => self.m_array_128.children(ptr).collect::<Vec<_>>(),
            MartNodeTypes::Mart256Node => self.m_array_256.children(ptr).collect::<Vec<_>>(),
            _ => unreachable!(),
        }
        .into_iter()
    }
}

impl<T: MartTrie + MartByteLabels> MartChildren for T {
    fn children(&self, ptr: &MartPointer) -> impl Iterator<Item = MartEdge> {
        let node_id = ptr.nid();
        let labels = self.label_iter(node_id);
        let ptr_idxs = self.ptr_offset_iter(node_id);
        self.ptr_iter(node_id, ptr_idxs)
            .zip(labels)
            .map(|(nptr, label)| MartEdge { label, ptr: nptr })
    }
}

pub struct InnerMartBruteForce<F> {
    inner: F,
}

impl<F> InnerMartBruteForce<F>
where
    F: MartFind,
{
    pub(crate) fn new(find: F) -> Self {
        Self { inner: find }
    }
}

impl<T: MartFind> MartBruteForce for InnerMartBruteForce<T> {
    fn brute_force(
        &self,
        ptr: MartNodeId,
        radius: u8,
        lookup_table: &HamTableEntry,
        hamming_distance_table: &HamTableEntry,
        bit_positions_table: &BitPositionsEntry,
    ) -> Vec<MartEdge> {
        Vec::from_iter((0..radius + 1).enumerate().flat_map(|(r, k)| {
            (bit_positions_table[r]..bit_positions_table[r + 1]).filter_map(move |bpos| {
                assert_eq!(k, hamming_distance_table[usize::from(lookup_table[bpos])]);
                let label = lookup_table[bpos];
                match k == hamming_distance_table[usize::from(label)] {
                    true => self
                        .inner
                        .find(ptr, label)
                        .map(|ptr| MartEdge { label, ptr }),
                    false => None,
                }
            })
        }))
    }
}
