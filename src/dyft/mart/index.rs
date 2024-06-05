//  https://github.com/kampersanda/dyft/blob/master/include/mart_index.hpp
use self::byte_pack::BytePack;
use super::state::StateType;
use super::structure::MartIndex;
use crate::dyft::*;
use num_traits::ToPrimitive;
use std::collections::HashMap;
use std::collections::VecDeque;

impl<'a, T> DyFT<T> for MartIndex<'a>
where
    T: VCodeTools,
{
    fn build(&mut self, database: &VCodeArray<T>, n: usize) {
        let start = self.m_ids.to_usize().unwrap();
        let end = n.min(database.size());
        (start..end).for_each(move |id| self.append(database.access(id), &database))
    }

    fn append(&mut self, vcode: &[T], database: &VCodeArray<T>) {
        // create a new id
        let new_id = self.inc_id();

        // create a mart cursor from the root pointer
        let mut mc = MartCursor::from_next(&self.m_rootptr);

        // println!("{:?}", Vec::from_iter(self.byte_pack_iter(vcode)));

        // append the new vcode by iterating through the chunks
        let byte_pos = self.iter_append(
            &vcode,
            &mut mc,
            MartPointer::leaf(self.m_postings_list.size()),
        );

        // position of the leaf node
        let leaf_pos = mc.nptr().nid();
        assert_eq!(mc.nptr().ntype(), MartNodeTypes::MartLeafNode);

        // insert the leaf position into the postings list
        self.m_postings_list.insert(leaf_pos, new_id);

        // depth of node
        let depth = self.depth::<T>(byte_pos);
        self.m_max_depth = self.m_max_depth.max(depth);

        // perform split if the bit position is not at the end of the chunk
        if byte_pos != self.m_end {
            self.perform_split(database, &mut mc, leaf_pos, byte_pos, depth);
        }
    }

    fn trie_search(&self, vcode: &[T]) -> impl Iterator<Item = u32> {
        // initialize a return iterator
        let mut ret = VecDeque::<u32>::with_capacity(1024);

        // initialize state stack
        let mut m_states = vec![StateType::new(
            MartPointer::new(self.m_rootptr.nid(), self.m_rootptr.ntype()),
            0,
            0,
        )];

        // pack the vcode into the chunks
        let ints_per_chunk: u8 = self.ints_per_chunk::<T>().try_into().unwrap();
        let labels: Vec<MartByteLabel> = self.byte_pack_iter(vcode).collect();

        // perform bfs search on the trie
        while let Some(mut state) = m_states.pop() {
            assert!(
                self.m_errors >= state.dist(),
                "Error: radius must be less than state.dist, state.dist={} radius={}",
                state.dist(),
                self.m_radius
            );
            let label = labels[state.pos()];
            let hamming_distance = HamTables::hamming_distance(self.m_bits, label);

            // if the hamming distance is less than the errors, perform trie search
            // with number of allowed error=state.dist() - self.m_errors, otherwise
            // perform exact search
            if state.dist() < self.m_errors {
                for MartEdge {
                    label,
                    ptr: MartPointer { nid, ntype },
                } in self.perform_scan(&state, *hamming_distance, label, ints_per_chunk)
                {
                    match ntype {
                        MartNodeTypes::MartLeafNode => {
                            if let Some(ids) = self.m_postings_list.access(nid) {
                                ret.extend(ids.into_iter());
                            }
                        }
                        _ => {
                            m_states.push(state.next(
                                nid,
                                ntype,
                                hamming_distance[usize::from(label)],
                            ));
                        }
                    }
                }
            } else {
                self.perform_exact_search(vcode, &mut state)
                    .map(|ids| ret.extend(ids));
            }
        }
        ret.into_iter()
    }
}

impl<'a> MartIndex<'a> {
    pub fn root(&self) -> &MartPointer {
        &self.m_rootptr
    }

    pub fn max_depth(&self) -> usize {
        self.m_max_depth
    }

