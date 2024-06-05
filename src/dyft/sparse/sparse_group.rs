use get_size::GetSize;

// https://github.com/kampersanda/dyft/blob/master/include/sparse_group.hpp
#[derive(Debug, GetSize)]
pub struct SparseGroup {
    m_bitmap: u64,
    m_group: Vec<u32>,
}

impl Default for SparseGroup {
    fn default() -> Self {
        Self {
            m_bitmap: 0,
            m_group: Vec::with_capacity(Self::SIZE as usize),
        }
    }
}

impl SparseGroup {
    pub const SIZE: u32 = 64;

    #[inline]
    const fn bitmask(idx: u32) -> u64 {
        (1u64 << idx as u64) - 1u64
    }

    pub fn access(&self, idx: u32) -> Option<&[u32]> {
        self.check_idx(idx);
        if self.m_bitmap & (1 << idx) == 0 {
            None
        } else {
            let bitmask = Self::bitmask(idx);
            let howmany = (self.m_bitmap & bitmask).count_ones() as usize;
            let totones = (self.m_bitmap).count_ones() as usize;
            let size = self.m_group[howmany + 1] - self.m_group[howmany];
            let ptr = &self.m_group[totones + 1 + self.m_group[howmany] as usize..];
            Some(&ptr[..size as usize])
        }
    }

    pub fn insert(&mut self, idx: u32, data: u32) {
        // id=leaf position for data
        // data=node id for inserted vcode
        self.check_idx(idx);
        if self.m_bitmap == 0 {
            self.m_bitmap = 1u64 << idx;
            self.m_group.clear();
            self.m_group.extend([0, 1, data].iter());
        } else {
            let bitmask = Self::bitmask(idx);
            let howmany = (self.m_bitmap & bitmask).count_ones() as usize;
            if self.m_bitmap & (1 << idx) == 0 {
                self.m_group.insert(howmany, self.m_group[howmany as usize]);
                self.m_bitmap |= 1 << idx;
            }

            let totones = self.m_bitmap.count_ones() as usize;
            let insertion_index = totones + 1 + self.m_group[howmany + 1] as usize;
            let len = self.m_group.len();
            assert!(insertion_index <= len, "SparseGroup::insert: out of bounds: {} > {}", insertion_index, len);
            self.m_group
                .insert(totones + 1 + self.m_group[howmany + 1] as usize, data);

            for i in howmany + 1..totones + 1 {
                self.m_group[i] += 1;
            }
        }
    }

    pub fn extend<T>(&mut self, idx: u32, datavec: T)
    where
        T: AsRef<[u32]>,
    {
        self.check_idx(idx);
        let n = datavec.as_ref().len() as u32;
        if self.m_bitmap == 0 {
            self.m_bitmap = 1u64 << idx as u64;
            self.m_group.clear();
            self.m_group
                .extend([0, n].iter().chain(datavec.as_ref().iter()));
        } else {
            let bitmask = Self::bitmask(idx);
            let howmany = (self.m_bitmap & bitmask).count_ones() as usize;
            
            // if the result is 0, it means the bit at position idx in the bitmap is not set
            let bitmask = bitmask + 1;
            if (self.m_bitmap & bitmask) == 0u64 {
                self.m_group.insert(howmany, self.m_group[howmany]);
                self.m_bitmap |= bitmask;
            }

            let start = (self.m_bitmap).count_ones() as usize;
            let pos = start + 1 + self.m_group[howmany + 1] as usize;
            self.m_group
                .splice(pos..pos, datavec.as_ref().iter().copied());

            for i in howmany + 1..start + 1 {
                self.m_group[i] += n as u32;
            }
        }
    }

