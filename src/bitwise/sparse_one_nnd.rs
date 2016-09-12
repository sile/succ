use std;
use std::iter;

use super::{Bit, Rank, Index};
use super::ops;
use super::ops::{RankBit, SelectOne, PredOne, SuccOne};

// TODO: parameter
const SMALL_SIZE: usize = (std::u8::MAX as usize) + 1;
const MIDDLE_SIZE: usize = SMALL_SIZE * 8;
const MIDDLE_COUNT: usize = 32;
const LARGE_SIZE: usize = MIDDLE_SIZE * MIDDLE_COUNT;

#[derive(Debug, Clone)]
pub struct SparseOneNnd {
    smalles: Vec<u8>,
    middles: Vec<Base<u16>>,
    larges: Vec<Base<u32>>,
}
impl iter::FromIterator<Bit> for SparseOneNnd {
    fn from_iter<I>(bits: I) -> Self
        where I: IntoIterator<Item = Bit>
    {
        let mut larges = Vec::new();
        let mut middles = Vec::new();
        let mut smalles = Vec::new();
        let mut small_count_index = 0;

        let mut rank = 0;
        let mut prev_index = 0;
        let mut middle_prev = Base::new(0, 0);
        let mut large_prev;

        for (i, b) in bits.into_iter().enumerate() {
            let small_base = smalles.len();
            if i % SMALL_SIZE == 0 {
                small_count_index = smalles.len();
                smalles.push(0);
                prev_index = i;
            }
            if i % LARGE_SIZE == 0 {
                large_prev = Base::new(small_base, rank);
                middle_prev = large_prev.clone();
                larges.push(Base::new(large_prev.small_index as u32, large_prev.rank as u32));
            }
            if i % MIDDLE_SIZE == 0 {
                middles.push(Base::new((small_base - middle_prev.small_index) as u16,
                                       (rank - middle_prev.rank) as u16));
            }
            if b.is_zero() {
                continue;
            }
            debug_assert!((i - prev_index) < 0x100);
            rank += 1;
            smalles.push((i - prev_index) as u8);
            smalles[small_count_index] += 1;
        }

        larges.shrink_to_fit();
        middles.shrink_to_fit();
        smalles.shrink_to_fit();

        SparseOneNnd {
            larges: larges,
            middles: middles,
            smalles: smalles,
        }
    }
}
impl RankBit for SparseOneNnd {
    fn rank_one(&self, index: Index) -> Rank {
        let large_index = (index / LARGE_SIZE as Index) as usize;
        let large_base = &self.larges[large_index];
        // let large_offset = large_index as Index * LARGE_SIZE as Index;

        let middle_index = (index / MIDDLE_SIZE as Index) as usize;
        let middle_base = &self.middles[middle_index];
        let middle_offset = middle_index as Index * MIDDLE_SIZE as Index;

        let mut small_index = large_base.small_index as usize + middle_base.small_index as usize;
        let mut curr_rank = large_base.rank as Rank + middle_base.rank as Rank;
        let mut curr_index = /*large_offset + */ middle_offset;
        while curr_index + SMALL_SIZE as Index <= index {
            curr_rank += self.smalles[small_index] as Rank;
            small_index += self.smalles[small_index] as usize + 1;
            curr_index += SMALL_SIZE as Index;
        }

        let count = self.smalles[small_index] as usize;
        assert!(index >= curr_index, "{}, {}", index, curr_index);
        let delta = (index - curr_index) as u8;

        curr_rank +
        self.smalles[small_index + 1..]
            .iter()
            .take(count)
            .take_while(|i| **i <= delta)
            .count() as Rank
    }
}
impl SelectOne for SparseOneNnd {
    fn select_one(&self, rank: Rank) -> Option<Index> {
        if rank == 0 {
            return None;
        }
        let rank = rank - 1;

        //
        let i = self.larges
            .binary_search_by_key(&rank, |e| e.rank as Rank)
            .unwrap_or_else(|i| i - 1);
        let large_base = &self.larges[i];
        let large_index = i as Index * LARGE_SIZE as Index;
        let middle_rank = rank - large_base.rank as Rank;

        let middle_start = i * MIDDLE_COUNT;
        let middle_end = ::std::cmp::min(self.middles.len(), middle_start + MIDDLE_COUNT);
        let middles = &self.middles[middle_start..middle_end];
        {
            let i = middles.binary_search_by_key(&middle_rank, |e| e.rank as Rank)
                .unwrap_or_else(|i| i - 1);
            let middle_base = &middles[i];
            let middle_index = i as Index * MIDDLE_SIZE as Index;

            let mut small_index = large_base.small_index as usize +
                                  middle_base.small_index as usize;
            let mut curr_rank = large_base.rank as Rank + middle_base.rank as Rank;
            let mut curr_index = large_index + middle_index;
            while (curr_rank + self.smalles[small_index] as Rank) <= rank {
                curr_rank += self.smalles[small_index] as Rank;
                curr_index += SMALL_SIZE as Index;
                small_index += self.smalles[small_index] as usize + 1;
                if !(small_index < self.smalles.len()) {
                    return None;
                }
            }

            let delta = (rank - curr_rank) as usize;
            curr_index += self.smalles[small_index + delta + 1] as Index;
            Some(curr_index)
        }
    }
}
impl PredOne for SparseOneNnd {
    fn pred_one(&self, index: Index) -> Option<Index> {
        ops::naive_pred_one(self, index)
    }
}
impl SuccOne for SparseOneNnd {
    fn succ_one(&self, index: Index) -> Option<Index> {
        ops::naive_succ_one(self, index)
    }
}
impl ops::ExternalByteSize for SparseOneNnd {
    fn external_byte_size(&self) -> u64 {
        self.smalles.len() as u64 +
        self.middles.len() as u64 * std::mem::size_of::<Base<u16>>() as u64 +
        self.larges.len() as u64 * std::mem::size_of::<Base<u32>>() as u64
    }
}

#[derive(Debug, Clone)]
struct Base<T> {
    small_index: T,
    rank: T,
}
impl<T> Base<T> {
    fn new(small_index: T, rank: T) -> Self {
        Base {
            small_index: small_index,
            rank: rank,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::{Bit, Index, Rank};
    use super::super::ops::*;

    #[test]
    fn it_works() {
        let bits = (0..1024).map(|i| Bit::from(i % 5 == 0)).collect::<Vec<_>>();
        let expected = LinearFid::new(bits.iter().cloned());
        let nnd = bits.iter().cloned().collect::<SparseOneNnd>();
        for i in 0..bits.len() {
            println!("I: {}", i);

            // rank
            assert_eq!(nnd.rank_one(i as Index), expected.rank_one(i as Index));

            // select
            assert_eq!(nnd.select_one((i + 1) as Rank),
                       expected.select_one((i + 1) as Rank));

            // pred
            assert_eq!(nnd.pred_one(i as Index), expected.pred_one(i as Index));

            // succ
            assert_eq!(nnd.succ_one(i as Index), expected.succ_one(i as Index));
        }
    }
}