    pub fn posting_list(&self) -> &SparseTable {
        &self.m_postings_list
    }

    pub fn depth<T>(&self, byte_pos: usize) -> usize
    where
        T: VCodeTools,
    {
        self.ints_per_chunk::<T>() * (byte_pos - self.m_begin - 1)
    }

    fn ints_per_chunk<T>(&self) -> usize
    where
        T: VCodeTools,
    {
        BytePack::<T>::ints_per_chunk(self.m_bits)
    }

    fn byte_pack_iter<'b, T>(&self, vcode: &'b [T]) -> impl Iterator<Item = MartByteLabel> + 'b
    where
        T: VCodeTools,
    {
        BytePack::<T>::pack_iter(
            vcode,
            self.byte_pack_range(),
            self.m_bits,
            self.ints_per_chunk::<T>(),
        )
    }

    fn byte_pack_bit_pos<T>(&self, vcode: &[T], bpos: usize) -> MartByteLabel
    where
        T: VCodeTools,
    {
        BytePack::<T>::pack(vcode, self.ints_per_chunk::<T>(), self.m_bits, bpos)
    }

    fn byte_pack_range(&self) -> std::ops::Range<usize> {
        self.m_begin..self.m_end
    }
}

impl<'a> MartIndex<'a> {
    fn inc_id(&mut self) -> MartNodeId {
        let id = self.m_ids;
        self.m_ids += 1;
        id
    }

    fn iter_append<T>(&mut self, vcode: &[T], mc: &mut MartCursor, ptr: MartPointer) -> usize
    where
        T: VCodeTools,
    {
        self.byte_pack_iter(vcode)
            .position(|label| match self.perform_insert_ptr(mc, label, &ptr) {
                MartInsertFlags::MartInserted => {
                    assert_eq!(mc.nptr().nid(), ptr.nid());
                    assert_eq!(mc.nptr().ntype(), MartNodeTypes::MartLeafNode);
                    self.m_postings_list.push();
                    true
                }
                _ if mc.is_leaf() => return true,
                _ => false,
            })
            .map(|byte_pos| self.m_begin + byte_pos + 1)
            .unwrap_or(self.m_end)
    }

    fn extract_buckets<T>(
        &mut self,
        m_database: &VCodeArray<T>,
        leaf_pos: u32,
        bit_pos: usize,
    ) -> Option<impl Iterator<Item = (u8, Vec<u32>)>>
    where
        T: VCodeTools,
    {
        if let Some(ids) = self.m_postings_list.extract(leaf_pos) {
            const EMPTY_VEC: Vec<u32> = Vec::new();
            let mut buckets = [EMPTY_VEC; MartNode256::BYTES];
            for id in ids {
                let vcode = m_database.access(id as usize);
                let key: usize = self.byte_pack_bit_pos(vcode, bit_pos).into();
                buckets[key].push(id);
            }
            Some(
                (0u8..)
                    .zip(buckets.into_iter())
                    .filter(|(_, idxs)| !idxs.is_empty()),
            )
        } else {
            None
        }
    }
    fn _extract_buckets<T>(
        &mut self,
        m_database: &VCodeArray<T>,
        leaf_pos: u32,
        byte_pos: usize,
    ) -> Option<impl Iterator<Item = (u8, Vec<u32>)>>
    where
        T: VCodeTools,
    {
        if let Some(ids) = self.m_postings_list.extract(leaf_pos) {
            let mut buckets = HashMap::new();
            for id in ids {
                let idx: usize = id.try_into().unwrap();
                let vcode = m_database.access(idx);
                let key = self.byte_pack_bit_pos(vcode, byte_pos);
                buckets.entry(key).or_insert_with(Vec::new).push(id);
            }
            Some(
                buckets
                    .into_iter()
                    .filter(|(_, idxs)| !idxs.is_empty())
                    .sorted_by(|&(a, _), (b, _)| a.cmp(b)),
            )
        } else {
            None
        }
    }

