use super::structure::MartIndex;
use crate::dyft::DyFTStatistics;
use crate::dyft::DyFTStats;
use crate::dyft::InNodeStatistics;
use crate::dyft::InNodeStats;

impl<'a> DyFTStatistics for MartIndex<'a> {
    fn stats(&self) -> DyFTStats {
        DyFTStats {
            size: self.size(),
            leaves: self.leaves(),
            split_count: self.split_count(),
            depth: <Self as DyFTStatistics>::depth(&self),
            innode_stats: self.innode_stats().collect(),
            partial_verification_count: 0,
            full_verification_count: 0,
            filtered_verification_count: 0,
        }
    }

    fn size(&self) -> usize {
        self.m_ids.try_into().unwrap()
    }

    fn leaves(&self) -> usize {
        self.m_postings_list.size().try_into().unwrap()
    }

    fn depth(&self) -> usize {
        self.m_max_depth
    }

    fn split_count(&self) -> usize {
        self.m_split_count
    }

    fn innode_stats(&self) -> impl Iterator<Item = InNodeStats> {
        [
            self.m_array_2.innode_stats(),
            self.m_array_4.innode_stats(),
            self.m_array_8.innode_stats(),
            self.m_array_16.innode_stats(),
            self.m_array_32.innode_stats(),
            self.m_array_64.innode_stats(),
            self.m_array_128.innode_stats(),
            self.m_array_256.innode_stats(),
        ]
        .into_iter()
    }
}
