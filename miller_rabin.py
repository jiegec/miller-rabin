#!/usr/bin/env python3

import random
import time

def is_prime(n, k):
    if n <= 1:
        return False
    if n == 2 or n == 3:
        return True
    
    d = n - 1
    r = 0
    while d % 2 == 1:
        d /= 2
        r += 1

    for i in range(0, k):
        a = random.randint(2, n - 2)
        x = pow(a, d, n)
        if x == 1 or x == n - 1:
            continue
        
        prime = False
        for j in range(0, r - 1):
            x = pow(x, 2, n)
            if x == n - 1:
                prime = True
                break
        
        if prime:
            continue
        else:
            return False
    return True

def check(path):
    with open(path) as f:
        print('Checking', path)
        line = int(f.read())
        start_time = time.time()
        assert(is_prime(line, 100))
        end_time = time.time()
        print('Used {} seconds'.format(end_time - start_time))


# small primes
assert(is_prime(5, 1000))
# large primes
assert(is_prime(2**255-19, 1000))
assert(is_prime(2**192-2**64-1, 1000))
assert(is_prime(2**521-1, 1000))
print('Testing passed')

# generated primes
check('prime_1024')
check('prime_2048')
check('prime_4096')
