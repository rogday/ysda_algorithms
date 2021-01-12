# bad_rand

This program solves the following problem:

Program called srand(SEED) and after given i iterations rand() produces sequence of n numbers(mod p in this case, 
but doesn't matter). The task is to guess SEED. 

### Solution

glibc's rand() is implemented as linear-feedback shift register with state size of 31. So, it is possible to express
the state after i iterations by multiplying initial state by matrix of coefficients. First, we find such matrix by using
the repeated custom_rand() calls with vectors of coefficients instead of actual numbers. Then it's just parallelized
brute-force, which covers u32 range in about 1.5 minutes on my machine.