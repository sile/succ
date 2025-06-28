use bitwise;
use bitwise::fixnum::Fixnum;
use bitwise::fixnum::FixnumLike;
use bitwise::ops::ExternalByteSize;
use bitwise::ops::GetClose;
use bitwise::ops::NndOne;
use bitwise::Bit;
use bitwise::Index;

pub type Block = u64;

pub type BitString = bitwise::BitString<Block>;

const OPEN: Bit = bitwise::ONE;
const CLOSE: Bit = bitwise::ZERO;

const BLOCK_SIZE: Index = 64; // TODO

impl Fixnum<Block> {
    fn relative_level(&self, parent: Index, child: Index) -> Index {
        assert!(parent <= child, "{} <= {}", parent, child);
        assert!(
            self.left_excess(child) >= self.left_excess(parent),
            "{}@{} <= {}@{}",
            child,
            self.left_excess(child),
            parent,
            self.left_excess(parent)
        );
        self.left_excess(child) - self.left_excess(parent)
    }
    fn far_child(&self, index: Index, level: Index) -> Index {
        if level == 0 {
            return index;
        }
        assert!(index > 0);

        // TODO: refactor
        let mut j = None;
        let mut i = index - 1;
        let mut l = 0;
        loop {
            if self.get(i) == CLOSE {
                l += 1;
                if level == l {
                    j = Some(i);
                }
            } else {
                l -= 1;
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        j.unwrap()
    }
    fn left_excess(&self, index: Index) -> Index {
        let mut level = 0;
        for i in 0..index {
            if self.get(i) == OPEN {
                level += 1;
            } else {
                if level != 0 {
                    level -= 1;
                }
            }
        }
        level
    }
}

#[derive(Debug)]
pub struct Parens<N> {
    bits: BitString,
    pioneers: Option<Box<PioneerFamily<N>>>,
}
impl<N> Parens<N>
where
    N: NndOne + From<BitString>,
{
    pub fn new(bits: BitString) -> Self {
        let pioneers = if bits.len() > Block::bitwidth() as Index {
            Some(Box::new(PioneerFamily::new(&bits)))
        } else {
            None
        };
        Parens {
            bits: bits,
            pioneers: pioneers,
        }
    }
}
impl<N> ExternalByteSize for Parens<N>
where
    N: ExternalByteSize,
{
    fn external_byte_size(&self) -> u64 {
        self.bits.external_byte_size()
            + self.pioneers.as_ref().map_or(0, |p| p.external_byte_size())
    }
}
impl<N> Parens<N>
where
    N: NndOne,
{
    pub fn get_close(&self, index: Index) -> Option<Index> {
        debug_assert_eq!(self.bits.get(index).unwrap_or(OPEN), OPEN);
        let base = index / BLOCK_SIZE;
        let offset = index % BLOCK_SIZE;
        let result = self.bits.as_fixnums().get(base as usize).map(|b| {
            b.get_close(offset)
                .map(|i| base * BLOCK_SIZE + i)
                .unwrap_or_else(|| {
                    let pioneers = self.pioneers.as_ref().unwrap();
                    let open_pioneer = pioneers.pred(index);
                    let open_block = open_pioneer / BLOCK_SIZE;
                    let level = if open_block == base {
                        b.relative_level(open_pioneer % BLOCK_SIZE, offset)
                    } else {
                        let next_fix = &self.bits.as_fixnums()[open_block as usize];
                        next_fix.relative_level(open_pioneer % BLOCK_SIZE, 0) +    // inner lvl
                         b.relative_level(0, offset) // this block
                    };

                    let close_pioneer = pioneers.get_close(open_pioneer);
                    let close_block_idx = (close_pioneer / BLOCK_SIZE) as usize;
                    let fixnums = self.bits.as_fixnums();
                    let close_fix = if close_block_idx < fixnums.len() {
                        &fixnums[close_block_idx]
                    } else {
                        /*  Pair crosses past the end (degenerate last word with
                         *  only opens) – fall back to a linear scan. */
                        let mut lvl = 0;
                        for i in index + 1..self.bits.len() {
                            if self.bits.get(i) == Some(OPEN) {
                                lvl += 1
                            } else if lvl == 0 {
                                return i;
                            } else {
                                lvl -= 1
                            }
                        }
                        unreachable!("balanced parentheses guarantee a close exists");
                    };
                    let local_close_index = close_fix.far_child(close_pioneer % BLOCK_SIZE, level);

                    let close_index = (close_pioneer / BLOCK_SIZE * BLOCK_SIZE) + local_close_index;
                    close_index
                })
        });
        result
    }
    pub fn get(&self, index: Index) -> Option<Bit> {
        self.bits.get(index)
    }
}

#[derive(Debug)]
struct PioneerFamily<N> {
    nnd: N,
    parens: Parens<N>,
}
impl<N> PioneerFamily<N>
where
    N: NndOne + From<BitString>,
{
    fn new(bits: &BitString) -> Self {
        let (flags, parens) = extract_pioneers(bits);
        PioneerFamily {
            nnd: From::from(flags),
            parens: Parens::new(parens),
        }
    }
}
impl<N> ExternalByteSize for PioneerFamily<N>
where
    N: ExternalByteSize,
{
    fn external_byte_size(&self) -> u64 {
        self.nnd.external_byte_size() + self.parens.external_byte_size()
    }
}
impl<N> PioneerFamily<N>
where
    N: NndOne,
{
    fn pred(&self, index: Index) -> Index {
        self.nnd.pred_one(index).unwrap()
    }
    fn get_close(&self, index: Index) -> Index {
        // NOTE: predとまとめればrank呼び出し回数を減らせる
        let rank = self.nnd.rank_one(index);
        let close = self.parens.get_close(rank - 1).unwrap();
        self.nnd.select_one(close + 1).unwrap()
    }
}

// TODO: optimize
fn extract_pioneers(bits: &BitString) -> (BitString, BitString) {
    let block_size = BLOCK_SIZE;
    assert!(bits.len() > block_size);
    let block = |i| i / block_size;

    let mut stack = Vec::new();
    let mut flags = BitString::with_capacity(bits.len());
    flags.resize(bits.len());

    let mut last_far = None;
    for (i, b) in bits.iter().enumerate() {
        if b {
            // open parenthesis
            stack.push(i);
        } else {
            // close parenthesis
            let open = stack.pop().unwrap() as Index;
            let close = i as Index;
            if block(open) == block(close) {
                continue;
            }

            // far parenthesis pair
            if let Some((last_open, last_close)) = last_far.take() {
                if block(last_open) != block(open) || block(last_close) != block(close) {
                    flags.set(last_open, true);
                    flags.set(last_close, true);
                }
            }
            last_far = Some((open, close));
        }
    }
    assert!(stack.is_empty(), "STACK: {:?}", stack);
    assert_eq!(last_far, Some((0, bits.len() - 1)));
    let (open, close) = last_far.unwrap();
    flags.set(open, true);
    flags.set(close, true);

    let parens = bits
        .iter()
        .zip(flags.iter())
        .filter(|&(_, p)| p)
        .map(|(bit, _)| bit)
        .collect::<BitString>();
    (flags, parens)
}