    pub fn extract(&mut self, idx: u32) -> Option<Vec<u32>> {
        self.check_idx(idx);
        if self.m_bitmap & (1 << idx) == 0 {
            None
        } else {
            let bitmask = (1 << idx) - 1;
            let howmany = (self.m_bitmap & bitmask).count_ones() as usize;
            let totones = (self.m_bitmap).count_ones() as usize;
            let size = self.m_group[howmany + 1] - self.m_group[howmany];
            let pos = totones + 1 + self.m_group[howmany] as usize;
            let res = self.m_group[pos..pos + size as usize].to_vec();
            for i in howmany + 2..totones + 1 {
                self.m_group[i] = self.m_group[i] - size;
            }
            self.m_group.drain(pos..pos + size as usize);
            self.m_group.remove(howmany + 1);
            self.m_bitmap &= !(1 << idx);
            Some(res)
        }
    }

    pub fn size(&self, idx: u32) -> u32 {
        self.check_idx(idx);
        if self.m_bitmap & (1 << idx) == 0 {
            0
        } else {
            let bitmask = Self::bitmask(idx);
            let howmany = (self.m_bitmap & bitmask).count_ones() as usize;
            self.m_group[howmany + 1] - self.m_group[howmany]
        }
    }

    pub fn print_group(&self) {
        println!("{:?}", self.m_group);
    }

    fn check_idx(&self, idx: u32) {
        assert!(
            idx < Self::SIZE,
            "idx must be less than Self::SIZE: {} > {}",
            idx,
            Self::SIZE
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_sparse_group_insert() {
        let mut sparse_group = SparseGroup::default();
        sparse_group.insert(0, 0);
        sparse_group.extract(0);
        sparse_group.extend(0, &[0]);
        sparse_group.insert(1, 1);
        sparse_group.extract(1);
        sparse_group.extend(1, &[1]);
        sparse_group.insert(2, 2);
        sparse_group.extract(2);
        sparse_group.extend(2, &[2]);
        sparse_group.insert(3, 3);
        sparse_group.extract(3);
        sparse_group.extend(3, &[3]);
        sparse_group.insert(4, 4);
        sparse_group.extract(4);
        sparse_group.extend(4, &[4]);
        sparse_group.insert(5, 5);
        sparse_group.extract(5);
        sparse_group.extend(5, &[5]);
        sparse_group.insert(6, 6);
        sparse_group.extract(6);
        sparse_group.extend(6, &[6]);
        sparse_group.insert(7, 7);
        sparse_group.extract(7);
        sparse_group.extend(7, &[7]);
        sparse_group.insert(6, 8);
        sparse_group.extract(6);
        sparse_group.extend(6, &[8]);
        sparse_group.extend(8, &[6]);
        sparse_group.insert(9, 9);
        sparse_group.extract(9);
        sparse_group.extend(9, &[9]);
        sparse_group.insert(10, 10);
        sparse_group.extract(10);
        sparse_group.extend(10, &[10]);
        sparse_group.insert(11, 11);
        sparse_group.extract(11);
        sparse_group.extend(11, &[11]);
        sparse_group.insert(12, 12);
        sparse_group.extract(12);
        sparse_group.extend(12, &[12]);
        sparse_group.insert(13, 13);
        sparse_group.extract(13);
        sparse_group.extend(13, &[13]);
        sparse_group.insert(14, 14);
        sparse_group.extract(14);
        sparse_group.extend(14, &[14]);
        sparse_group.insert(15, 15);
        sparse_group.extract(15);
        sparse_group.extend(15, &[15]);
        sparse_group.insert(16, 16);
        sparse_group.extract(16);
        sparse_group.extend(16, &[16]);
        sparse_group.insert(17, 17);
        sparse_group.extract(17);
        sparse_group.extend(17, &[17]);
        sparse_group.insert(18, 18);
        sparse_group.extract(18);
        sparse_group.extend(18, &[18]);
        sparse_group.insert(19, 19);
        sparse_group.extract(19);
        sparse_group.extend(19, &[19]);
        let truth = &[
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 0, 1, 2, 3,
            4, 5, 8, 7, 6, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19,
        ];
        let len = sparse_group.m_group.len();
        assert_eq!(truth.len(), len);
        assert_eq!(truth, &sparse_group.m_group.as_ref());
    }
}
