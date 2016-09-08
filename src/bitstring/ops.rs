use super::Rank;
use super::Index;

pub trait RankZero {
    fn rank_zero(&self, index: Index) -> Rank;
}

pub trait RankOne {
    fn rank_one(&self, index: Index) -> Rank;
}

pub trait SelectZero {
    fn select_zero(&self, rank: Rank) -> Option<Index>;
}

pub trait SelectOne {
    fn select_one(&self, rank: Rank) -> Option<Index>;
}

pub trait PredZero {
    fn pred_zero(&self, index: Index) -> Option<Index>;
}

pub trait PredOne {
    fn pred_one(&self, index: Index) -> Option<Index>;
}

pub trait SuccZero {
    fn succ_zero(&self, index: Index) -> Option<Index>;
}

pub trait SuccOne {
    fn succ_one(&self, index: Index) -> Option<Index>;
}
