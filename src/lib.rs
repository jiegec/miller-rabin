#![feature(const_generics)]
#![allow(incomplete_features)]

use num_bigint::*;
use rand;

/// Run Miller-Rabin primality test for `t` rounds
pub fn miller_rabin<T: ToBigUint>(num: &T, t: usize) -> bool {
    let n = num.to_biguint().unwrap();
    let zero: BigUint = 0usize.into();
    let one: BigUint = 1usize.into();
    let two: BigUint = 2usize.into();
    let n1 = &n - &one;

    // trvial cases
    if n <= one {
        return false;
    } else if n == two || n == 3usize.into() {
        return true;
    }

    // find r and d such that n = 2^r*d+1
    let mut r = 0;
    let mut d: BigUint = n.clone() - &one;
    while (&d & &one) == zero {
        d >>= 1;
        r += 1;
    }

    // try t times
    let mut rng = rand::thread_rng();
    for _ in 0..t {
        // pick a random integer a in the range [2, n − 2]
        let a = rng.gen_biguint(n1.bits());
        if a >= two && a < n1 {
            // x ← a^d mod n
            let mut x = a.modpow(&d, &n);
            // if x = 1 or x = n − 1 then
            if x == one || x == n1 {
                //   continue WitnessLoop
                continue;
            }

            let mut prime = false;
            // repeat r − 1 times:
            for _ in 0..r - 1 {
                // x ← x^2 mod n
                x = x.modpow(&two, &n);
                // if x = n − 1 then
                if x == n1 {
                    // continue WitnessLoop
                    prime = true;
                    break;
                }
            }

            if prime {
                continue;
            } else {
                // composite
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    const TIMES: usize = 100;

    #[test]
    fn test_miller_rabin() {
        assert!(miller_rabin(&2, TIMES));
        assert!(miller_rabin(&3, TIMES));
        assert!(miller_rabin(&5, TIMES));
        assert!(miller_rabin(&7, TIMES));

        assert!(!miller_rabin(&4, TIMES));
    }
}