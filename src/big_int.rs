use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Shl, Shr, Sub};

type Unit = u64;
type DoubleUnit = u128;
const BITS: usize = std::mem::size_of::<Unit>() * 8;

#[derive(Clone)]
pub struct BigUInt<const WORDS: usize> {
    // from low to high
    num: [Unit; WORDS],
}

impl<const WORDS: usize> BigUInt<WORDS> {
    pub fn zero() -> BigUInt<WORDS> {
        BigUInt { num: [0; WORDS] }
    }

    pub fn one() -> BigUInt<WORDS> {
        let mut res = BigUInt { num: [0; WORDS] };
        res.num[0] = 1;
        res
    }

    pub fn two() -> BigUInt<WORDS> {
        let mut res = BigUInt { num: [0; WORDS] };
        res.num[0] = 2;
        res
    }

    pub fn three() -> BigUInt<WORDS> {
        let mut res = BigUInt { num: [0; WORDS] };
        res.num[0] = 3;
        res
    }

    pub fn is_zero(&self) -> bool {
        for i in 0..WORDS {
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
        let mut res = Self { num: [0; WORDS] };
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

    pub fn monty_mul(&self, other: &Self, n: &Self, mc: Unit) -> Self {
        let mut res = [0; WORDS];
        let mut res1 = 0;
        let mut res2;
        for i in 0..WORDS {
            let mut c = 0;
            for j in 0..WORDS {
                let mut cs = res[j] as DoubleUnit;
                cs += self.num[j] as DoubleUnit * other.num[i] as DoubleUnit;
                cs += c as DoubleUnit;
                c = (cs >> BITS) as Unit;
                res[j] = cs as Unit;
            }
            let cs = res1 as DoubleUnit + c as DoubleUnit;
            res1 = cs as Unit;
            res2 = (cs >> BITS) as Unit;
            let m = (res[0] as DoubleUnit * mc as DoubleUnit) as Unit;
            let mut cs = res[0] as DoubleUnit + m as DoubleUnit * n.num[0] as DoubleUnit;
            c = (cs >> BITS) as Unit;
            for j in 1..WORDS {
                cs = res[j] as DoubleUnit;
                cs += m as DoubleUnit * n.num[j] as DoubleUnit;
                cs += c as DoubleUnit;
                c = (cs >> BITS) as Unit;
                res[j - 1] = cs as Unit;
            }
            cs = res1 as DoubleUnit + c as DoubleUnit;
            res[WORDS - 1] = cs as Unit;
            res1 = res2 + (cs >> BITS) as Unit;
        }
        let mut ret = Self { num: res };
        while ret >= *n {
            ret = ret - n;
        }
        ret
    }

    /// R = 2^BITS, calculate R^2 mod n
    pub fn r2n(&self) -> Self {
        let mc = self.calc_montgomery_constant();

        // init to R-1
        let mut m = Self { num: [0; WORDS] };
        for i in 0..WORDS {
            m.num[i] = Unit::MAX;
        }

        m
    }

    pub fn pow_mod(&self, pow: &Self, n: &Self) -> Self {
        let mut res = self.clone();
        res
    }
}

impl<const WORDS: usize> From<Unit> for BigUInt<WORDS> {
    fn from(num: Unit) -> Self {
        let mut res = BigUInt { num: [0; WORDS] };
        res.num[0] = num;
        res
    }
}

impl<const WORDS: usize> Add<Unit> for BigUInt<WORDS> {
    type Output = BigUInt<WORDS>;

    fn add(mut self, other: Unit) -> Self {
        let (new, overflow) = self.num[0].overflowing_add(other);
        if !overflow {
            // fast path
            self.num[0] = new;
        } else {
            for i in 1..WORDS {
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

impl<const WORDS: usize> Add<BigUInt<WORDS>> for BigUInt<WORDS> {
    type Output = BigUInt<WORDS>;

    fn add(mut self, other: BigUInt<WORDS>) -> Self {
        let mut carry = 0;
        for i in 0..WORDS {
            let (new, overflow) = self.num[i].overflowing_add(other.num[i]);
            let (new, overflow2) = new.overflowing_add(carry);
            self.num[i] = new;
            carry = (overflow || overflow2) as Unit;
        }
        self
    }
}

impl<const WORDS: usize> Sub<BigUInt<WORDS>> for BigUInt<WORDS> {
    type Output = BigUInt<WORDS>;

    fn sub(self, other: BigUInt<WORDS>) -> Self {
        self - &other
    }
}

impl<const WORDS: usize> Sub<&BigUInt<WORDS>> for BigUInt<WORDS> {
    type Output = BigUInt<WORDS>;

    fn sub(mut self, other: &BigUInt<WORDS>) -> Self {
        let mut borrow = 0;
        for i in 0..WORDS {
            let (new, underflow) = self.num[i].overflowing_sub(other.num[i]);
            let (new, underflow2) = new.overflowing_sub(borrow);
            self.num[i] = new;
            borrow = (underflow || underflow2) as Unit;
        }
        self
    }
}

impl<const WORDS: usize> Shl<usize> for BigUInt<WORDS> {
    type Output = BigUInt<WORDS>;

    fn shl(mut self, other: usize) -> Self {
        // only sub word shifting is supported yet
        assert!(other < BITS);
        if other != 0 {
            self.num[WORDS - 1] <<= other;
            for i in (0..WORDS - 1).rev() {
                self.num[i + 1] |= self.num[i] >> (BITS - other);
                self.num[i] <<= other;
            }
        }
        self
    }
}

impl<const WORDS: usize> Shr<usize> for BigUInt<WORDS> {
    type Output = BigUInt<WORDS>;

    fn shr(mut self, other: usize) -> Self {
        // only sub word shifting is supported yet
        assert!(other < BITS);
        if other != 0 {
            self.num[0] >>= other;
            for i in 1..WORDS {
                self.num[i - 1] |= self.num[i] << (BITS - other);
                self.num[i] >>= other;
            }
        }
        self
    }
}

impl<const WORDS: usize> Eq for BigUInt<WORDS> {}

impl<const WORDS: usize> PartialEq for BigUInt<WORDS> {
    fn eq(&self, other: &Self) -> bool {
        for i in 0..WORDS {
            if self.num[i] != other.num[i] {
                return false;
            }
        }
        true
    }
}

impl<const WORDS: usize> Ord for BigUInt<WORDS> {
    fn cmp(&self, other: &Self) -> Ordering {
        for i in (0..WORDS).rev() {
            if self.num[i] > other.num[i] {
                return Ordering::Greater;
            } else if self.num[i] < other.num[i] {
                return Ordering::Less;
            }
        }
        Ordering::Equal
    }
}

impl<const WORDS: usize> PartialOrd for BigUInt<WORDS> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<const WORDS: usize> fmt::Debug for BigUInt<WORDS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let octets = BITS / 4;
        write!(f, "0x")?;
        for i in (0..WORDS).rev() {
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
    const C: usize = 2;
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
