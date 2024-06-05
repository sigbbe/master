// https://github.com/kampersanda/dyft/blob/master/include/sparse_table.hpp
#![allow(dead_code)]
use std::fmt::Debug;
use get_size::GetSize;

#[doc = "../../../doc/sparse_table.md"]
use super::sparse_group::SparseGroup;

#[derive(Debug, GetSize)]
pub struct SparseTable {
    m_groups: Vec<SparseGroup>,
    m_size: u32,
}

impl Default for SparseTable {
    fn default() -> Self {
        Self {
            m_groups: Vec::with_capacity(128),
            m_size: u32::default(),
        }
    }
}

impl SparseTable {
    const fn group_pos(idx: u32) -> usize {
        (idx / SparseGroup::SIZE) as usize
    }
    const fn group_mod(idx: u32) -> u32 {
        idx % SparseGroup::SIZE
    }
    pub fn clear(&mut self) {
        self.m_groups.clear();
        self.m_size = 0;
    }
    pub fn access(&self, idx: u32) -> Option<&[u32]> {
        assert!(
            idx < self.m_size,
            "SparseTable::access: idx out of bounds: {} >= {}",
            idx,
            self.m_size
        );
        self.m_groups[Self::group_pos(idx)].access(Self::group_mod(idx))
    }

    pub fn push(&mut self) {
        if self.m_size / SparseGroup::SIZE == self.m_groups.len() as u32 {
            self.m_groups.push(SparseGroup::default());
        }
        self.m_size += 1;
    }
    pub fn extend<T>(&mut self, datavec: T)
    where
        T: AsRef<[u32]>,
    {
        if self.m_size / SparseGroup::SIZE == self.m_groups.len() as u32 {
            self.m_groups.push(SparseGroup::default());
        }
        self.m_groups[Self::group_pos(self.m_size)].extend(Self::group_mod(self.m_size), datavec);
        self.m_size += 1;
    }
    pub fn insert(&mut self, idx: u32, data: u32) {
        assert!(
            idx < self.m_size,
            "SparseTable::insert: idx out of bounds: {} >= {}",
            idx,
            self.m_size
        );
        self.m_groups[Self::group_pos(idx)].insert(Self::group_mod(idx), data)
    }
    pub fn insert_iter<T>(&mut self, idx: u32, datavec: T)
    where
        T: AsRef<[u32]>,
    {
        assert!(
            idx < self.m_size,
            "SparseTable::insert_iter: idx out of bounds: {} >= {}",
            idx,
            self.m_size
        );
        self.m_groups[Self::group_pos(idx)].extend(Self::group_mod(idx), datavec)
    }
    pub fn extract(&mut self, idx: u32) -> Option<Vec<u32>> {
        assert!(
            idx < self.m_size,
            "SparseTable::extract: idx out of bounds: {} >= {}",
            idx,
            self.m_size
        );
        self.m_groups[Self::group_pos(idx)].extract(Self::group_mod(idx))
    }
    pub fn size(&self) -> u32 {
        self.m_size
    }
    pub fn group_size(&self, idx: u32) -> u32 {
        assert!(
            idx < self.m_size,
            "SparseTable::group_size: idx out of bounds: {} >= {}",
            idx,
            self.m_size
        );
        self.m_groups[Self::group_pos(idx)].size(Self::group_mod(idx))
    }

    pub fn print_group(&self, idx: u32) {
        assert!(
            idx < self.m_size,
            "SparseTable::print_group: idx out of bounds: {} >= {}",
            idx,
            self.m_size
        );
        self.m_groups[Self::group_pos(idx)].print_group()
    }
}
