// https://github.com/kampersanda/dyft/blob/master/include/vcode_tools.hpp

use num_traits::bounds::Bounded;
use num_traits::FromPrimitive;
use num_traits::One;
use num_traits::PrimInt;
use num_traits::ToPrimitive;
use num_traits::WrappingAdd;
use num_traits::WrappingMul;
use num_traits::Zero;
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::Add;
use std::ops::Mul;
pub struct BitTools;

impl BitTools {
    #[inline]
    pub const fn popcnt<T: PrimInt>(x: T) -> u32 {
        x.count_ones()
    }

    #[inline]
    pub const fn bitmask(idx: u32) -> u64 {
        (1u64 << idx as u64) - 1u64
    }
}

pub trait VCodeToolsBase:
    PrimInt
    + ToPrimitive
    + FromPrimitive
    + Bounded
    + Debug
    + Zero
    + One
    + Add
    + Mul
    + Sum
    + WrappingAdd
    + WrappingMul
    + Hash
    + Send
    + Sync
{
    fn size_of() -> usize {
        std::mem::size_of::<Self>() * std::mem::size_of::<*const Self>()
    }
}

pub trait VCodeTools: VCodeToolsBase {
    const N_DIM: usize;

    fn wrap_to_t(f: f64) -> Self;

    fn hamdist(x: &[Self], y: &[Self], bits: usize) -> u32;

    fn hamdist_radius(x: &[Self], y: &[Self], bits: usize, radius: u32) -> u32;

    fn to_vint(bytes_in: &[u8], j: usize) -> Self;

    fn to_vints<'a, 'b>(bytes_in: &'a [u8], bits: usize) -> impl Iterator<Item = Self> + 'b
    where
        'a: 'b,
    {
        (0..bits).map(move |i| Self::to_vint(bytes_in, i))
    }

    fn to_byte(x: &[Self], i: usize, bits: usize) -> u8;
}

macro_rules! impl_vcode_tools {
    ($t: ty) => {
        impl VCodeToolsBase for $t {}

        impl VCodeTools for $t {
            
            const N_DIM: usize = std::mem::size_of::<Self>() * 8;

            // https://users.rust-lang.org/t/how-to-make-wrapping-cast-instead-of-the-saturating-one-on-i32/100291
            fn wrap_to_t(f: f64) -> Self {
                f.rem_euclid((Self::MAX as f64) + 1.0) as Self
            }

            fn hamdist(x: &[Self], y: &[Self], bits: usize) -> u32 {
                if bits == 1 {
                    BitTools::popcnt(x[0] ^ y[0])
                } else {
                    let mut diff: Self = 0;
                    for j in 0..bits {
                        diff |= (x[j]) ^ (y[j]);
                    }
                    BitTools::popcnt(diff)
                }
            }

            fn hamdist_radius(x: &[Self], y: &[Self], bits: usize, radius: u32) -> u32 {
                if bits == 1 {
                    BitTools::popcnt(x[0] ^ y[0])
                } else {
                    #[allow(unused_assignments)]
                    let mut dist = 0;
                    let mut diff: Self = 0;
                    for j in 0..bits {
                        let j = j as usize;
                        diff |= (x[j]) ^ (y[j]);
                        dist = BitTools::popcnt(diff);
                        if dist > radius {
                            return dist;
                        }
                    }
                    BitTools::popcnt(diff)
                }
            }

            fn to_vint(bytes_in: &[u8], j: usize) -> Self {
                assert!(Self::size_of() <= bytes_in.len(), "bytes_in.len() ({}) must be greater than or equal to Self::size_of() ({})", bytes_in.len(), Self::size_of());
                let mut v: Self = 0;
                for i in 0..Self::size_of() {
                    let b: Self = ((bytes_in[i] >> j) & 1).into();
                    v |= b << i;
                }
                v
            }

            fn to_byte(x: &[Self], i: usize, bits: usize) -> u8 {
                assert!(Self::size_of() >= bits as usize);
                if bits == 1 {
                    ((x[0] >> i) & 1) as u8
                } else {
                    let mut c: u8 = 0;
                    for j in 0..bits {
                        c |= (((x[j] >> i) & 1) << j) as u8;
                    }
                    c
                }
            }
        }
    };

    ($( $t: ty ),*) => {
        $( impl_vcode_tools!($t); )*
    };
}

impl_vcode_tools!(u8, u16, u32, u64, u128, usize);
// impl_vcode_tools!(i16, i32, i64, i128, isize);

#[cfg(test)]
mod test {
    use super::*;

    const BYTES_IN_TEST: &[u8] = &[
        1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
        26, 27, 28, 29, 30, 31, 32,
    ];

    #[test]
    fn hamdist_test() {
        assert_eq!(0, u8::hamdist(&[1, 0, 1], &[1, 0, 1], 3));
        assert_eq!(3, u32::hamdist(&[1, 2, 3, 4, 5], &[1, 2, 4, 4, 5], 3));
        assert_eq!(3, u8::hamdist(&[1, 2, 3, 4, 5], &[1, 2, 4, 4, 5], 3));
    }

    #[test]
    fn to_vint_test() {
        assert_eq!(2139127680, u32::to_vint(BYTES_IN_TEST, 3))
    }

    #[test]
    fn to_vints_test() {
        let mut out = u32::to_vints(BYTES_IN_TEST, 3);
        assert_eq!(Some(1431655765), out.next());
        assert_eq!(Some(1717986918), out.next());
        assert_eq!(Some(2021161080), out.next());
        assert_eq!(None, out.next());
    }
}
