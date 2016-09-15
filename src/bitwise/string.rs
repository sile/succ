use std::mem;
use std::fmt;
use std::iter;

use super::Bit;
use super::Rank;
use super::Index;
use super::fixnum::Fixnum;
use super::fixnum::FixnumLike;
use super::ops;
use super::ops::{RankBit, SelectZero, SelectOne, PredZero, PredOne, SuccZero, SuccOne};

#[derive(Debug, Clone)]
pub struct BitString<N = u64> {
    fixnums: Vec<Fixnum<N>>,
    len: Index,
}
impl Default for BitString<u64> {
    fn default() -> Self {
        Self::new()
    }
}
impl<N> BitString<N>
    where N: FixnumLike
{
    pub fn new() -> Self {
        Self::with_capacity(0)
    }
    pub fn with_capacity(capacity: Index) -> Self {
        BitString {
            fixnums: Vec::with_capacity((capacity / N::bitwidth() as Index) as usize + 1),
            len: 0,
        }
    }
    pub fn get(&self, index: Index) -> Option<Bit> {
        if index < self.len() {
            let (base, offset) = Self::base_and_offset(index);
            Some(unsafe { self.fixnums.get_unchecked(base) }.get(offset))
        } else {
            None
        }
    }
    pub fn resize(&mut self, size: Index) {
        let new_len = (size / N::bitwidth() as u64) as usize + 1;
        self.fixnums.resize(new_len, Fixnum::zero());
        self.len = size;
    }
    pub fn set(&mut self, index: Index, bit: Bit) {
        assert!(index < self.len());
        let (base, offset) = Self::base_and_offset(index);
        self.fixnums[base as usize].set(offset, bit);
    }
    pub fn push(&mut self, bit: Bit) {
        if bit {
            let (base, offset) = Self::base_and_offset(self.len);
            while self.fixnums.len() <= base as usize {
                self.fixnums.push(Fixnum::zero());
            }
            self.fixnums[base as usize].set(offset, bit);
        }
        self.len += 1;
    }
    pub fn len(&self) -> Index {
        self.len
    }
    pub fn shrink_to_fit(&mut self) {
        self.fixnums.shrink_to_fit();
    }
    pub fn iter(&self) -> Iter<N> {
        Iter::new(self)
    }
    pub fn one_indices(&self) -> OneIndices<N> {
        OneIndices::new(self)
    }
    pub fn as_fixnums(&self) -> &[Fixnum<N>] {
        &self.fixnums
    }
    pub fn into_fixnums(self) -> Vec<Fixnum<N>> {
        self.fixnums
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
        for b in self.as_fixnums() {
            if rest >= N::bitwidth() as Index {
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
        let mut rest_len = self.len();
        let mut index = 0 as Index;
        for b in self.as_fixnums() {
            let mut zeros = (N::bitwidth() - b.pop_count()) as Rank;

            if rest_len < N::bitwidth() as Index {
                zeros -= N::bitwidth() as Index - rest_len;
            } else {
                rest_len -= N::bitwidth() as Index;
            }

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
        for b in self.as_fixnums() {
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
        ops::naive_pred_zero(self, index)
    }
}
impl<N> PredOne for BitString<N>
    where N: FixnumLike
{
    fn pred_one(&self, index: Index) -> Option<Index> {
        ops::naive_pred_one(self, index)
    }
}
impl<N> SuccZero for BitString<N>
    where N: FixnumLike
{
    fn succ_zero(&self, index: Index) -> Option<Index> {
        ops::naive_succ_zero(self, index)
    }
}
impl<N> SuccOne for BitString<N>
    where N: FixnumLike
{
    fn succ_one(&self, index: Index) -> Option<Index> {
        let (mut base, mut offset) = Self::base_and_offset(index);
        while base < self.fixnums.len() {
            if let Some(i) = self.fixnums[base].succ_one(offset) {
                return Some(base as Index * N::bitwidth() as Index + i);
            }
            base += 1;
            offset = 0;
        }
        None
    }
}
impl<N> ops::ExternalByteSize for BitString<N> {
    fn external_byte_size(&self) -> u64 {
        self.fixnums.len() as u64 * mem::size_of::<N>() as u64
    }
}

impl<N> iter::FromIterator<Bit> for BitString<N>
    where N: FixnumLike
{
    fn from_iter<T>(iter: T) -> Self
        where T: IntoIterator<Item = Bit>
    {
        let iter = iter.into_iter();
        let mut bs = Self::with_capacity(iter.size_hint().1.unwrap_or(0) as u64);
        for b in iter {
            bs.push(b);
        }
        bs
    }
}

impl<N> fmt::Display for BitString<N>
    where N: FixnumLike
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for b in self.iter() {
            try!(write!(f, "{}", if b { 1 } else { 0 }))
        }
        Ok(())
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
    type Item = Bit;
    fn next(&mut self) -> Option<Self::Item> {
        self.i += 1;
        self.bs.get(self.i - 1)
    }
}

pub struct OneIndices<'a, N: 'a> {
    bs: &'a BitString<N>,
    i: Index,
}
impl<'a, N: 'a> OneIndices<'a, N> {
    pub fn new(bs: &'a BitString<N>) -> Self {
        OneIndices { bs: bs, i: 0 }
    }
}
impl<'a, N: 'a> Iterator for OneIndices<'a, N>
    where N: FixnumLike
{
    type Item = Index;
    fn next(&mut self) -> Option<Self::Item> {
        self.bs.succ_one(self.i).map(|index| {
            self.i = index + 1;
            index
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use super::super::{Index, Rank};
    use super::super::{ZERO, ONE};
    use super::super::ops::*;

    #[test]
    fn it_works() {
        let bits = [ZERO, ONE, ONE, ONE, ZERO, ONE, ZERO, ZERO, ONE, ZERO, ZERO, ONE, ONE, ZERO,
                    ONE, ONE, ZERO, ONE];
        let mut bs = BitString::<u8>::new();
        for b in &bits {
            bs.push(From::from(*b));
        }
        assert_eq!(bs.iter().collect::<Vec<_>>(), bits);

        let expected = LinearFid::new(bits.iter().cloned());
        for i in 0..bits.len() {
            println!("I: {}", i);

            // rank
            assert_eq!(bs.rank_zero(i as Index), expected.rank_zero(i as Index));
            assert_eq!(bs.rank_one(i as Index), expected.rank_one(i as Index));

            // select
            assert_eq!(bs.select_zero((i + 1) as Rank),
                       expected.select_zero((i + 1) as Rank));
            assert_eq!(bs.select_one((i + 1) as Rank),
                       expected.select_one((i + 1) as Rank));

            // pred
            assert_eq!(bs.pred_zero(i as Index), expected.pred_zero(i as Index));
            assert_eq!(bs.pred_one(i as Index), expected.pred_one(i as Index));

            // succ
            assert_eq!(bs.succ_zero(i as Index), expected.succ_zero(i as Index));
            assert_eq!(bs.succ_one(i as Index), expected.succ_one(i as Index));
        }
    }

    #[test]
    fn to_string() {
        let bits = [ZERO, ONE, ONE, ONE, ZERO, ONE, ZERO, ZERO, ONE, ZERO];
        let bs = bits.iter().cloned().collect::<BitString>();
        assert_eq!(bs.to_string(), "0111010010");
    }
}
