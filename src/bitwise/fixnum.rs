use std::mem;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Not, Shl, Shr, Sub};

use super::Bit;
use super::ops::GetClose;
use super::ops::RankBit;
use super::ops::{SelectZero, SelectOne};
use super::ops::{PredZero, PredOne};
use super::ops::{SuccZero, SuccOne};
use super::{Index, Rank};

pub trait FixnumLike
    where Self: Sized + Copy + Eq + U64Like,
          Self: Add<Output = Self> + Sub<Output = Self> + Not<Output = Self>,
          Self: BitAnd<Output = Self> + BitOr<Output = Self> + BitXor<Output = Self>,
          Self: Shr<Index, Output = Self> + Shl<Index, Output = Self>
{
    fn bitwidth() -> usize {
        mem::size_of::<Self>() * 8
    }
}
impl<T> FixnumLike for T
    where T: Sized + Copy + Eq + U64Like,
          T: Add<Output = T> + Sub<Output = T> + Not<Output = T>,
          T: BitAnd<Output = T> + BitOr<Output = T> + BitXor<Output = T>,
          T: Shr<Index, Output = T> + Shl<Index, Output = T>
{
}

#[derive(Debug, Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fixnum<T>(T);
impl<T> Fixnum<T>
    where T: FixnumLike
{
    pub fn new(n: T) -> Self {
        Fixnum(n)
    }
    pub fn zero() -> Self {
        Fixnum(T::zero())
    }
    pub fn one() -> Self {
        Fixnum(T::one())
    }
    pub fn bitwidth() -> usize {
        T::bitwidth()
    }
    pub fn to_inner(&self) -> T {
        self.0
    }
    pub fn get(&self, index: Index) -> Bit {
        Bit::from((self.0 & (T::one() << index)) != T::zero())
    }
    pub fn set(&mut self, index: Index, bit: Bit) {
        let x = self.0;
        if bit.is_one() {
            self.0 = x | T::one() << index;
        } else {
            self.0 = x ^ (T::one() << index) & x;
        }
    }
    pub fn pop_count(&self) -> usize {
        let mut x = self.0;
        x = x - ((x >> 1) & T::from_u64(0x5555555555555555));
        x = (x & T::from_u64(0x3333333333333333)) + ((x >> 2) & T::from_u64(0x3333333333333333));
        if Self::bitwidth() == 8 {
            x = x + (x >> 4);
        } else {
            x = (x + (x >> 4)) & T::from_u64(0x0F0F0F0F0F0F0F0F);
        }
        if Self::bitwidth() > 8 {
            x = x + (x >> 8);
        }
        if Self::bitwidth() > 16 {
            x = x + (x >> 16);
        }
        if Self::bitwidth() > 32 {
            x = x + (x >> 32);
        }
        (x & (T::from_u64((Self::bitwidth() as u64) << 1) - T::one())).to_u64() as usize
    }
}
impl<T> RankBit for Fixnum<T>
    where T: FixnumLike
{
    fn rank_one(&self, index: Index) -> Rank {
        debug_assert!(index < Self::bitwidth() as Index);
        let n = if index + 1 == Self::bitwidth() as Index {
            self.0
        } else {
            self.0 & ((T::one() << (index + 1)) - T::one())
        };
        Fixnum(n).pop_count() as Rank
    }
}
impl<T> SelectZero for Fixnum<T>
    where T: FixnumLike
{
    fn select_zero(&self, rank: Rank) -> Option<Index> {
        Fixnum(!self.0).select_one(rank)
    }
}
impl<T> SelectOne for Fixnum<T>
    where T: FixnumLike
{
    fn select_one(&self, rank: Rank) -> Option<Index> {
        if rank == 0 {
            return None;
        }
        let x0 = self.0;
        let x1 = x0 - ((x0 >> 1) & T::from_u64(0x5555555555555555));
        let x2 = (x1 & T::from_u64(0x3333333333333333)) +
                 ((x1 >> 2) & T::from_u64(0x3333333333333333));
        let x3 = if Self::bitwidth() == 8 {
            x2 + (x2 >> 4)
        } else {
            (x2 + (x2 >> 4)) & T::from_u64(0x0F0F0F0F0F0F0F0F)
        };
        let x4 = if Self::bitwidth() > 8 {
            x3 + (x3 >> 8)
        } else {
            x3
        };
        let x5 = if Self::bitwidth() > 16 {
            x4 + (x4 >> 16)
        } else {
            x4
        };
        let x6 = if Self::bitwidth() > 32 {
            x5 + (x5 >> 32)
        } else {
            x5
        };
        let pop_count = (x6 & (T::from_u64((Self::bitwidth() as u64) << 1) - T::one()))
            .to_u64() as Rank;
        if pop_count < rank {
            return None;
        }

        let start = match Self::bitwidth() {
            8 => 3,
            16 => 2,
            32 => 1,
            _ => 0,
        };
        let mut rank = rank;
        let mut offset = 0;
        let mut width = T::from_u64(Self::bitwidth() as u64);
        for &x in &[x5, x4, x3, x2, x1, x0][start..] {
            let low = (x >> offset) & (width - T::one());
            width = width >> 1;
            if (low.to_u64() as Rank) < rank {
                rank = rank - low.to_u64() as Rank;
                offset = offset + width.to_u64() as Index;
            }
        }
        Some(offset)
    }
}
impl<T> PredZero for Fixnum<T>
    where T: FixnumLike
{
    fn pred_zero(&self, index: Index) -> Option<Index> {
        Fixnum(!self.0).pred_one(index)
    }
}
impl<T> PredOne for Fixnum<T>
    where T: FixnumLike
{
    fn pred_one(&self, index: Index) -> Option<Index> {
        if self.get(index).is_one() {
            Some(index)
        } else {
            let mut x = self.0 & ((T::one() << index) - T::one());
            if x == T::zero() {
                None
            } else {
                let width = Self::bitwidth();
                x = x | x >> 1;
                x = x | x >> 2;
                x = x | x >> 4;
                if width > 8 {
                    x = x | x >> 8;
                }
                if width > 16 {
                    x = x | x >> 16;
                }
                if width > 32 {
                    x = x | x >> 32;
                }
                let leading_zeros = Fixnum(!x).pop_count();
                Some((width - leading_zeros - 1) as Index)
            }
        }
    }
}
impl<T> SuccZero for Fixnum<T>
    where T: FixnumLike
{
    fn succ_zero(&self, index: Index) -> Option<Index> {
        Fixnum(!self.0).succ_one(index)
    }
}
impl<T> SuccOne for Fixnum<T>
    where T: FixnumLike
{
    fn succ_one(&self, index: Index) -> Option<Index> {
        if self.get(index).is_one() {
            Some(index)
        } else {
            let x = self.0 ^ (self.0 & ((T::one() << index) - T::one()));
            if x == T::zero() {
                None
            } else {
                let trailing_zeros = Fixnum(!x & (x - T::one())).pop_count();
                Some(trailing_zeros as Index)
            }
        }
    }
}
impl<T> GetClose for Fixnum<T>
    where T: FixnumLike
{
    fn get_close(&self, index: Index) -> Option<Index> {
        let mut level = 0;
        for i in index..T::bitwidth() as Index {
            if self.get(i).is_one() {
                level += 1;
            } else {
                level -= 1;
                if level == 0 {
                    return Some(i);
                }
            }
        }
        None
    }
}
impl<T> fmt::Display for Fixnum<T>
    where T: FixnumLike
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..T::bitwidth() {
            try!(write!(f, "{}", self.get(i as Index)));
        }
        Ok(())
    }
}

