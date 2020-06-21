# Approximations of Mathematical Functions

## Generating Coefficients

The coefficients should be _checked in to the repository._ There is really no guarantee that we would get the same results on different systems or with different versions of NumPy.

Coefficients can be generated with the Python program `calc.py`. This will create coefficient files CSV files.

## Goals

- Functions should be fast

- Implementations should be simple

- Inputs and outputs are arrays

- Adjustable speed / complexity tradeoff

- Accurate results when inputs and outputs are finite and within a limited range (up to 1e6, which should be the largest number we need for audio)

## Functions

### exp2

We create a polynomial approximation to `2^x`, for x in the range -0.5..+0.5 using the Remez exchange algorithm, adjusted to minimize relative error.

### sin1

Polynomial approximations to `sin(2 pi x)` for x in the range 0..0.25, or -0.25..0.25, depending on the version.

- sin1_smooth: Smooth approximation over -0.25..0.25. Higher orders have higher-order continuous derivatives. The function is odd, and only odd polynomial coefficients are included.

- sin1_l1: Fix f(0) = 0 and minimize L1 error on 0..0.25 with Remez exchange algorithm.