    fn split_node<T>(&mut self, m_database: &VCodeArray<T>, mc: &mut MartCursor, bpos: usize)
    where
        T: VCodeTools,
    {
        assert!(bpos < T::N_DIM);
        assert_eq!(mc.ntype(), MartNodeTypes::MartLeafNode);

        self.m_split_count += 1;
        let leaf_pos = mc.nptr().nid();

        assert!(self.m_postings_list.size() > leaf_pos);

        self.m_edges.clear();

        if let Some(idbufs) = self.extract_buckets(m_database, leaf_pos, bpos) {
            for (label, idxs) in idbufs {
                if self.m_edges.is_empty() {
                    let nptr = MartPointer::leaf(leaf_pos);
                    self.m_edges.push(MartEdge { label, ptr: nptr });
                    self.m_postings_list.insert_iter(leaf_pos, &idxs);
                } else {
                    let nptr = MartPointer::leaf(self.m_postings_list.size());
                    self.m_edges.push(MartEdge { label, ptr: nptr });
                    self.m_postings_list.extend(&idxs);
                }
            }
        }
        self.perform_make_node(mc);
        self.perform_update_srcptr(mc);
    }

    fn perform_split<T>(
        &mut self,
        m_database: &VCodeArray<T>,
        mc: &mut MartCursor,
        leaf_pos: u32,
        byte_pos: usize,
        depth: usize,
    ) where
        T: VCodeTools,
    {
        match self.m_splitthreshold {
            SplitThresholds::Threshold(split_threshold) => {
                if split_threshold <= self.m_postings_list.group_size(leaf_pos) {
                    self.split_node(m_database, mc, byte_pos);
                }
            }
            SplitThresholds::Thresholds(split_thresholds) => {
                if split_thresholds[depth] * self.m_in_weight
                    <= self.m_postings_list.group_size(leaf_pos) as f32
                {
                    self.split_node(m_database, mc, byte_pos);
                }
            }
        }
    }

    fn perform_make_node(&mut self, mc: &mut MartCursor) {
        let n_edges = self.m_edges.len();
        if n_edges < 2 {
            mc.nptr = self.m_array_2.make_node_with_edges(&self.m_edges);
        } else if n_edges < 4 {
            mc.nptr = self.m_array_4.make_node_with_edges(&self.m_edges);
        } else if n_edges < 8 {
            mc.nptr = self.m_array_8.make_node_with_edges(&self.m_edges);
        } else if n_edges < 16 {
            mc.nptr = self.m_array_16.make_node_with_edges(&self.m_edges);
        } else if n_edges < 32 {
            mc.nptr = self.m_array_32.make_node_with_edges(&self.m_edges);
        } else if n_edges < 64 {
            mc.nptr = self.m_array_64.make_node_with_edges(&self.m_edges);
        } else if n_edges < 128 {
            mc.nptr = self.m_array_128.make_node_with_edges(&self.m_edges);
        } else {
            mc.nptr = self.m_array_256.make_node_with_edges(&self.m_edges);
        }
    }

