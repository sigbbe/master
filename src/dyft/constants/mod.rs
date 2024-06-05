use get_size::GetSize;
use new_split_thresholds::NEW_SPLIT_THRS;

#[rustfmt::skip]
mod new_split_thresholds;
include!("ham_tables/hd_table.rs");
include!("ham_tables/lu_table.rs");

pub type SplitThr = [f32; 64];
type SplitThrs = [[SplitThr; 17]; 8];

#[derive(GetSize)]
pub enum SplitThresholds<'a> {
    Thresholds(&'a SplitThr),
    Threshold(u32),
}

pub struct DyftFactors;

impl DyftFactors {
    #[inline]
    pub const fn split_thresholds<'a>(
        split_threshold: Option<u32>,
        bits: usize,
        radius: u8,
    ) -> SplitThresholds<'a> {
        if let Some(t) = split_threshold {
            SplitThresholds::Threshold(t)
        } else {
            Self::abort_if_out(bits, radius);
            SplitThresholds::Thresholds(&NEW_SPLIT_THRS[bits - 1][usize::from(radius)])
        }
    }

    #[inline]
    const fn abort_if_out(bits: usize, radius: u8) {
        assert!(bits < 9 && bits > 0);
        assert!(radius < 17);
    }
}

pub type HamTable = [[HamTableEntry; 256]; 8];
pub type HamTableEntry = [HamDist; 256];
pub type HamDist = u8;
pub type BitPositionsEntry = [usize; 10];

pub struct HamTables;

impl HamTables {
    /// Hamming distance table
    #[inline]
    pub fn hamming_distance<'a>(bits: usize, label: u8) -> &'a HamTableEntry {
        &HAMMING_DISTANCE_TABLE[bits - 1][usize::from(label)]
    }
    /// Lookup table
    #[inline]
    pub fn lookup<'a>(bits: usize, label: u8) -> &'a HamTableEntry {
        &LOOKUP_TABLE[bits - 1][usize::from(label)]
    }
    /// Byte position table
    #[inline]
    pub fn bit_positions<'a>(bits: usize) -> &'a BitPositionsEntry {
        &BIT_POSITION_TABLE[bits - 1]
    }
}

pub const BIT_POSITION_TABLE: [BitPositionsEntry; 8] = [
    // b=1
    [0, 1, 9, 37, 93, 163, 219, 247, 255, 256],
    // b=2
    [0, 1, 13, 67, 175, 256, 256, 256, 256, 256],
    // b=3
    [0, 1, 15, 64, 64, 64, 64, 64, 64, 64],
    // b=4
    [0, 1, 31, 256, 256, 256, 256, 256, 256, 256],
    // b=5
    [0, 1, 32, 32, 32, 32, 32, 32, 32, 32],
    // b=6
    [0, 1, 64, 64, 64, 64, 64, 64, 64, 64],
    // b=7
    [0, 1, 128, 128, 128, 128, 128, 128, 128, 128],
    // b=8
    [0, 1, 256, 256, 256, 256, 256, 256, 256, 256],
];
