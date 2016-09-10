use super::Rank;
use super::Index;

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