    fn perform_update_srcptr(&mut self, mc: &MartCursor) {
        match mc.ptype() {
            MartNodeTypes::Mart2Node => self.m_array_2.update_srcptr(mc),
            MartNodeTypes::Mart4Node => self.m_array_4.update_srcptr(mc),
            MartNodeTypes::Mart8Node => self.m_array_8.update_srcptr(mc),
            MartNodeTypes::Mart16Node => self.m_array_16.update_srcptr(mc),
            MartNodeTypes::Mart32Node => self.m_array_32.update_srcptr(mc),
            MartNodeTypes::Mart64Node => self.m_array_64.update_srcptr(mc),
            MartNodeTypes::Mart128Node => self.m_array_128.update_srcptr(mc),
            MartNodeTypes::Mart256Node => self.m_array_256.update_srcptr(mc),
            _ => panic!("Error: Invalid node type"),
        }
    }
    fn perform_insert_ptr(
        &mut self,
        mc: &mut MartCursor,
        label: u8,
        ptr: &MartPointer,
    ) -> MartInsertFlags {
        match mc.ntype() {
            MartNodeTypes::Mart2Node => match self.m_array_2.insert_ptr(mc, label, ptr) {
                MartInsertFlags::MartNeededToExpand => self.expand_2(mc, label, ptr),
                flag => flag,
            },
            MartNodeTypes::Mart4Node => match self.m_array_4.insert_ptr(mc, label, ptr) {
                MartInsertFlags::MartNeededToExpand => self.expand_4(mc, label, ptr),
                flag => flag,
            },
            MartNodeTypes::Mart8Node => match self.m_array_8.insert_ptr(mc, label, ptr) {
                MartInsertFlags::MartNeededToExpand => self.expand_8(mc, label, ptr),
                flag => flag,
            },
            MartNodeTypes::Mart16Node => match self.m_array_16.insert_ptr(mc, label, ptr) {
                MartInsertFlags::MartNeededToExpand => self.expand_16(mc, label, ptr),
                flag => flag,
            },
            MartNodeTypes::Mart32Node => match self.m_array_32.insert_ptr(mc, label, ptr) {
                MartInsertFlags::MartNeededToExpand => self.expand_32(mc, label, ptr),
                flag => flag,
            },
            MartNodeTypes::Mart64Node => match self.m_array_64.insert_ptr(mc, label, ptr) {
                MartInsertFlags::MartNeededToExpand => self.expand_64(mc, label, ptr),
                flag => flag,
            },
            MartNodeTypes::Mart128Node => match self.m_array_128.insert_ptr(mc, label, ptr) {
                MartInsertFlags::MartNeededToExpand => self.expand_128(mc, label, ptr),
                flag => flag,
            },
            MartNodeTypes::Mart256Node => self.m_array_256.insert_ptr(mc, label, ptr),
            _ => panic!("Error: Invalid node type: {:#?}", mc.nptr().ntype()),
        }
    }

    fn perform_inner_scan(
        &self,
        state: &StateType,
        hamming_distance: &HamTableEntry,
        radius: u8,
        label: MartByteLabel,
    ) -> Vec<MartEdge> {
        match state.ntype() {
            MartNodeTypes::Mart2Node => self.m_array_2.linear_scan(state.nid()),
            MartNodeTypes::Mart4Node => self.m_array_4.linear_scan(state.nid()),
            MartNodeTypes::Mart8Node => self.m_array_8.linear_scan(state.nid()),
            MartNodeTypes::Mart16Node => self.m_array_16.linear_scan(state.nid()),
            MartNodeTypes::Mart32Node => self.m_array_32.linear_scan(state.nid()),
            ntype => {
                let lookup = HamTables::lookup(self.m_bits, label);
                match ntype {
                    MartNodeTypes::Mart64Node => self.m_array_64.brute_force(
                        state.nid(),
                        radius,
                        lookup,
                        hamming_distance,
                        &self.m_bit_positions,
                    ),
                    MartNodeTypes::Mart128Node => self.m_array_128.brute_force(
                        state.nid(),
                        radius,
                        lookup,
                        hamming_distance,
                        &self.m_bit_positions,
                    ),
                    MartNodeTypes::Mart256Node => self.m_array_256.brute_force(
                        state.nid(),
                        radius,
                        lookup,
                        hamming_distance,
                        &self.m_bit_positions,
                    ),
                    _ => panic!("Error: Invalid node type: {:#?}", state.ntype()),
                }
            }
        }
    }