pub trait U64Like: Sized {
    fn to_u64(&self) -> u64;
    fn from_u64(n: u64) -> Self;
    fn zero() -> Self {
        Self::from_u64(0)
    }
    fn one() -> Self {
        Self::from_u64(1)
    }
}
impl U64Like for u8 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(n: u64) -> Self {
        n as Self
    }
}
impl U64Like for u16 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(n: u64) -> Self {
        n as Self
    }
}
impl U64Like for u32 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }
    fn from_u64(n: u64) -> Self {
        n as Self
    }
}
impl U64Like for u64 {
    fn to_u64(&self) -> u64 {
        *self
    }
    fn from_u64(n: u64) -> Self {
        n
    }
}

#[cfg(test)]
mod test {
    use std;
    use super::*;
    use super::super::Bit::*;
    use super::super::ops::{RankBit, SelectZero, SelectOne};
    use super::super::ops::{PredZero, PredOne, SuccZero, SuccOne};

    fn f(n: u64) -> Fixnum<u64> {
        Fixnum(n)
    }

    #[test]
    fn get_and_set() {
        let mut block = f(0b11010);
        assert_eq!(block.get(0), Zero);
        assert_eq!(block.get(1), One);
        assert_eq!(block.get(2), Zero);
        assert_eq!(block.get(3), One);
        assert_eq!(block.get(4), One);

        block.set(3, Zero);
        assert_eq!(block, f(0b10010));
        block.set(3, Zero);
        assert_eq!(block, f(0b10010));
        block.set(3, One);
        assert_eq!(block, f(0b11010));
    }

