#![feature(const_generics)]
#![allow(incomplete_features)]

pub mod big_int;

pub fn miller_rabin<const COUNT: usize>(num: &big_int::BigUInt<COUNT>, t: usize) -> bool {
    // trvial cases
    if num < &big_int::BigUInt::<COUNT>::two() {
        return false;
    } else if num == &big_int::BigUInt::<COUNT>::two() || num == &big_int::BigUInt::<COUNT>::three()
    {
        return false;
    }

    // find r and d such that n = 2^r*d+1
    let mut d = num.clone() - big_int::BigUInt::<COUNT>::one();
    let mut r = 0;
    while d.low_bit() {
        r += 1;
        d = d >> 1;
    }
    true
}
