use miller_rabin;
use num_bigint::*;
use std::env;
use std::fs::File;
use std::io::{Read, Result};

fn main() -> Result<()> {
    for path in env::args().skip(1) {
        let mut file = File::open(&path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // strip
        while data.len() > 0 && data[data.len() - 1] == b'\n' {
            data.pop();
        }

        println!("checking {}", path);
        let prime = miller_rabin::miller_rabin(
            &BigUint::parse_bytes(&data, 10).expect("invalid prime"),
            1000,
        );
        if prime {
            println!("file {} contains a prime", path);
        } else {
            println!("file {} contains a composite", path);
        }
    }
    Ok(())
}
