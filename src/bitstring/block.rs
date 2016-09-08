use std::mem;
use super::ops::{RankOne, SelectOne};
use super::{Index, Rank};

pub trait Block: Sized + Default {
    fn len() -> usize {
        mem::size_of::<Self>()
    }
    fn zero() -> Self {
        Default::default()
    }
    fn get(&self, index: Index) -> bool;
    fn set(&mut self, index: Index, bit: bool);
    fn pop_count(&self) -> usize;
}
impl Block for u8 {
    fn get(&self, index: Index) -> bool {
        debug_assert!(index < Self::len() as Index);
        (*self & (1 << index)) != 0
    }
    fn set(&mut self, index: Index, bit: bool) {
        debug_assert!(index < Self::len() as Index);
        if bit {
            *self |= 1 << index;
        } else {
            *self ^= (1 << index) | !*self;
        }
    }
    fn pop_count(&self) -> usize {
        let mut x = *self;
        x = x - ((x >> 1) & 0x55);
        x = (x & 0x33) + ((x >> 2) & 0x33);
        x = x + (x >> 4);
        (x & 0b1111) as usize
    }
}
impl Block for u16 {
    fn get(&self, index: Index) -> bool {
        debug_assert!(index < Self::len() as Index);
        (*self & (1 << index)) != 0
    }
    fn set(&mut self, index: Index, bit: bool) {
        debug_assert!(index < Self::len() as Index);
        if bit {
            *self |= 1 << index;
        } else {
            *self ^= (1 << index) | !*self;
        }
    }
    fn pop_count(&self) -> usize {
        let mut x = *self;
        x = x - ((x >> 1) & 0x5555);
        x = (x & 0x3333) + ((x >> 2) & 0x3333);
        x = (x + (x >> 4)) & 0x0F0F;
        x = x + (x >> 8);
        (x & 0b11111) as usize
    }
}
impl Block for u32 {
    fn get(&self, index: Index) -> bool {
        debug_assert!(index < Self::len() as Index);
        (*self & (1 << index)) != 0
    }
    fn set(&mut self, index: Index, bit: bool) {
        debug_assert!(index < Self::len() as Index);
        if bit {
            *self |= 1 << index;
        } else {
            *self ^= (1 << index) | !*self;
        }
    }
    fn pop_count(&self) -> usize {
        let mut x = *self;
        x = x - ((x >> 1) & 0x55555555);
        x = (x & 0x33333333) + ((x >> 2) & 0x33333333);
        x = (x + (x >> 4)) & 0x0F0F0F0F;
        x = x + (x >> 8);
        x = x + (x >> 16);
        (x & 0b111111) as usize
    }
}
impl Block for u64 {
    fn get(&self, index: Index) -> bool {
        debug_assert!(index < Self::len() as Index);
        (*self & (1 << index)) != 0
    }
    fn set(&mut self, index: Index, bit: bool) {
        debug_assert!(index < Self::len() as Index);
        if bit {
            *self |= 1 << index;
        } else {
            *self ^= (1 << index) | !*self;
        }
    }
    fn pop_count(&self) -> usize {
        let mut x = *self;
        x = x - ((x >> 1) & 0x5555555555555555);
        x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
        x = (x + (x >> 4)) & 0x0F0F0F0F0F0F0F0F;
        x = x + (x >> 8);
        x = x + (x >> 16);
        x = x + (x >> 32);
        (x & 0b1111111) as usize
    }
}
impl RankOne for u64 {
    fn rank_one(&self, index: Index) -> Rank {
        (*self & ((0b10 << index) - 1)).pop_count() as Rank
    }
}
impl SelectOne for u64 {
    fn select_one(&self, mut rank: Rank) -> Option<Index> {
        if rank == 0 {
            return None;
        }
        let x0 = *self;
        let x1 = x0 - ((x0 >> 1) & 0x5555555555555555);
        let x2 = (x1 & 0x3333333333333333) + ((x1 >> 2) & 0x3333333333333333);
        let x3 = (x2 + (x2 >> 4)) & 0x0F0F0F0F0F0F0F0F;
        let x4 = x3 + (x3 >> 8);
        let x5 = x4 + (x4 >> 16);
        let x6 = x5 + (x5 >> 32);
        if x6 & 0b1111111 < rank {
            return None;
        }

        let mut offset = 0;
        let mut width = 64;
        for x in &[x5, x4, x3, x2, x1, x0] {
            let low = (x >> offset) & (width - 1);
            width >>= 1;
            if low < rank {
                rank -= low;
                offset += width;
            }
        }
        Some(offset)
    }
}
