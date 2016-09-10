use super::Rank;
use super::Index;
use super::fixnum::Fixnum;
use super::fixnum::FixnumLike;
use super::ops::{RankBit, SelectZero, SelectOne, PredZero, PredOne, SuccZero, SuccOne};

#[derive(Debug, Clone)]
pub struct BitString<N = u64> {
    blocks: Vec<Fixnum<N>>,
    len: Index,
}
impl<N> BitString<N>
    where N: FixnumLike
{
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    pub fn with_capacity(capacity: Index) -> Self {
        BitString {
            blocks: Vec::with_capacity((capacity / 8) as usize + 1),
            len: 0,
        }
    }
    pub fn get(&self, index: Index) -> Option<bool> {
        let (base, offset) = Self::base_and_offset(index);
        self.blocks.get(base).map(|b| b.get(offset))
    }
    pub fn push(&mut self, bit: bool) {
        let (base, offset) = Self::base_and_offset(self.len);
        if self.blocks.len() == base as usize {
            self.blocks.push(Fixnum::zero());
        }
        self.blocks[base as usize].set(offset, bit);
        self.len += 1;
    }
    pub fn len(&self) -> Index {
        self.len
    }
    pub fn iter(&self) -> Iter<N> {
        Iter::new(self)
    }
    pub fn as_blocks(&self) -> &[Fixnum<N>] {
        &self.blocks
    }
    pub fn into_blocks(self) -> Vec<Fixnum<N>> {
        self.blocks
    }

    fn base_and_offset(index: Index) -> (usize, Index) {
        ((index / N::bitwidth() as Index) as usize, index % N::bitwidth() as Index)
    }
}
impl<N> RankBit for BitString<N>
    where N: FixnumLike
{
    fn rank_one(&self, index: Index) -> Rank {
        let mut rank = 0;
        let mut rest = index;
        for b in self.as_blocks() {
            if rest > N::bitwidth() as Index {
                rank += b.pop_count() as Rank;
                rest -= N::bitwidth() as Index;
            } else {
                rank += b.rank_one(rest);
                break;
            }
        }
        rank
    }
}
impl<N> SelectZero for BitString<N>
    where N: FixnumLike
{
    fn select_zero(&self, rank: Rank) -> Option<Index> {
        if rank == 0 {
            return None;
        }

        let mut rest = rank;
        let mut index = 0 as Index;
        for b in self.as_blocks() {
            let zeros = (N::bitwidth() - b.pop_count()) as Rank;
            if zeros < rest {
                rest -= zeros;
                index += N::bitwidth() as Index;
            } else {
                index += b.select_zero(rest).unwrap();
                return Some(index);
            }
        }
        None
    }
}
impl<N> SelectOne for BitString<N>
    where N: FixnumLike
{
    fn select_one(&self, rank: Rank) -> Option<Index> {
        if rank == 0 {
            return None;
        }

        let mut rest = rank;
        let mut index = 0 as Index;
        for b in self.as_blocks() {
            let ones = b.pop_count() as Rank;
            if ones < rest {
                rest -= ones;
                index += N::bitwidth() as Index;
            } else {
                index += b.select_one(rest).unwrap();
                return Some(index);
            }
        }
        None
    }
}
impl<N> PredZero for BitString<N>
    where N: FixnumLike
{
    fn pred_zero(&self, index: Index) -> Option<Index> {
        self.select_zero(self.rank_zero(index))
    }
}
impl<N> PredOne for BitString<N>
    where N: FixnumLike
{
    fn pred_one(&self, index: Index) -> Option<Index> {
        self.select_one(self.rank_one(index))
    }
}
impl<N> SuccZero for BitString<N>
    where N: FixnumLike
{
    fn succ_zero(&self, index: Index) -> Option<Index> {
        let rank = self.rank_zero(index);
        if Some(index) == self.select_zero(rank) {
            Some(index)
        } else {
            self.select_zero(rank + 1)
        }
    }
}
impl<N> SuccOne for BitString<N>
    where N: FixnumLike
{
    fn succ_one(&self, index: Index) -> Option<Index> {
        let rank = self.rank_one(index);
        if Some(index) == self.select_one(rank) {
            Some(index)
        } else {
            self.select_one(rank + 1)
        }
    }
}

pub struct Iter<'a, N: 'a> {
    bs: &'a BitString<N>,
    i: Index,
}
impl<'a, N: 'a> Iter<'a, N> {
    pub fn new(bs: &'a BitString<N>) -> Self {
        Iter { bs: bs, i: 0 }
    }
}
impl<'a, N: 'a> Iterator for Iter<'a, N>
    where N: FixnumLike
{
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1;
        self.bs.get(self.i - 1)
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {}
}
