#[doc(inline)]
pub use self::string::BitString;

#[doc(inline)]
pub use self::sparse_one_nnd::SparseOneNnd;

pub mod fixnum;
pub mod ops;
pub mod sparse_one_nnd;
pub mod string;

pub type Index = u64;
pub type Rank = u64;

pub type Bit = bool;
pub const ZERO: Bit = false;
pub const ONE: Bit = true;
