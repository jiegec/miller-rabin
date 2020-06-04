use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Shl, Shr, Sub};

type Unit = u64;
const BITS: usize = std::mem::size_of::<Unit>() * 8;

#[derive(Clone)]
pub struct BigUInt<const COUNT: usize> {
    // from low to high
    num: [Unit; COUNT],
}

impl<const COUNT: usize> BigUInt<COUNT> {
    pub fn zero() -> BigUInt<COUNT> {
        BigUInt { num: [0; COUNT] }
    }

    pub fn one() -> BigUInt<COUNT> {
        let mut res = BigUInt { num: [0; COUNT] };
        res.num[0] = 1;
        res
    }

    pub fn two() -> BigUInt<COUNT> {
        let mut res = BigUInt { num: [0; COUNT] };
        res.num[0] = 2;
        res
    }

    pub fn three() -> BigUInt<COUNT> {
        let mut res = BigUInt { num: [0; COUNT] };
        res.num[0] = 3;
        res
    }

    pub fn is_zero(&self) -> bool {
        for i in 0..COUNT {
            if self.num[i] != 0 {
                return false;
            }
        }
        true
    }

    pub fn low_bit(&self) -> bool {
        (self.num[0] & 1) == 1
    }

    pub fn from_hex_str(input: &str) -> Self {
        let mut res = Self { num: [0; COUNT] };
        for (i, ch) in input.bytes().rev().enumerate() {
            let num = if b'0' <= ch && ch <= b'9' {
                ch - b'0'
            } else if b'a' <= ch && ch <= b'z' {
                ch - b'a'
            } else if b'A' <= ch && ch <= b'Z' {
                ch - b'Z'
            } else {
                0
            } as Unit;
            res.num[i / (BITS / 4)] |= num << ((i % (BITS / 4)) * 4);
        }
        res
    }

    pub fn calc_montgomery_constant(&self) -> Unit {
        // -n[0]^{-1} mod w
        inverse_pow2_bits(self.num[0])
    }

    pub fn pow_mod(&self, pow: &Self, n: &Self) -> Self {
        let mut res = self.clone();
        res
    }
}

impl<const COUNT: usize> From<Unit> for BigUInt<COUNT> {
    fn from(num: Unit) -> Self {
        let mut res = BigUInt { num: [0; COUNT] };
        res.num[0] = num;
        res
    }
}

impl<const COUNT: usize> Add<Unit> for BigUInt<COUNT> {
    type Output = BigUInt<COUNT>;

    fn add(mut self, other: Unit) -> Self {
        let (new, overflow) = self.num[0].overflowing_add(other);
        if !overflow {
            // fast path
            self.num[0] = new;
        } else {
            for i in 1..COUNT {
                let (new, overflow) = self.num[i].overflowing_add(1);
                self.num[i] = new;
                if !overflow {
                    break;
                }
            }
        }
        self
    }
}

impl<const COUNT: usize> Add<BigUInt<COUNT>> for BigUInt<COUNT> {
    type Output = BigUInt<COUNT>;

    fn add(mut self, other: BigUInt<COUNT>) -> Self {
        let mut carry = 0;
        for i in 0..COUNT {
            let (new, overflow) = self.num[i].overflowing_add(other.num[i]);
            let (new, overflow2) = new.overflowing_add(carry);
            self.num[i] = new;
            carry = (overflow || overflow2) as Unit;
        }
        self
    }
}

impl<const COUNT: usize> Sub<BigUInt<COUNT>> for BigUInt<COUNT> {
    type Output = BigUInt<COUNT>;

    fn sub(mut self, other: BigUInt<COUNT>) -> Self {
        let mut borrow = 0;
        for i in 0..COUNT {
            let (new, underflow) = self.num[i].overflowing_sub(other.num[i]);
            let (new, underflow2) = new.overflowing_sub(borrow);
            self.num[i] = new;
            borrow = (underflow || underflow2) as Unit;
        }
        self
    }
}

impl<const COUNT: usize> Shl<usize> for BigUInt<COUNT> {
    type Output = BigUInt<COUNT>;

    fn shl(mut self, other: usize) -> Self {
        // only sub word shifting is supported yet
        assert!(other < BITS);
        if other != 0 {
            self.num[COUNT - 1] <<= other;
            for i in (0..COUNT - 1).rev() {
                self.num[i + 1] |= self.num[i] >> (BITS - other);
                self.num[i] <<= other;
            }
        }
        self
    }
}

impl<const COUNT: usize> Shr<usize> for BigUInt<COUNT> {
    type Output = BigUInt<COUNT>;

    fn shr(mut self, other: usize) -> Self {
        // only sub word shifting is supported yet
        assert!(other < BITS);
        if other != 0 {
            self.num[0] >>= other;
            for i in 1..COUNT {
                self.num[i - 1] |= self.num[i] << (BITS - other);
                self.num[i] >>= other;
            }
        }
        self
    }
}

