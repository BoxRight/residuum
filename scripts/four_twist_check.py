#!/usr/bin/env python3
"""Exhaustive check: the FOUR Value of src/experiments/four.rs is the twist
structure over the two-element Boolean algebra, and it is residuated.

Copies Value::tensor and Value::le from four.rs and checks, over all values:
the tensor equals the twist product (a, b)*(c, d) = (a*c, (a->d) & (c->b));
it is associative and commutative with unit I = (1, 1); and the twist
implication (b, c) |-> ((b1->c1) & (c2->b2), b1 & c2) satisfies
a*b <= c  <=>  a <= b -> c in the truth order. Pure Python, 64 cases.
"""
import itertools

VALUES = [(p, n) for p in (False, True) for n in (False, True)]


def imp(a, b):
    return (not a) or b


def tensor(a, b):  # Value::tensor in four.rs
    return (a[0] and b[0], ((not b[0]) or a[1]) and ((not a[0]) or b[1]))


def le(a, b):  # Value::le in four.rs (truth order)
    return ((not a[0]) or b[0]) and ((not b[1]) or a[1])


def twist_product(a, b):
    return (a[0] and b[0], imp(a[0], b[1]) and imp(b[0], a[1]))


def twist_residual(b, c):
    return (imp(b[0], c[0]) and imp(c[1], b[1]), b[0] and c[1])


def main():
    unit = (True, True)
    checks = {
        'tensor is the twist product': all(
            tensor(a, b) == twist_product(a, b) for a in VALUES for b in VALUES),
        'commutative': all(tensor(a, b) == tensor(b, a) for a in VALUES for b in VALUES),
        'associative': all(
            tensor(tensor(a, b), c) == tensor(a, tensor(b, c))
            for a, b, c in itertools.product(VALUES, repeat=3)),
        'unit I = (1, 1)': all(tensor(a, unit) == a for a in VALUES),
        'residuation': all(
            le(tensor(a, b), c) == le(a, twist_residual(b, c))
            for a, b, c in itertools.product(VALUES, repeat=3)),
    }
    for name, ok in checks.items():
        print(f'{name}: {ok}')
    return all(checks.values())


if __name__ == '__main__':
    raise SystemExit(0 if main() else 1)
