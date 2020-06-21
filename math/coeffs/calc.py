import argparse
import math
import numpy
import numpy.polynomial.polynomial as polynomial
import pathlib
import sys

from typing import List, TextIO, Tuple

class SolverError(Exception):
    pass

def chebyshev_nodes(n: int) -> numpy.ndarray:
    """Generate N Chebyshev nodes in the range [-1,1]."""
    d = math.pi * 0.5
    return numpy.sin(numpy.linspace(-d, d, 2 * n + 1)[1::2])

def rescale(x, xrange):
    """Rescale the x array so it covers xrange exactly."""
    x0, x1 = xrange
    xmin = numpy.min(x)
    xmax = numpy.max(x)
    xspan = xmax - xmin
    return (x - xmin) * (x1 / xspan) + (xmax - x) * (x0 / xspan)

FUNCTIONS = {}
def function(*, name, min_order):
    def wrap(f):
        assert name not in FUNCTIONS
        FUNCTIONS[name] = (f, min_order)
        return f
    return wrap

@function(name='exp2', min_order=2)
def exp2_coeffs(order: int) -> numpy.ndarray:
    """Coefficients for 2^x on (-0.5, 0.5).
    
    Coefficients are chosen to minimize maximum equivalent input error.
    """
    xrange = -0.5, 0.5
    x0, x1 = xrange
    # Remez algorithm, adapted to minimize relative error.
    # Signs: alternating +1, -1
    signs = numpy.zeros((order + 2,))
    signs[0::2] = 1
    signs[1::2] = -1
    # X: initial set of sample points
    # Chebyshev nodes, to avoid Runge's phenomenon
    x = chebyshev_nodes(order + 2)
    x = rescale(x, xrange)
    # x = 0.5 * x + 0.5 * (numpy.power(2, x) - 1)

    y = numpy.exp2(x)
    last_rel_err = math.inf
    last_poly_coeffs = None
    for i in range(100):
        # Solve equation: a_j * x_i^j + (-1)^i * E * 2^x_i = 2^x_i
        # This gives us coeffs for polynomial, followed by E
        error_coeffs = signs * y
        lin_coeffs = numpy.append(
            numpy.power(x[:, None], numpy.arange(0, order + 1)[None, :]),
            error_coeffs.reshape((order + 2, 1)),
            axis=1,
        )
        poly_coeffs = numpy.linalg.solve(lin_coeffs, y)
        # The [:-1] strips off E, and gives us just the coeffs.
        poly_coeffs = poly_coeffs[:-1]

        # Find extrema of (p(x) - 2^x) / 2^x
        # Which are extrema of p(x) 2^-x - 1
        # Which we find by solving p'(x) 2^-x - log 2 2^-x p(x) = 0
        # Which has the same roots as log2 p(x) - p'(x)
        rel_coeffs = numpy.log(2) * poly_coeffs
        rel_coeffs[:-1] -= numpy.arange(1, order + 1) * poly_coeffs[1:]
        roots = numpy.roots(rel_coeffs[::-1])
        if numpy.any(numpy.iscomplex(roots)):
            raise SolverError('Roots are complex')
        roots.sort()
        if numpy.min(roots) <= x0 or x1 <= numpy.max(roots):
            raise SolverError('Roots are too large')
        x[0] = x0
        x[1 : -1] = roots
        x[-1] = x1

        # Calculate maximum relative error
        y = numpy.exp2(x)
        rel_err = numpy.max(
            numpy.abs((polynomial.Polynomial(poly_coeffs)(x) - y) / y))
        if not math.isinf(last_rel_err):
            improvement = (last_rel_err - rel_err) / last_rel_err
            if improvement <= 0:
                rel_err, poly_coeffs = last_rel_err, last_poly_coeffs
                break
            elif improvement < 1e-6:
                break
        last_rel_err = rel_err
        last_poly_coeffs = poly_coeffs

    return poly_coeffs

@function(name='sin1_smooth', min_order=1)
def sin1_smooth_coeffs(order: int) -> numpy.ndarray:
    """Coefficients for sin(2 pi x) on (-0.25, 0.25).

    Coefficients are chosen to make higher order derivatives smooth. Only
    odd-numbered coefficients are included.
    """
    # We solve for an odd polynomial p(x) where the odd derivatives are 0 at
    # x=1, and then fix p(1) = 1.
    mat_coeffs = numpy.zeros((order, order))
    vec_coeffs = numpy.zeros((order))
    poly = numpy.ones((order,))
    powers = numpy.arange(order) * 2 + 1
    for n in range(order - 1):
        poly *= powers
        powers -= 1
        # 2n+1-th derivative of f is 0 at x=1
        mat_coeffs[n] = poly
        poly *= powers
        powers -= 1
    # f(1) = 1
    mat_coeffs[order - 1] = 1
    vec_coeffs[order - 1] = 1
    poly_coeffs = numpy.linalg.solve(mat_coeffs, vec_coeffs)
    # Above coefficients are for sin(pi x / 2), rescale for sin(2 pi x).
    poly_coeffs *= 4 ** numpy.arange(1, 2 * order + 1, 2)
    return poly_coeffs

