use super::Index;

#[allow(dead_code)]
pub struct BitString<B = u64> {
    blocks: Vec<B>,
    len: Index,
}
