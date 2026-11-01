#!/usr/bin/env python3
"""Random check: FOUR fusion entailment equals the knowledge-least model.

Copies the Value semantics of src/experiments/four.rs (tensor and the truth
order used by clauses) and compares, on small random ground programs:

1. brute-force entailment over all 4^n interpretations, as four.rs does;
2. the knowledge-meet of all models (checked to be a model itself);
3. the compiled Horn propagation described in
   docs/semi-naive-pending-optimizations.md, section 12.

Pure Python, no dependencies, bounded to at most three atoms per program.
This is a finite experiment, not a proof, and it does not run Rust code.
"""
import itertools
import random


def tensor(a, b):
    return (a[0] and b[0], ((not b[0]) or a[1]) and ((not a[0]) or b[1]))


def le(a, b):
    return ((not a[0]) or b[0]) and ((not b[1]) or a[1])


def val(lit, world):
    atom, positive = lit
    p, n = world[atom]
    return (p, n) if positive else (n, p)


def evaluate(ant, world):
    if ant == 'I':
        return (True, True)
    if ant[0] == 'lit':
        return val(ant[1], world)
    return tensor(evaluate(ant[1], world), evaluate(ant[2], world))


def is_model(world, facts, clauses):
    return (all(val(f, world)[0] for f in facts)
            and all(le(evaluate(a, world), val(c, world)) for a, c in clauses))


def models(n, facts, clauses):
    values = [(p, q) for p in (False, True) for q in (False, True)]
    for combo in itertools.product(values, repeat=n):
        world = dict(enumerate(combo))
        if is_model(world, facts, clauses):
            yield world


def brute(n, facts, clauses, goal):
    return all(val(goal, w)[0] for w in models(n, facts, clauses))


def var(lit, component):
    atom, positive = lit
    if positive:
        return (atom, component)
    return (atom, 'n' if component == 'p' else 'p')


def p_vars(ant):
    if ant == 'I':
        return set()
    if ant[0] == 'lit':
        return {var(ant[1], 'p')}
    return p_vars(ant[1]) | p_vars(ant[2])


def n_clauses(ant, guard, out):
    if ant == 'I':
        return
    if ant[0] == 'lit':
        out.append((frozenset(guard), var(ant[1], 'n')))
        return
    n_clauses(ant[1], guard | p_vars(ant[2]), out)
    n_clauses(ant[2], guard | p_vars(ant[1]), out)


def compiled(facts, clauses, goal):
    horn = [(frozenset(), var(f, 'p')) for f in facts]
    for ant, cons in clauses:
        horn.append((frozenset(p_vars(ant)), var(cons, 'p')))
        n_clauses(ant, {var(cons, 'n')}, horn)
    on = set()
    changed = True
    while changed:
        changed = False
        for body, head in horn:
            if head not in on and body <= on:
                on.add(head)
                changed = True
    return var(goal, 'p') in on


def random_antecedent(n, depth):
    r = random.random()
    if depth == 0 or r < 0.4:
        return ('lit', (random.randrange(n), random.random() < 0.6))
    if r < 0.47:
        return 'I'
    return ('and', random_antecedent(n, depth - 1), random_antecedent(n, depth - 1))


def main(trials=5000, seed=7):
    random.seed(seed)
    meet_failures = 0
    mismatches = 0
    for _ in range(trials):
        n = random.randint(1, 3)
        facts = [(random.randrange(n), random.random() < 0.7)
                 for _ in range(random.randint(0, 2))]
        clauses = [(random_antecedent(n, 3), (random.randrange(n), random.random() < 0.6))
                   for _ in range(random.randint(0, 4))]
        goal = (random.randrange(n), random.random() < 0.6)
        all_models = list(models(n, facts, clauses))
        meet = {i: (all(m[i][0] for m in all_models), all(m[i][1] for m in all_models))
                for i in range(n)}
        if not is_model(meet, facts, clauses):
            meet_failures += 1
        expected = brute(n, facts, clauses, goal)
        if expected != val(goal, meet)[0] or expected != compiled(facts, clauses, goal):
            mismatches += 1
    print(f'{trials} programs: meet of models not a model: {meet_failures}; '
          f'entailment mismatches: {mismatches}')
    return meet_failures == 0 and mismatches == 0


if __name__ == '__main__':
    raise SystemExit(0 if main() else 1)