    fn perform_scan<'s>(
        &self,
        state: &StateType,
        hamming_distance: HamTableEntry,
        label: MartByteLabel,
        ints_per_chunk: MartByteLabel,
    ) -> impl Iterator<Item = MartEdge> + 's {
        let radius = u8::min(self.m_errors - state.dist(), ints_per_chunk);
        self.perform_inner_scan(state, &hamming_distance, radius, label)
            .into_iter()
            .filter(move |edge| hamming_distance[edge.label_idx()] <= radius)
    }

    fn perform_exact_search<T>(&self, vcode: &[T], state: &mut StateType) -> Option<Vec<MartNodeId>>
    where
        T: VCodeTools,
    {
        while let Some(child_ptr) =
            self.perform_find_child(*state.ptr(), self.byte_pack_bit_pos(vcode, state.pos()))
        {
            if child_ptr.is_leaf() {
                let leaf_pos = child_ptr.nid();
                if let Some(ids) = self.m_postings_list.access(leaf_pos) {
                    return Some(ids.to_vec());
                }
            }
            *state.ptr_mut() = child_ptr;
            *state.pos_mut() += 1;
        }
        None
    }

    fn perform_find_child(&self, ptr: MartPointer, label: u8) -> Option<MartPointer> {
        match ptr.ntype() {
            MartNodeTypes::Mart2Node => self.m_array_2.find_child(ptr, label),
            MartNodeTypes::Mart4Node => self.m_array_4.find_child(ptr, label),
            MartNodeTypes::Mart8Node => self.m_array_8.find_child(ptr, label),
            MartNodeTypes::Mart16Node => self.m_array_16.find_child(ptr, label),
            MartNodeTypes::Mart32Node => self.m_array_32.find_child(ptr, label),
            MartNodeTypes::Mart64Node => self.m_array_64.find_child(ptr, label),
            MartNodeTypes::Mart128Node => self.m_array_128.find_child(ptr, label),
            MartNodeTypes::Mart256Node => self.m_array_256.find_child(ptr, label),
            MartNodeTypes::MartLeafNode | MartNodeTypes::MartNilNode => {
                panic!("Error: Invalid node type")
            }
        }
    }

    pub fn perform_find_children(&self, ptr: &MartPointer) -> Vec<MartEdge> {
        match ptr.ntype() {
            MartNodeTypes::Mart2Node => self.m_array_2.children(ptr).collect(),
            MartNodeTypes::Mart4Node => self.m_array_4.children(ptr).collect(),
            MartNodeTypes::Mart8Node => self.m_array_8.children(ptr).collect(),
            MartNodeTypes::Mart16Node => self.m_array_16.children(ptr).collect(),
            MartNodeTypes::Mart32Node => self.m_array_32.children(ptr).collect(),
            MartNodeTypes::Mart64Node => self.m_array_64.children(ptr).collect(),
            MartNodeTypes::Mart128Node => self.m_array_128.children(ptr).collect(),
            MartNodeTypes::Mart256Node => self.m_array_256.children(ptr).collect(),
            MartNodeTypes::MartLeafNode | MartNodeTypes::MartNilNode => {
                panic!("Error: Invalid node type")
            }
        }
    }
}

macro_rules! define_expand_fn {
    ($name:ident, $from:ident, $to:ident, $insert:ident) => {
        impl<'a> MartIndex<'a> {
            fn $name(
                &mut self,
                mc: &mut MartCursor,
                label: u8,
                new_ptr: &MartPointer,
            ) -> MartInsertFlags {
                *mc.nptr_mut() = {
                    let m_edges = self.$from.extract_edges(mc);
                    self.$to.make_node_with_edges(&m_edges)
                };
                self.perform_update_srcptr(mc);
                self.$to.$insert(mc, label, new_ptr)
            }
        }
    };
}

define_expand_fn!(expand_2, m_array_2, m_array_4, append_ptr);
define_expand_fn!(expand_4, m_array_4, m_array_8, append_ptr);
define_expand_fn!(expand_8, m_array_8, m_array_16, append_ptr);
define_expand_fn!(expand_16, m_array_16, m_array_32, append_ptr);
define_expand_fn!(expand_32, m_array_32, m_array_64, append_ptr);
define_expand_fn!(expand_64, m_array_64, m_array_128, append_ptr);
define_expand_fn!(expand_128, m_array_128, m_array_256, insert_ptr);
