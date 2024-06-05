use crate::dyft::*;
use std::{fmt::Debug, ops::Range};

/// BytePack is a struct that is used to pack a slice of integers into a slice of bytes, i.e.,
/// the struct is used to pack the vcode into a byte array. The struct is created with the
/// number of bits to use for each integer in the vcode, and the begin and end position of the
/// vcode slice. The number of integers to pack into each chunk is calculated from the number of
/// bits and the size of the chunk. The struct has a method to pack a slice of integers into a slice
/// of bytes, and a method to pack a single integer into a byte.
#[derive(Debug)]
pub struct BytePack<T>
where
    T: VCodeTools,
{
    _marker: std::marker::PhantomData<T>,
}

impl<T: VCodeTools> BytePack<T> {
    pub fn ints_per_chunk(bits: usize) -> usize {
        8 / bits
    }

    pub fn pack(vcode: &[T], ints_per_chunk: usize, bits: usize, bpos: usize) -> u8 {
        Self::inner_pack_byte(vcode, ints_per_chunk, bits, bpos)
    }

    pub fn pack_iter<'a>(
        vcode: &'a [T],
        range: Range<usize>,
        bits: usize,
        ints_per_chunk: usize,
    ) -> impl Iterator<Item = u8> + 'a {
        range.map(move |bpos| {
            BytePack::<T>::inner_pack_byte(vcode, ints_per_chunk, bits, bpos)
        })
    }

    pub fn byte_iter<'a>(
        vcode: &'a [T],
        range: Range<usize>,
        bits: usize,
    ) -> impl Iterator<Item = u8> + 'a {
        range.map(move |bpos| T::to_byte(vcode, bpos, bits))
    }

    fn inner_pack_byte(vcode: &[T], ints_per_chunk: usize, bits: usize, bpos: usize) -> u8 {
        if bits == 1 {
            Self::inner_byte_pack_bits(vcode[bpos], ints_per_chunk, bpos)
        } else {
            Self::inner_byte_pack_integers(vcode, ints_per_chunk, bits, bpos)
        }
    }

    fn inner_byte_pack_bits(vcode: T, ints_per_chunk: usize, bpos: usize) -> u8 {
        match vcode.to_u8() {
            Some(byte) => byte >> (bpos as u8 * ints_per_chunk as u8),
            None => panic!("Error packing vcode at pos={bpos}: {:?}", vcode),
        }
    }

    fn inner_byte_pack_integers(
        vcode: &[T],
        ints_per_chunk: usize,
        bits: usize,
        bpos: usize,
    ) -> u8 {
        let s = bpos * ints_per_chunk;
        let e = (s + ints_per_chunk).min(T::N_DIM);
        (s..e).enumerate().fold(0, |acc, (j, i)| {
            acc | T::to_byte(vcode, i, bits) << j * bits
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const SAMPLE_VCODE_32: [u32; 8] = [3348024405, 1932228148, 4083539617, 3119178199, 3348024405, 1932228148, 4083539617, 3119178199];
    // const SAMPLE_VCODE_32: [u32; 4] = [3348024405, 1932228148, 4083539617, 3119178199, 3348024405, 1932228148, 4083539617, 3119178199];

    const SAMPLE_CHUNK: [u8; 32] = [
        141, 11, 107, 201, 104, 87, 166, 223, 182, 181, 224, 156, 127, 129, 238, 215, 0, 0, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    const SAMPLE_BYTES: [u8; 32] = [
        13, 8, 11, 0, 11, 6, 9, 12, 8, 6, 7, 5, 6, 10, 15, 13, 6, 11, 5, 11, 0, 14, 12, 9, 15, 7,
        1, 8, 14, 14, 7, 13,
    ];

    #[test]
    fn test_to_chunks() {
        let bits = SAMPLE_VCODE_32.len();
        let vcodes = VCodeArray::<u32>::new(&SAMPLE_VCODE_32, bits);
        let ints_per_chunk = BytePack::<u32>::ints_per_chunk(bits);
        let bytes =
            BytePack::<u32>::pack_iter(vcodes.access(0), 0..u32::N_DIM, bits, ints_per_chunk)
                .collect::<Vec<u8>>();
        assert_eq!(&SAMPLE_CHUNK, &bytes.as_slice());
    }

    #[test]
    fn test_to_bytes() {
        let bits = SAMPLE_VCODE_32.len();
        let vcodes = VCodeArray::<u32>::new(&SAMPLE_VCODE_32, bits);
        let bytes =
            BytePack::<u32>::byte_iter(vcodes.access(0), 0..u32::N_DIM, bits).collect::<Vec<u8>>();
        assert_eq!(&SAMPLE_BYTES, &bytes.as_slice());
    }
}
