use std::fmt;

pub use self::string::BitString;

pub mod ops;
pub mod fixnum;
pub mod string;

pub type Index = u64;
pub type Rank = u64;

#[derive(Debug, Clone, Copy, Hash, PartialOrd, Ord, PartialEq, Eq)]
pub enum Bit {
    Zero = 0,
    One = 1,
}
impl Bit {
    pub fn is_one(&self) -> bool {
        *self == Bit::One
    }
    pub fn is_zero(&self) -> bool {
        *self == Bit::Zero
    }
    pub fn as_bool(&self) -> bool {
        self.is_one()
    }
}
impl From<bool> for Bit {
    fn from(f: bool) -> Self {
        if f { Bit::One } else { Bit::Zero }
    }
}
impl fmt::Display for Bit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
