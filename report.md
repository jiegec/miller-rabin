# Miller-Rabin 算法实现

分别用 Rust 和 Python 实现了 Miller-Rabin 算法，并且使用 openssl 生成了满足要求的素数，并构造了两个素数的乘积作为合数的测例。

使用 openssl 生成了不同位数的素数，分别在 `prime_128` `prime_1024` `prime_2048` 和 `prime_4096` 文件里。生成的脚本见 `gen_prime.sh` 。它保证素数最高位和最低位都是 1 。

并且额外生成了两个 64 位素数，把乘积放在 `composite_128` 文件中。

## Rust 版本

开发时采用的是 nightly 版本的 Rust 。

核心代码在 `src/lib.rs` 中。通过 `cargo test` 运行内置的测试，或者运行 `cargo --release -- prime_1024` 对特定文件的数值进行判断，比如：

```shell
$ cargo run --release -- prime_128
    Finished release [optimized] target(s) in 0.03s
     Running `target/release/miller-rabin prime_128`
checking prime_128
file prime_128 contains a prime
$ cargo run --release -- composite_128
    Finished release [optimized] target(s) in 0.03s
     Running `target/release/miller-rabin composite_128`
checking composite_128
file composite_128 contains a composite
```

其它文件亦可进行类似地运行。

## Python 版本

采用 Python 3.7.7 进行开发。

代码在 `miller_rabin.py` 中，进行了一些简单的测试，并且对几个生成的大测例进行了测试：

```shell
$ python3 miller_rabin.py
Testing passed
Checking composite_128
Used 0.0001709461212158203 seconds
Checking prime_128
Used 0.009345054626464844 seconds
Checking prime_1024
Used 0.5056991577148438 seconds
Checking prime_2048
Used 2.8593337535858154 seconds
Checking prime_4096
Used 20.936081171035767 seconds
```