use std::mem;
use super::ops::{RankZero, RankOne, SelectZero, SelectOne, PredZero, PredOne, SuccZero, SuccOne};
use super::{Index, Rank};

// pub trait Block: Sized + Default + RankZero + RankOne +
// SelectZero + SelectOne + PredZero + PredOne + SuccZero + SuccOne {
pub trait Block: Sized + Default {
    fn len() -> usize {
        mem::size_of::<Self>() * 8
    }
    fn zero() -> Self {
        Default::default()
    }
    fn get(&self, index: Index) -> bool;
    fn set(&mut self, index: Index, bit: bool);
    fn pop_count(&self) -> usize;
}
// impl Block for u8 {
//     fn get(&self, index: Index) -> bool {
//         debug_assert!(index < Self::len() as Index);
//         (*self & (1 << index)) != 0
//     }
//     fn set(&mut self, index: Index, bit: bool) {
//         debug_assert!(index < Self::len() as Index);
//         if bit {
//             *self |= 1 << index;
//         } else {
//             *self ^= (1 << index) & *self;
//         }
//     }
//     fn pop_count(&self) -> usize {
//         let mut x = *self;
//         x = x - ((x >> 1) & 0x55);
//         x = (x & 0x33) + ((x >> 2) & 0x33);
//         x = x + (x >> 4);
//         (x & 0b1111) as usize
//     }
// }
// impl Block for u16 {
//     fn get(&self, index: Index) -> bool {
//         debug_assert!(index < Self::len() as Index);
//         (*self & (1 << index)) != 0
//     }
//     fn set(&mut self, index: Index, bit: bool) {
//         debug_assert!(index < Self::len() as Index);
//         if bit {
//             *self |= 1 << index;
//         } else {
//             *self ^= (1 << index) & *self;
//         }
//     }
//     fn pop_count(&self) -> usize {
//         let mut x = *self;
//         x = x - ((x >> 1) & 0x5555);
//         x = (x & 0x3333) + ((x >> 2) & 0x3333);
//         x = (x + (x >> 4)) & 0x0F0F;
//         x = x + (x >> 8);
//         (x & 0b11111) as usize
//     }
// }
// impl Block for u32 {
//     fn get(&self, index: Index) -> bool {
//         debug_assert!(index < Self::len() as Index);
//         (*self & (1 << index)) != 0
//     }
//     fn set(&mut self, index: Index, bit: bool) {
//         debug_assert!(index < Self::len() as Index);
//         if bit {
//             *self |= 1 << index;
//         } else {
//             *self ^= (1 << index) & *self;
//         }
//     }
//     fn pop_count(&self) -> usize {
//         let mut x = *self;
//         x = x - ((x >> 1) & 0x55555555);
//         x = (x & 0x33333333) + ((x >> 2) & 0x33333333);
//         x = (x + (x >> 4)) & 0x0F0F0F0F;
//         x = x + (x >> 8);
//         x = x + (x >> 16);
//         (x & 0b111111) as usize
//     }
// }
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
            *self ^= (1 << index) & *self;
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
impl RankZero for u64 {
    fn rank_zero(&self, index: Index) -> Rank {
        (!*self).rank_one(index)
    }
}
impl RankOne for u64 {
    fn rank_one(&self, index: Index) -> Rank {
        (*self & ((0b10 << index) - 1)).pop_count() as Rank
    }
}
impl SelectZero for u64 {
    fn select_zero(&self, rank: Rank) -> Option<Index> {
        (!*self).select_one(rank)
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
impl PredZero for u64 {
    fn pred_zero(&self, index: Index) -> Option<Index> {
        (!*self).pred_one(index)
    }
}
impl PredOne for u64 {
    fn pred_one(&self, index: Index) -> Option<Index> {
        if self.get(index) {
            Some(index)
        } else {
            let mut x = *self & ((1 << index) - 1);
            if x == 0 {
                None
            } else {
                x |= x >> 1;
                x |= x >> 2;
                x |= x >> 4;
                x |= x >> 8;
                x |= x >> 16;
                x |= x >> 32;
                let leading_zeros = (!x).pop_count();
                Some((Self::len() - leading_zeros - 1) as Index)
            }
        }
    }
}
impl SuccZero for u64 {
    fn succ_zero(&self, index: Index) -> Option<Index> {
        (!*self).succ_one(index)
    }
}
impl SuccOne for u64 {
    fn succ_one(&self, index: Index) -> Option<Index> {
        if self.get(index) {
            Some(index)
        } else {
            let x = *self ^ (*self & ((1 << index) - 1));
            if x == 0 {
                None
            } else {
                let trailing_zeros = (!x & (x - 1)).pop_count();
                Some(trailing_zeros as Index)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std;
    use bitstring::ops::{RankZero, RankOne, SelectZero, SelectOne};
    use bitstring::ops::{PredZero, PredOne, SuccZero, SuccOne};
    use super::*;

    #[test]
    fn get_and_set() {
        let mut block = 0b11010 as u64;
        assert_eq!(block.get(0), false);
        assert_eq!(block.get(1), true);
        assert_eq!(block.get(2), false);
        assert_eq!(block.get(3), true);
        assert_eq!(block.get(4), true);

        block.set(3, false);
        assert_eq!(block, 0b10010);
        block.set(3, false);
        assert_eq!(block, 0b10010);
        block.set(3, true);
        assert_eq!(block, 0b11010);
    }

    #[test]
    fn pop_count() {
        assert_eq!(0b1001010101010111010u64.pop_count(), 10);
        assert_eq!(0u64.pop_count(), 0);
        assert_eq!(std::u64::MAX.pop_count(), 64);
    }

    #[test]
    fn rank_and_select() {
        assert_eq!(0b101010010101000001u64.rank_zero(10), 7);
        assert_eq!(0b101010010101000001u64.select_zero(7), Some(9));

        assert_eq!(0b101010010101000001u64.rank_one(10), 4);
        assert_eq!(0b101010010101000001u64.select_one(4), Some(10));

        assert_eq!(0b101010010101000001u64.select_one(7), Some(17));
        assert_eq!(0b101010010101000001u64.select_one(8), None);

        assert_eq!(std::u64::MAX.rank_one(60), 61);
        assert_eq!(std::u64::MAX.select_one(61), Some(60));
    }

    #[test]
    fn pred_and_succ() {
        assert_eq!(0b101011110101000001u64.pred_zero(0), None);
        assert_eq!(0b101011110101000001u64.pred_zero(5), Some(5));
        assert_eq!(0b101011110101000001u64.pred_zero(6), Some(5));
        assert_eq!(0b101011110101000001u64.pred_zero(11), Some(9));

        assert_eq!(0b101011110101000001u64.pred_one(0), Some(0));
        assert_eq!(0b101011110101000001u64.pred_one(5), Some(0));
        assert_eq!(0b101011110101000001u64.pred_one(6), Some(6));
        assert_eq!(0b101011110101000001u64.pred_one(11), Some(11));

        assert_eq!(0b101011110101000001u64.succ_zero(0), Some(1));
        assert_eq!(0b101011110101000001u64.succ_zero(5), Some(5));
        assert_eq!(0b101011110101000001u64.succ_zero(6), Some(7));
        assert_eq!(0b101011110101000001u64.succ_zero(11), Some(14));
        assert_eq!(0b101011110101000001u64.succ_zero(30), Some(30));

        assert_eq!(0b101011110101000001u64.succ_one(0), Some(0));
        assert_eq!(0b101011110101000001u64.succ_one(5), Some(6));
        assert_eq!(0b101011110101000001u64.succ_one(6), Some(6));
        assert_eq!(0b101011110101000001u64.succ_one(11), Some(11));
        assert_eq!(0b101011110101000001u64.succ_one(30), None);
    }
}
