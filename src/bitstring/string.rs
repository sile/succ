use super::Rank;
use super::Index;
use super::block::Block;
use super::ops::{RankZero, RankOne, SelectZero, SelectOne, PredZero, PredOne, SuccZero, SuccOne};

pub struct BitString<B = u64> {
    blocks: Vec<B>,
    len: Index,
}
impl Default for BitString<u64> {
    fn default() -> Self {
        Self::new()
    }
}
impl<B> BitString<B>
    where B: Block
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
            self.blocks.push(B::zero());
        }
        self.blocks[base as usize].set(offset, bit);
        self.len += 1;
    }
    pub fn len(&self) -> Index {
        self.len
    }
    pub fn iter(&self) -> Iter<B> {
        Iter::new(self)
    }
    pub fn as_blocks(&self) -> &[B] {
        &self.blocks
    }
    pub fn into_blocks(self) -> Vec<B> {
        self.blocks
    }

    fn base_and_offset(index: Index) -> (usize, Index) {
        ((index / B::len() as Index) as usize, index % B::len() as Index)
    }
}
impl<B> RankZero for BitString<B>
    where B: Block
{
    fn rank_zero(&self, index: Index) -> Rank {
        index - self.rank_one(index)
    }
}
impl<B> RankOne for BitString<B>
    where B: Block
{
    fn rank_one(&self, index: Index) -> Rank {
        let mut rank = 0;
        let mut rest = index;
        for b in self.as_blocks() {
            if rest > B::len() as Index {
                rank += b.pop_count() as Rank;
                rest -= B::len() as Index;
            } else {
                rank += b.rank_one(rest);
                break;
            }
        }
        rank
    }
}
impl<B> SelectZero for BitString<B>
    where B: Block
{
    fn select_zero(&self, rank: Rank) -> Option<Index> {
        if rank == 0 {
            return None;
        }

        let mut rest = rank;
        let mut index = 0 as Index;
        for b in self.as_blocks() {
            let zeros = (B::len() - b.pop_count()) as Rank;
            if zeros < rest {
                rest -= zeros;
                index += B::len() as Index;
            } else {
                index += b.select_zero(rest).unwrap();
                return Some(index);
            }
        }
        None
    }
}
impl<B> SelectOne for BitString<B>
    where B: Block
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
                index += B::len() as Index;
            } else {
                index += b.select_one(rest).unwrap();
                return Some(index);
            }
        }
        None
    }
}
impl<B> PredZero for BitString<B>
    where B: Block
{
    fn pred_zero(&self, index: Index) -> Option<Index> {
        self.select_zero(self.rank_zero(index))
    }
}
impl<B> PredOne for BitString<B>
    where B: Block
{
    fn pred_one(&self, index: Index) -> Option<Index> {
        self.select_one(self.rank_one(index))
    }
}
impl<B> SuccZero for BitString<B>
    where B: Block
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
impl<B> SuccOne for BitString<B>
    where B: Block
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

pub struct Iter<'a, B: 'a> {
    bs: &'a BitString<B>,
    i: Index,
}
impl<'a, B: 'a> Iter<'a, B> {
    pub fn new(bs: &'a BitString<B>) -> Self {
        Iter { bs: bs, i: 0 }
    }
}
impl<'a, B: 'a> Iterator for Iter<'a, B>
    where B: Block
{
    type Item = bool;
    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1;
        self.bs.get(self.i - 1)
    }
}
