""""Benchmark driver."""
import argparse
import csv
import dataclasses
import numpy
import pathlib
import subprocess
import sys

from typing import Optional

def die(*msg):
    print('Error:', *msg, file=sys.stderr)
    raise SystemExit(1)

def read_csv(path):
    ops = {}
    with path.open() as fp:
        r = csv.reader(fp)
        row = next(r)
        if row is None:
            die('File {!r} empty'.format(str(path)))
        expect = ["Operator", "TimeNS"]
        if row != expect:
            die('File {!r} has columns {!r}, expected {!r}'
                .format(str(path), row, expect))
        for lineno, row in enumerate(r, 2):
            try:
                opname, timens = row
                timens = float(timens)
            except ValueError:
                die('File {!r}: could not parse data on line {}'
                    .format(str(path), lineno))
            optimes = ops.get(opname)
            if optimes is None:
                ops[opname] = [timens]
            else:
                optimes.append(timens)
    for opname, optimes in ops.items():
        ops[opname] = numpy.array(optimes, numpy.float64)
    return ops

@dataclasses.dataclass
class Stats:
    median: float
    variation: Optional[float]

def stats(times):
    median = numpy.median(times).item()
    variation = None
    if len(times) > 1:
        variation = numpy.mean(numpy.abs(times - median)).item()
    return Stats(median, variation)

def compare(refdata, newdata):
    for opname, times in newdata.items():
        reftimes = refdata.get(opname)
        if reftimes is None:
            continue
        print('Operator {}'.format(opname))
        rst = stats(reftimes)
        nst = stats(times)
        print('    Median:    {:5.3f} -> {:5.3f} ns/sample'
              .format(rst.median, nst.median))
        if rst.variation is not None or nst.variation is not None:
            rv = '-----' if rst.variation is None else format(rst.variation, '.3f')
            nv = '-----' if nst.variation is None else format(nst.variation, '.3f')
            print('    Variation: {:5} -> {:5} ns/sample'
                  .format(rv, nv))
        change = 100 * (nst.median - rst.median) / rst.median
        color = ''
        if change > 2:
            color = '31'
        elif change < -2:
            color = '32'
        set_color = ''
        reset_color = ''
        if color:
            set_color = '\x1b[{}m'.format(color)
            reset_color = '\x1b[0m'
        print('    Change: {}{:+.2f}%{}'.format(set_color, change, reset_color))

        print()

def show(data):
    for opname, times in data.items():
        print('Operator {}'.format(opname))
        st = stats(times)
        print('    Median:    {:.3f}ns/sample'.format(st.median))
        if st.variation is not None:
            print('    Variation: {:.3f}ns/sample'.format(st.variation))

def main(argv):
    p = argparse.ArgumentParser('bench.py')
    p.add_argument('function', default=[], nargs='*',
                   help='Functions to benchmark')
    p.add_argument('--save', help='Save results as reference',
                   action='store_true')
    p.add_argument('--compare', help='Compare results to reference',
                   action='store_true')
    p.add_argument('--runs', type=int, help='Number of benchmark runs')
    p.add_argument('--size', type=int, help='Size of array')
    p.add_argument('--iter', type=int, help='Number of iterations per run')
    p.add_argument('--impl', choices={'vector', 'scalar'},
                   default='vector', help='Operator implementation')
    args = p.parse_args(argv)

    bench_args = []
    if args.runs is not None:
        bench_args.append('-runs={}'.format(args.runs))
    if args.iter is not None:
        bench_args.append('-iter={}'.format(args.iter))
    if args.size is not None:
        bench_args.append('-size={}'.format(args.size))

    here = pathlib.Path(__file__).parent
    ref = here / 'bench_ref.csv'
    out = here / 'bench_out.csv'

    bazel_args = []
    if args.impl != 'vector':
        bazel_args.append('--define=ops=' + args.impl)

    proc = subprocess.run(
        ['bazel', 'build', '-c', 'opt', ':oprun', *bazel_args],
        cwd=here,
        stdin=subprocess.DEVNULL,
    )
    if proc.returncode:
        die('Build failed')
    print(file=sys.stderr)
    print('Running benchmarks', file=sys.stderr)
    exe = here / '../../bazel-bin/c/ops/oprun'

    proc = subprocess.run(
        [exe, 'benchmark', *bench_args, '-out=bench_out.csv',
         '--', *args.function],
        cwd=here,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
    )
    if proc.returncode:
        die('Benchmark failed')

    out_data = read_csv(out)
    if args.compare:
        ref_data = read_csv(ref)
        compare(ref_data, out_data)
    else:
        show(out_data)
    if args.save:
        out.replace(ref)

if __name__ == '__main__':
    main(sys.argv[1:])