    #[test]
    fn pop_count() {
        assert_eq!(f(0b1001010101010111010).pop_count(), 10);
        assert_eq!(f(0).pop_count(), 0);
        assert_eq!(f(std::u64::MAX).pop_count(), 64);
    }

    #[test]
    fn rank_and_select() {
        assert_eq!(f(0b101010010101000001).rank_zero(10), 7);
        assert_eq!(f(0b101010010101000001).select_zero(7), Some(9));

        assert_eq!(f(0b101010010101000001).rank_one(10), 4);
        assert_eq!(f(0b101010010101000001).select_one(4), Some(10));

        assert_eq!(f(0b101010010101000001).select_one(7), Some(17));
        assert_eq!(f(0b101010010101000001).select_one(8), None);

        assert_eq!(f(std::u64::MAX).rank_one(60), 61);
        assert_eq!(f(std::u64::MAX).select_one(61), Some(60));
    }

    #[test]
    fn pred_and_succ() {
        assert_eq!(f(0b101011110101000001).pred_zero(0), None);
        assert_eq!(f(0b101011110101000001).pred_zero(5), Some(5));
        assert_eq!(f(0b101011110101000001).pred_zero(6), Some(5));
        assert_eq!(f(0b101011110101000001).pred_zero(11), Some(9));

        assert_eq!(f(0b101011110101000001).pred_one(0), Some(0));
        assert_eq!(f(0b101011110101000001).pred_one(5), Some(0));
        assert_eq!(f(0b101011110101000001).pred_one(6), Some(6));
        assert_eq!(f(0b101011110101000001).pred_one(11), Some(11));

        assert_eq!(f(0b101011110101000001).succ_zero(0), Some(1));
        assert_eq!(f(0b101011110101000001).succ_zero(5), Some(5));
        assert_eq!(f(0b101011110101000001).succ_zero(6), Some(7));
        assert_eq!(f(0b101011110101000001).succ_zero(11), Some(14));
        assert_eq!(f(0b101011110101000001).succ_zero(30), Some(30));

        assert_eq!(f(0b101011110101000001).succ_one(0), Some(0));
        assert_eq!(f(0b101011110101000001).succ_one(5), Some(6));
        assert_eq!(f(0b101011110101000001).succ_one(6), Some(6));
        assert_eq!(f(0b101011110101000001).succ_one(11), Some(11));
        assert_eq!(f(0b101011110101000001).succ_one(30), None);
    }

    #[test]
    fn to_string() {
        assert_eq!(Fixnum(0b0011110101000001u16).to_string(),
                   "1000001010111100");
    }
}