impl<const COUNT: usize> Eq for BigUInt<COUNT> {}

impl<const COUNT: usize> PartialEq for BigUInt<COUNT> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..COUNT {
            if self.num[i] != other.num[i] {
                return false;
            }
        }
        true
    }
}

impl<const COUNT: usize> Ord for BigUInt<COUNT> {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..COUNT).rev() {
            if self.num[i] > other.num[i] {
                return Ordering::Greater;
            } else if self.num[i] < other.num[i] {
                return Ordering::Less;
            }
        }
        Ordering::Equal
    }
}

impl<const COUNT: usize> PartialOrd for BigUInt<COUNT> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const COUNT: usize> fmt::Debug for BigUInt<COUNT> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octets = BITS / 4;
        write!(f, "0x")?;
        for i in (0..COUNT).rev() {
            let s = format!("{:X}", self.num[i]);
            let pad = octets - s.len();
            write!(f, "{}{}", String::from("0").repeat(pad), s)?
        }
        Ok(())
    }
}

/// ax + by = gcd(a, b)
/// returns (x, y)
fn extended_gcd(a: Unit, b: Unit) -> (Unit, Unit) {
    if a == 0 as Unit {
        return (0, 1);
    }
    let (x1, y1) = extended_gcd(b % a, a);
    let x = y1.wrapping_sub((b / a).wrapping_mul(x1));
    let y = x1;
    (x, y)
}

/// -num^{-1} mod 2^BITS
fn inverse_pow2_bits(num: Unit) -> Unit {
    // a = num, b = 2^BITS
    // b % a = (-a) % a
    let (x1, y1) = extended_gcd((!num + 1) % num, num);
    // inverse
    let x = y1.wrapping_sub((1 + (!num + 1) / num) * x1);
    return !x + 1;
}

#[cfg(test)]
mod tests {
    const C: usize = 4;
    use super::*;

    #[test]
    fn equals() {
        assert_eq!(BigUInt::<C>::zero(), BigUInt::<C>::zero());
        assert_eq!(BigUInt::<C>::one(), BigUInt::<C>::one());
        assert_ne!(BigUInt::<C>::zero(), BigUInt::<C>::one());
        assert_ne!(BigUInt::<C>::zero(), BigUInt::<C>::one());
    }

    #[test]
    fn add() {
        assert_eq!(BigUInt::<C>::zero() + 1, BigUInt::<C>::one());
        assert_eq!(BigUInt::<C>::zero() + 2, BigUInt::<C>::two());
        assert_eq!(BigUInt::<C>::one() + 0, BigUInt::<C>::one());
        assert_eq!(BigUInt::<C>::one() + 1, BigUInt::<C>::two());
        assert_eq!(
            BigUInt::<C>::one() + BigUInt::<C>::one(),
            BigUInt::<C>::two()
        );
    }

    #[test]
    fn shl() {
        assert_eq!(BigUInt::<C>::zero() << 1, BigUInt::<C>::zero());
        assert_eq!(BigUInt::<C>::zero() << 2, BigUInt::<C>::zero());
        assert_eq!(BigUInt::<C>::one() << 0, BigUInt::<C>::one());
        assert_eq!(BigUInt::<C>::one() << 1, BigUInt::<C>::two());
        assert_eq!(BigUInt::<C>::one() << 3, BigUInt::<C>::two() << 2);
    }

    #[test]
    fn shr() {
        assert_eq!(BigUInt::<C>::zero() >> 1, BigUInt::<C>::zero());
        assert_eq!(BigUInt::<C>::zero() >> 2, BigUInt::<C>::zero());
        assert_eq!(BigUInt::<C>::one() >> 0, BigUInt::<C>::one());
        assert_eq!(BigUInt::<C>::one() >> 1, BigUInt::<C>::zero());
        assert_eq!(BigUInt::<C>::two() >> 1, BigUInt::<C>::one());
    }

    #[test]
    fn from_hex_str() {
        assert_eq!(BigUInt::<C>::from_hex_str("0"), BigUInt::<C>::zero());
        assert_eq!(BigUInt::<C>::from_hex_str("1"), BigUInt::<C>::one());
        assert_eq!(BigUInt::<C>::from_hex_str("2"), BigUInt::<C>::two());
        assert_eq!(BigUInt::<C>::from_hex_str("10"), BigUInt::<C>::one() << 4);
        assert_eq!(BigUInt::<C>::from_hex_str("100"), BigUInt::<C>::one() << 8);
    }

    #[test]
    fn test_inverse_pow2_bits() {
        // 2 ** BITS - gmpy2.invert(n, 2 ** BITS)
        assert_eq!(inverse_pow2_bits(0x3a79c436_46842eff), 0x3857b70d_56252f01);
        // 25519
        assert_eq!(inverse_pow2_bits(0xffffffff_ffffffed), 0x86bca1af_286bca1b);
    }
}
