use std::cmp::Ordering;
use std::fmt;
use std::ops::Add;

type Unit = u64;

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
        let octets = std::mem::size_of::<Unit>() * 8 / 4;
        write!(f, "0x")?;
        for i in (0..COUNT).rev() {
            let s = format!("{:X}", self.num[i]);
            let pad = octets - s.len();
            write!(f, "{}{}", String::from("0").repeat(pad), s)?
        }
        Ok(())
    }
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
        assert_eq!(BigUInt::<C>::zero(), BigUInt::<C>::one());
    }
}
