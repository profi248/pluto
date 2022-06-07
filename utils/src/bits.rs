/// An iterator which returns bits from a sequence
/// of numbers, in little-endian and lowest significant bit first.
pub struct BitsIter<'a> {
    data: std::borrow::Cow<'a, [u8]>,
    current_index: usize,
    bits_count: usize,
    index_multiplier: usize,
}

impl<'a> BitsIter<'a> {
    fn from_slice(data: std::borrow::Cow<'a, [u8]>, bits_count: usize) -> Self {
        // Ceil bits_count / 8
        let index_multiplier = bits_count.div_euclid(8) + if bits_count.rem_euclid(8) == 0 { 0 } else { 1 };

        Self {
            data,
            current_index: 0,
            bits_count,
            index_multiplier
        }
    }
}

impl<'a> Iterator for BitsIter<'a> {
    type Item = u8;

    // TODO: override some of the other functions here to more efficiently iterate.

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.current_index / self.bits_count;
        let bit = self.current_index % self.bits_count;

        let byte = self.data.get(index * self.index_multiplier + bit / 8)?;

        let bit_mask = bit % 8;

        let bit = byte & ((1 << bit_mask) as u8) > 0;
        self.current_index += 1;
        Some(bit as u8)
    }
}

pub trait IterBits {
    /// Iterates over all bits in the number.
    fn iter_bits(&self) -> BitsIter;
    /// Iterates over the first `n` bits in each number.
    /// 
    /// Can be used to simulate arbitrarily sized unsigned binary 
    /// numbers.
    fn iter_n_bits(&self, n: usize) -> BitsIter;
}

impl IterBits for Vec<u8> {
    fn iter_bits(&self) -> BitsIter {
        self.iter_n_bits(8)
    }

    fn iter_n_bits(&self, n: usize) -> BitsIter {
        BitsIter::from_slice(self[..].into(), n)
    }
}

impl IterBits for [u8] {
    fn iter_bits(&self) -> BitsIter {
        self.iter_n_bits(8)
    }

    fn iter_n_bits(&self, n: usize) -> BitsIter {
        BitsIter::from_slice(self.into(), n)
    }
}

impl<const N: usize> IterBits for [u8; N] {
    fn iter_bits(&self) -> BitsIter {
        self.iter_n_bits(8)
    }

    fn iter_n_bits(&self, n: usize) -> BitsIter {
        BitsIter::from_slice(self[..].into(), n)
    }
}

impl IterBits for u8 {
    fn iter_bits(&self) -> BitsIter {
        self.iter_n_bits(8)
    }

    fn iter_n_bits(&self, n: usize) -> BitsIter {
        BitsIter::from_slice(vec![*self].into(), n)
    }
}

macro_rules! __impl_iter_bits {
    ($($t:ty => $n:expr);+) => {
        $(
            impl IterBits for Vec<$t> {
                fn iter_bits(&self) -> BitsIter {
                    self.iter_n_bits($n)
                }

                fn iter_n_bits(&self, n: usize) -> BitsIter {
                    let v: Vec<u8> = self.iter().copied().map(|a| a.to_le_bytes().into_iter())
                        .flatten().collect();
                    BitsIter::from_slice(v.into(), n)
                }
            }

            impl IterBits for [$t] {
                fn iter_bits(&self) -> BitsIter {
                    self.iter_n_bits($n)
                }

                fn iter_n_bits(&self, n: usize) -> BitsIter {
                    let v: Vec<u8> = self.iter().copied().map(|a| a.to_le_bytes().into_iter())
                        .flatten().collect();
                    BitsIter::from_slice(v.into(), n)
                }
            }

            impl<const N: usize> IterBits for [$t; N] {
                fn iter_bits(&self) -> BitsIter {
                    self.iter_n_bits($n)
                }

                fn iter_n_bits(&self, n: usize) -> BitsIter {
                    let v: Vec<u8> = self.iter().copied().map(|a| a.to_le_bytes().into_iter())
                        .flatten().collect();
                    BitsIter::from_slice(v.into(), n)
                }
            }

            impl IterBits for $t {
                fn iter_bits(&self) -> BitsIter {
                    self.iter_n_bits($n)
                }

                fn iter_n_bits(&self, n: usize) -> BitsIter {
                    BitsIter::from_slice(self.to_le_bytes().to_vec().into(), n)
                }
            }
        )+
    }
}

__impl_iter_bits!(
    u16 => 16;
    u32 => 32;
    u64 => 64
);

#[test]
fn test_iter_n_bits() {
    // it doesn't like that its read in a macro.
    #![allow(unused_assignments)]

    let vec: Vec<u16> = vec![0b00000_101_0111_0011, 0b00000_011_0101_1000];
    let mut iter = vec.iter_n_bits(11);

    let mut counter = 0;

    macro_rules! assert_values {
        ($($b:expr),+) => {
            $(
                assert_eq!(iter.next(), Some($b), "i = {}", counter);

                counter += 1;
            )+
        }
    }

    assert_values!(
        1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1,
        0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0
    );

    assert_eq!(iter.next(), None);

    let bits: Vec<u8> = vec.iter_n_bits(11).collect();

    let vec2: Vec<u16> = bits.chunks(11).map(|bits| {
        let bits: [u8; 11] = bits.try_into().unwrap();
        let bits = bits.map(|b| b as u16);

        bits.into_iter()
            .fold(
                (0u16, 0u16),
                |(count, value), bit| (count + 1, value | (bit << count))
            ).1
    }).collect();

    assert_eq!(vec, vec2);
}