@function(name='sin1_l1', min_order=2)
def sin1_l1_coeffs(order: int) -> numpy.ndarray:
    """Coefficients for sin(2 pi x) on (0, 0.25).

    Constant coefficient is chosen to be zero, and omitted from result. Maximum
    error is minimized.
    """
    # We create for a polynomial for a quarter wave of a sine.
    # Remez algorithm.
    # Signs: alternating +1, -1
    signs = numpy.zeros((order + 2,))
    signs[0::2] = 1
    signs[1::2] = -1
    # Fix f(0) = 0
    signs[0] = 0
    # X: initial set of sample points
    # Chebyshev nodes, to avoid Runge's phenomenon
    x = chebyshev_nodes(order + 2)
    x = rescale(x, (0.0, 0.25))

    tau = 2 * numpy.pi
    last_error = math.inf
    last_poly_coeffs = None
    for _ in range(100):
        # Solve for polynomial coefficients.
        lin_coeffs = numpy.append(
            numpy.power(x[:, None], numpy.arange(0, order + 1)[None, :]),
            signs[:,None],
            axis=1,
        )
        poly_coeffs = numpy.linalg.solve(lin_coeffs, numpy.sin(tau * x))[:-1]
        poly_coeffs[0] = 0 # Should be 0 anyway.

        # Find X values of maximum error.
        # Which are solutions to d/dx p(x) - 2 pi cos(2 pi x) = 0
        extrema = x[1:-1].copy()
        dpoly_coeffs = poly_coeffs[1:] * numpy.arange(1, order + 1)
        ddpoly_coeffs = dpoly_coeffs[1:] * numpy.arange(1, order)
        for _ in range(10):
            # Newton's method: x <- x - f(x) / f'(x)
            powers = numpy.power(extrema[:, None],
                                 numpy.arange(0, order)[None, :])
            fx = powers @ dpoly_coeffs - tau * numpy.cos(tau * extrema)
            dfx = (powers[:,:-1] @ ddpoly_coeffs
                   + tau * tau * numpy.sin(tau * extrema))
            deltax = fx / dfx
            extrema -= deltax
            maxdelta = numpy.max(numpy.abs(deltax))
            if maxdelta < 1e-10:
                break
        # Use these x values for next iteration.
        x[1:-1] = extrema
        if not numpy.all(x[:-1] < x[1:]):
            raise SolverError('extrema not ascending')

        # Calculate maximum error
        powers = numpy.power(extrema[:, None],
                             numpy.arange(0, order + 1)[None, :])
        error = numpy.max(numpy.abs(
            powers @ poly_coeffs - numpy.sin(tau * extrema)))
        if not math.isinf(last_error):
            improvement = (last_error - error) / last_error
            if improvement <= 0:
                error, poly_coeffs = last_error, last_poly_coeffs
                break
            elif improvement < 1e-6:
                break
        last_error = error
        last_poly_coeffs = poly_coeffs

    return poly_coeffs[1:]

def write_data(data: List[Tuple[int, numpy.ndarray]], fp: TextIO) -> None:
    for n, coeffs in data:
        cells = [str(n)]
        for coeff in coeffs:
            cells.append(str(coeff))
        fp.write(','.join(cells) + '\n')

def main(argv: List[str]) -> None:
    p = argparse.ArgumentParser('calc.py')
    p.add_argument('-o', '--output')
    p.add_argument('-n', '--order', type=int, default=8)
    p.add_argument('function', choices=FUNCTIONS)
    args = p.parse_args(argv)

    function, min_order = FUNCTIONS[args.function]
    data = [(n, function(n)) for n in range(min_order, args.order + 1)]

    output = args.output
    if output is None:
        output = (pathlib.Path(__file__).resolve().parent
                  / (args.function + '.csv'))
        print('Writing', output, file=sys.stderr)
        with output.open('w') as fp:
            write_data(data, fp)
    elif output == '-':
        write_data(data, sys.stdout)
    else:
        print('Writing', output, file=sys.stderr)
        with open(output, 'w') as fp:
            write_data(data, fp)

if __name__ == '__main__':
    main(sys.argv[1:])
