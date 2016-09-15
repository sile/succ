use bitwise;
use super::Bit;
use super::Rank;
use super::Index;

pub trait ExternalByteSize {
    fn external_byte_size(&self) -> u64;
}

pub trait GetClose {
    fn get_close(&self, index: Index) -> Option<Index>;
}

pub trait GetOpen {
    fn get_open(&self, index: Index) -> Option<Index>;
}

pub trait RankBit {
    fn rank_zero(&self, index: Index) -> Rank {
        (index + 1) - self.rank_one(index)
    }
    fn rank_one(&self, index: Index) -> Rank {
        (index + 1) - self.rank_zero(index)
    }
}

pub trait SelectZero {
    fn select_zero(&self, rank: Rank) -> Option<Index>;
}

pub trait SelectOne {
    fn select_one(&self, rank: Rank) -> Option<Index>;
}

pub trait SelectBit: SelectZero + SelectOne {}
impl<T> SelectBit for T where T: SelectZero + SelectOne {}

pub trait PredZero {
    fn pred_zero(&self, index: Index) -> Option<Index>;
}

pub trait PredOne {
    fn pred_one(&self, index: Index) -> Option<Index>;
}

pub trait PredBit: PredZero + PredOne {}
impl<T> PredBit for T where T: PredZero + PredOne {}

pub trait SuccZero {
    fn succ_zero(&self, index: Index) -> Option<Index>;
}

pub trait SuccOne {
    fn succ_one(&self, index: Index) -> Option<Index>;
}

pub trait SuccBit: SuccZero + SuccOne {}
impl<T> SuccBit for T where T: SuccZero + SuccOne {}

pub trait BitVectorZero: RankBit + SelectZero {}
impl<T> BitVectorZero for T where T: RankBit + SelectZero {}

pub trait BitVectorOne: RankBit + SelectOne {}
impl<T> BitVectorOne for T where T: RankBit + SelectOne {}

pub trait BitVector: BitVectorZero + BitVectorOne {}
impl<T> BitVector for T where T: BitVectorZero + BitVectorOne {}

// Nearest Neighnour Dictionary
pub trait NndZero: BitVectorZero + PredZero + SuccZero {}
impl<T> NndZero for T where T: BitVectorZero + PredZero + SuccZero {}

pub trait NndOne: BitVectorOne + PredOne + SuccOne {}
impl<T> NndOne for T where T: BitVectorOne + PredOne + SuccOne {}

//  Full Indexable Dictionary
pub trait Fid: NndZero + NndOne {}
impl<T> Fid for T where T: NndZero + NndOne {}

pub fn naive_pred_zero<T>(this: &T, index: Index) -> Option<Index>
    where T: BitVectorZero
{
    this.select_zero(this.rank_zero(index))
}

pub fn naive_pred_one<T>(this: &T, index: Index) -> Option<Index>
    where T: BitVectorOne
{
    this.select_one(this.rank_one(index))
}

pub fn naive_succ_zero<T>(this: &T, index: Index) -> Option<Index>
    where T: BitVectorZero
{
    let rank = this.rank_zero(index);
    if Some(index) == this.select_zero(rank) {
        Some(index)
    } else {
        this.select_zero(rank + 1)
    }
}

pub fn naive_succ_one<T>(this: &T, index: Index) -> Option<Index>
    where T: BitVectorOne
{
    let rank = this.rank_one(index);
    if Some(index) == this.select_one(rank) {
        Some(index)
    } else {
        this.select_one(rank + 1)
    }
}

pub struct LinearFid<T> {
    iter: T,
}
impl<T> LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    pub fn new(iter: T) -> Self {
        LinearFid { iter: iter }
    }
}
impl<T> RankBit for LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    fn rank_one(&self, index: Index) -> Rank {
        assert_eq!(index + 1, (index + 1) as usize as Index);
        self.iter.clone().take((index + 1) as usize).filter(|b| *b).count() as Rank
    }
}
impl<T> SelectZero for LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    fn select_zero(&self, rank: Rank) -> Option<Index> {
        if rank == 0 {
            None
        } else {
            assert_eq!(rank, rank as usize as Rank);
            self.iter
                .clone()
                .enumerate()
                .filter(|&(_, b)| !b)
                .map(|(i, _)| i as Index)
                .nth(rank as usize - 1)
        }
    }
}
impl<T> SelectOne for LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    fn select_one(&self, rank: Rank) -> Option<Index> {
        if rank == 0 {
            None
        } else {
            assert_eq!(rank, rank as usize as Rank);
            self.iter
                .clone()
                .enumerate()
                .filter(|&(_, b)| b)
                .map(|(i, _)| i as Index)
                .nth(rank as usize - 1)
        }
    }
}
impl<T> PredZero for LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    fn pred_zero(&self, index: Index) -> Option<Index> {
        assert_eq!(index + 1, (index + 1) as usize as Index);
        self.iter
            .clone()
            .take((index + 1) as usize)
            .enumerate()
            .filter(|&(_, b)| !b)
            .map(|(i, _)| i as Index)
            .last()
    }
}
impl<T> PredOne for LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    fn pred_one(&self, index: Index) -> Option<Index> {
        assert_eq!(index + 1, (index + 1) as usize as Index);
        self.iter
            .clone()
            .take((index + 1) as usize)
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(i, _)| i as Index)
            .last()
    }
}
impl<T> SuccZero for LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    fn succ_zero(&self, index: Index) -> Option<Index> {
        assert_eq!(index, index as usize as Index);
        let mut suffix = self.iter.clone().skip(index as usize);
        if suffix.next() == Some(bitwise::ZERO) {
            Some(index)
        } else {
            suffix.position(|b| !b).map(|i| index + i as Index + 1)
        }
    }
}
impl<T> SuccOne for LinearFid<T>
    where T: Iterator<Item = Bit> + Clone
{
    fn succ_one(&self, index: Index) -> Option<Index> {
        assert_eq!(index, index as usize as Index);
        let mut suffix = self.iter.clone().skip(index as usize);
        if suffix.next() == Some(bitwise::ONE) {
            Some(index)
        } else {
            suffix.position(|b| b).map(|i| index + i as Index + 1)
        }
    }
}
