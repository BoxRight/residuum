#!/usr/bin/env python3
"""Render bounded test exports. Does not parse law, compute closure or run tests.

Run the seven individual legal_geometry tests with test_safe.py first.
The three extremes tests additionally export intrinsic bases of closed states.
The two frontiers tests export goal-support and provenance certificates.
The two residual_comparison tests diagnose the existing query's supported fragment.
The query_dependencies tests distinguish structural relevance from activation.
All inference data comes from the elaborated AST and the real close_program.
"""
import ast
import argparse
import collections
import csv
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parent.parent
RAW = ROOT / 'target/legal-geometry'
OUT = ROOT / 'docs/legal-geometry'


def rows(name):
    with (RAW / name).open() as stream:
        return list(csv.DictReader(stream, delimiter='\t'))


def components(nodes, edges, directed):
    adjacency = {n: set() for n in nodes}
    for a, b in edges:
        adjacency[a].add(b)
        if not directed:
            adjacency[b].add(a)
    def reachable(n):
        todo, seen = [n], set()
        while todo:
            x = todo.pop()
            if x not in seen:
                seen.add(x)
                todo.extend(adjacency[x] - seen)
        return seen
    reach = {n: reachable(n) for n in nodes}
    remaining, result = set(nodes), []
    while remaining:
        n = min(remaining)
        component = {x for x in remaining if x in reach[n] and (not directed or n in reach[x])}
        remaining -= component
        result.append(sorted(component))
    return result


def dot(name, nodes, edges, kinds=None):
    quote = lambda s: json.dumps(s, ensure_ascii=False)
    lines = ['digraph G {', 'rankdir=LR;', 'node [fontname="sans-serif", fontsize=10];']
    for n in sorted(nodes):
        style = 'shape=box' if n.startswith('rule:') else 'shape=ellipse'
        label = n if not kinds or n not in kinds else n + '\n' + kinds[n]
        lines.append(f'{quote(n)} [{style}, label={quote(label)}];')
    for a, b, label in sorted(set(edges)):
        lines.append(f'{quote(a)} -> {quote(b)} [label={quote(label)}, fontsize=8];')
    lines.append('}')
    (OUT / f'{name}.dot').write_text('\n'.join(lines) + '\n')


def graph_report(label):
    inventory = rows(f'{label}-inventory.tsv')
    types = {r['name'] for r in inventory if r['kind'] == 'entity'}
    type_rows = rows(f'{label}-structure.tsv')
    type_edges = [(r['source'], r['target']) for r in type_rows]
    dot(f'{label}-types', types, [(r['source'], r['target'], r['field'] + ':' + r['declared_type']) for r in type_rows])
    inference = rows(f'{label}-inference.tsv')
    predicates = {r['name'] for r in inventory if r['kind'] == 'verb'}
    predicates |= {r['source'] for r in inference if r['kind'] == 'premise'}
    heads = {r['rule']: r['target'] for r in inference if r['kind'] == 'consequence'}
    projected = [(r['source'], heads[r['rule']]) for r in inference if r['kind'] == 'premise']
    hypernodes = predicates | {'rule:' + r['name'] for r in inventory if r['kind'] == 'rule'}
    categories = {r['verb']: r['category'] for r in rows(f'{label}-verbs.tsv')}
    categories['~Act'] = 'Seeded / EvidentialNot'
    categories.update({'rule:' + r['name']: r['origin_module'] for r in inventory if r['kind'] == 'rule'})
    dot(f'{label}-inference', hypernodes, [(r['source'], r['target'], r['kind']) for r in inference], categories)
    effects = rows(f'{label}-effects.tsv')
    effect_nodes = {'State input', 'State output'} | {r['verb'] for r in effects} | {r['target'] for r in effects}
    effect_edges = [(r['verb'], r['target'], r['operation'] + ': ' + r['target_type']) for r in effects]
    effect_edges += [('State input', v, 'execute, externally requested') for v in sorted({r['verb'] for r in effects})]
    effect_edges += [(v, 'State output', 'functional snapshot') for v in sorted({r['verb'] for r in effects})]
    dot(f'{label}-effects', effect_nodes, effect_edges)
    imports = rows(f'{label}-imports.tsv')
    module_nodes = {r[k] for r in imports for k in ['importer', 'imported']}
    dot(f'{label}-modules', module_nodes,
        [(r['importer'], r['imported'], 'import') for r in imports])
    return {
        'declarations': dict(collections.Counter(r['kind'] for r in inventory)),
        'verb_categories': dict(collections.Counter(r['category'] for r in rows(f'{label}-verbs.tsv'))),
        'type_components': components(types, type_edges, False),
        'type_scc': components(types, type_edges, True),
        'predicate_components': components(predicates, projected, False),
        'predicate_scc': components(predicates, projected, True),
        'predicate_count': len(predicates),
        'hypergraph_node_count': len(hypernodes),
        'hypergraph_edge_count': len(inference),
    }


def window_report(name):
    states = rows(f'{name}-states.tsv')
    frontiers = rows(f'{name}-frontiers.tsv')
    result = {}
    for model in ['general', 'patrimonial']:
        selected = [r for r in states if r['model'] == model]
        closures = {r['closure'] for r in selected}
        inputs = {r['input'] for r in selected}
        support = collections.defaultdict(list)
        for r in frontiers:
            if r['model'] == model:
                support[r['event']].append({'mask': int(r['minimal_mask']), 'support': ast.literal_eval(r['support'])})
        sizes = [len(ast.literal_eval(r['closure'])) for r in selected]
        regions = collections.Counter(e for r in selected for e in ast.literal_eval(r['closure']))
        result[model] = {'input_states': len(inputs), 'distinct_closures': len(closures),
                         'min_events': min(sizes), 'max_events': max(sizes),
                         'max_justifications': max(int(r['proofs']) for r in selected),
                         'minimal_supports': dict(sorted(support.items())),
                         'region_cardinalities': dict(sorted(regions.items()))}
    return result


def theory_hasse():
    rows_ = rows('theory-states.tsv')
    # Extract the stable Debug strings only as opaque Event objects, no semantic evaluation.
    # A tiny balanced-brace scanner splits the already exported BTreeSet<Event>.
    def split_events(text):
        depth, start, result = 0, None, []
        for i, c in enumerate(text):
            if c == '{':
                depth += 1
                if depth == 2:
                    start = i - len('Event ')
            elif c == '}':
                if depth == 2:
                    result.append(text[start:i + 1])
                depth -= 1
        return frozenset(result)
    report = {}
    for model in ['general', 'patrimonial']:
        groups = collections.defaultdict(list)
        for r in rows_:
            groups[split_events(r[model])].append(int(r['mask']))
        ordered = sorted(groups, key=lambda x: (len(x), min(groups[x])))
        names = {s: f'{model[0].upper()}{i}: masks {groups[s]}' for i, s in enumerate(ordered)}
        edges = [(a, b) for a in ordered for b in ordered if a < b and not any(a < c < b for c in ordered)]
        dot(f'{model}-theory-hasse', set(names.values()), [(names[a], names[b], 'inclusion cover') for a, b in edges])
        report[model] = []
        for s in ordered:
            minimal = [a for a in groups[s] if not any(b != a and b & a == b for b in groups[s])]
            if len(minimal) != 1:
                raise SystemExit(f'Non-unique observed theory generator in {model}: {groups[s]}')
            report[model].append({'class': names[s], 'events': len(s), 'masks': groups[s],
                                  'minimal_input_mask': minimal[0]})
    return report


def extreme_report():
    """Summarize engine certificates; no new closure computation or powerset."""
    names = ['extreme-theory-counts.tsv', 'extreme-theory-states.tsv',
             'extreme-seeded-states.tsv', 'extreme-common-states.tsv',
             'extreme-extension-comparison.tsv', 'extreme-certificates.tsv',
             'extreme-basis-removal.tsv']
    present = [(RAW / n).is_file() for n in names]
    if not any(present):
        return None
    if not all(present):
        raise SystemExit('Incomplete extreme-point exports: run all three individual extremes tests')
    common = rows('extreme-common-states.tsv')
    for r in common:
        r['closed_ids'] = ast.literal_eval(r['closed_ids'])
        r['extreme_ids'] = ast.literal_eval(r['extreme_ids'])
        r['closed_events'] = ast.literal_eval(r['closed_events'])
        r['extreme_events'] = ast.literal_eval(r['extreme_events'])
        for field in ['closed_count', 'basis_count', 'generator_count']:
            r[field] = int(r[field])
    categories = {m: collections.Counter() for m in ['general', 'patrimonial']}
    certificates = rows('extreme-certificates.tsv')
    for r in certificates:
        if r['state'] == 'whole-universe' and r['extreme'] == 'true':
            categories[r['model']][r['category']] += 1
    return {
        'theory_counts': [{k: (v if k == 'model' else int(v)) for k, v in r.items()}
                          for r in rows('extreme-theory-counts.tsv')],
        'seeded_states_checked': len(rows('extreme-seeded-states.tsv')),
        'common_closed_states': common,
        'extension_comparison': rows('extreme-extension-comparison.tsv'),
        'whole_universe_extreme_categories': {m: dict(v) for m, v in categories.items()},
        'removal_certificates': len(certificates),
        'basis_omission_certificates': len(rows('extreme-basis-removal.tsv')),
        'large_generator_counts': 'Interval formula, not powerset enumeration',
    }


def goal_frontier_report():
    """Read sufficient/minimal supports and compare their upward closures."""
    names = ['goal-frontier-certificates.tsv', 'goal-frontier-omissions.tsv',
             'goal-frontier-totals.tsv', 'goal-frontier-regions.tsv']
    present = [(RAW / n).is_file() for n in names]
    if not any(present):
        return None
    if not all(present):
        raise SystemExit('Incomplete goal-frontier exports: run both individual frontiers tests')
    universe = {r['id']: r['event'] for r in rows('universe.tsv')}
    frontier = collections.defaultdict(set)
    for r in rows('goal-frontier-certificates.tsv'):
        frontier[r['model'], r['domain'], r['conclusion_id']].add(
            frozenset(ast.literal_eval(r['support_ids'])))
    selected = {}
    prefixes = [
        'obligation(gNorm,', 'permission(gNorm,', 'prohibition(gNorm,',
        'compliance(gNorm,', 'violation(gNorm,', 'enforceability(gNorm,',
        'disputedPerformance(gNorm,', 'inconsistent(gAct,',
        'patrimonialLiability(gSubject, gObject, gNorm,',
        'patrimonialAsset(gBeneficiary, gObject, gNorm,',
        'owns(gBeneficiary, gObject, gNorm,',
        'capacity(gBeneficiary, use, gObject, gNorm,',
        'law(gStatute,', 'annulled(gLow,',
    ]
    for prefix in prefixes:
        matching = [i for i, event in universe.items() if event.startswith(prefix)]
        if len(matching) != 1:
            raise SystemExit(f'Non-unique selected goal: {prefix}: {matching}')
        selected[prefix.split('(')[0]] = matching[0]
    relations = []
    for model in ['general', 'patrimonial']:
        for domain in ['all-events', 'seeded']:
            for a, ai in selected.items():
                for b, bi in selected.items():
                    fa = frontier[model, domain, ai]
                    fb = frontier[model, domain, bi]
                    failures = [s for s in fa if not any(t <= s for t in fb)]
                    witness = min(failures, key=lambda s: (len(s), sorted(s))) if failures else None
                    relations.append({'model': model, 'domain': domain, 'from': a, 'to': b,
                                      'region_inclusion': not failures,
                                      'counterexample_support': None if witness is None else sorted(witness)})
    comparison = []
    for domain in ['all-events', 'seeded']:
        for q in universe:
            fg = frontier['general', domain, q]
            fp = frontier['patrimonial', domain, q]
            # Inclusion of regions, not inclusion of minimal-antichain members.
            assert all(any(t <= s for t in fp) for s in fg)
            if fg != fp:
                comparison.append({'domain': domain, 'conclusion_id': q, 'conclusion': universe[q],
                                   'added_supports': sorted([sorted(s) for s in fp - fg]),
                                   'removed_supports': sorted([sorted(s) for s in fg - fp])})
    summary = {
        'totals': [{k: (v if k in ['model', 'domain'] else int(v)) for k, v in r.items()}
                   for r in rows('goal-frontier-totals.tsv')],
        'selected_goals': {name: {'id': i, 'event': universe[i],
                                'frontiers': {m: {d: sorted([sorted(s) for s in frontier[m, d, i]])
                                                 for d in ['all-events', 'seeded']}
                                              for m in ['general', 'patrimonial']}}
                           for name, i in selected.items()},
        'region_relations': relations,
        'extension_comparison': comparison,
        'bounded_region_checks': sum(int(r['states']) for r in rows('goal-frontier-regions.tsv')),
        'meaning': 'Minimal sufficient supports, not complete-state bases or preferred explanations',
    }
    for name, i in selected.items():
        summary['selected_goals'][name]['necessary_core'] = {
            m: {d: (sorted(set.intersection(*(set(s) for s in frontier[m, d, i])))
                    if frontier[m, d, i] else None)
                for d in ['all-events', 'seeded']}
            for m in ['general', 'patrimonial']}
    columns = ['model', 'domain', 'from', 'to', 'region_inclusion', 'counterexample_support']
    with (OUT / 'data/goal-frontier-relations.tsv').open('w') as stream:
        writer = csv.DictWriter(stream, fieldnames=columns, delimiter='\t')
        writer.writeheader()
        writer.writerows(relations)
    return summary


def residual_frontier_report():
    names = ['residual-frontier-comparison.tsv', 'residual-frontier-witnesses.tsv',
             'residual-frontier-boundaries.tsv']
    present = [(RAW / n).is_file() for n in names]
    if not any(present):
        return None
    if not all(present):
        raise SystemExit('Incomplete residual comparison: run both individual tests')
    comparison = rows(names[0])
    counts = {m: dict(collections.Counter(r['status'] for r in comparison if r['model'] == m))
              for m in ['general', 'patrimonial']}
    return {'status_counts': counts, 'queries': comparison,
            'boundary_cases': rows(names[2]),
            'classification': {
                'equal-minimal-frontier': 'Completed query matches the geometric frontier inside the finite Seeded domain',
                'unsupported-unbound-parameter': 'Err, not no explanation or a completed search',
                'outside-finite-source-domain': 'Forward-certified Seeded hypotheses outside the fixed finite evidence domain',
            }}


def query_dependency_report():
    names = ['cone', 'edges', 'preservation', 'explicit-context',
             'default-status', 'activation-guard', 'activation-boundary']
    files = [f'query-dependency-{n}.tsv' for n in names]
    if not any((RAW / n).exists() for n in files):
        return None
    if not all((RAW / n).exists() for n in files):
        raise SystemExit('Incomplete query dependency exports: run the four individual tests')
    data = {n: rows(f'query-dependency-{n}.tsv') for n in names}
    graph = [r for r in data['edges'] if r['model'] == 'general']
    nodes, edges = set(), []
    for r in graph:
        rule = 'rule:' + r['rule']
        nodes.update([r['premise_literal'], rule, r['consequent_literal']])
        edges.extend([(r['premise_literal'], rule, 'premise'),
                      (rule, r['consequent_literal'], 'consequence')])
    dot('query-compliance-dependencies', nodes, edges,
        {n: 'requires every premise' for n in nodes if n.startswith('rule:')})
    result = {
        'method': 'conservative signature cone, not a ground match or an activation',
        'graph_nodes': len(nodes),
        'models': {},
        'preservation_comparisons': len(data['preservation']),
        'explicit_contexts': data['explicit-context'],
        'default_cases': data['default-status'],
        'activation_cases': data['activation-guard'],
        'activation_boundary': data['activation-boundary'],
    }
    for model in ['general', 'patrimonial']:
        cone = [r for r in data['cone'] if r['model'] == model]
        result['models'][model] = {
            'rules': [r['rule'] for r in cone],
            'typed_body_only_parameters': sum(r['body_only'].count('Arg {') for r in cone),
        }
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--render', action='store_true', help='Also render DOT as standalone SVG with Graphviz')
    args = parser.parse_args()
    required = ['counterexamples.tsv', 'universe.tsv', 'theory-states.tsv', 'boundaries.tsv',
                'anti-exchange.tsv', 'predicate-order.tsv', 'initial-evidence.tsv',
                'whole-universe-provenance.tsv']
    required += ['full-event-frontiers.tsv', 'full-event-frontier-totals.tsv']
    required += [f'{n}-states.tsv' for n in ['performance', 'authority', 'legislation', 'transferCase', 'holding']]
    for name in required:
        if not (RAW / name).is_file():
            raise SystemExit(f'Missing {name}: run individual tests first')
    OUT.mkdir(parents=True, exist_ok=True)
    data = OUT / 'data'
    data.mkdir(exist_ok=True)
    for path in sorted(RAW.glob('*.tsv')):
        shutil.copyfile(path, data / path.name)
    summary = {'graphs': {n: graph_report(n) for n in ['general', 'patrimonial', 'reference']},
               'windows': {n: window_report(n) for n in ['performance', 'authority', 'legislation', 'transferCase', 'holding']},
               'common_universe_events': len(rows('universe.tsv')),
               'initial_evidence_events': len(rows('initial-evidence.tsv')),
               'anti_exchange': rows('anti-exchange.tsv'),
               'theory_quotients': theory_hasse()}
    full = rows('full-event-frontiers.tsv')
    supports = {m: collections.defaultdict(set) for m in ['general', 'patrimonial']}
    for r in full:
        supports[r['model']][r['conclusion']].add(tuple(ast.literal_eval(r['minimal_support_ids'])))
    summary['full_event_frontiers'] = {
        'totals': rows('full-event-frontier-totals.tsv'),
        'changed_conclusions': [j for j in sorted(supports['general']) if supports['general'][j] != supports['patrimonial'][j]],
    }
    paths = [ROOT / 'examples/legal/modules/legal_monotonic.res',
             ROOT / 'examples/experiments/legal_geometry/ontology_reference/legal_general.res',
             ROOT / 'examples/experiments/legal_geometry/ontology_reference/legal_patrimonial.res']
    summary['sources'] = [{'path': str(p), 'bytes': p.stat().st_size,
                           'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for p in paths]
    expected = ['24655c98ede7b128b8cd82df3a4473c114428b987bb0fe6a6d370c2b37f5575e',
                '639fdcdf194cbd27167be633d53f25c3e6f1e1bcdd1f2ef7a606981b70c60bbc',
                'f7daf4d5eb937a7fde8534dba5618acc50d5aaa5b05ff86c9656892eb3a03fbf']
    if [s['sha256'] for s in summary['sources']] != expected:
        raise SystemExit('Source changed: refuse to publish observations from different ontologies')
    (OUT / 'results.json').write_text(json.dumps(summary, indent=2, ensure_ascii=False) + '\n')
    extremes = extreme_report()
    if extremes is not None:
        (OUT / 'extreme-results.json').write_text(json.dumps(extremes, indent=2, ensure_ascii=False) + '\n')
    goals = goal_frontier_report()
    if goals is not None:
        (OUT / 'goal-frontier-results.json').write_text(json.dumps(goals, indent=2, ensure_ascii=False) + '\n')
    residual = residual_frontier_report()
    if residual is not None:
        (OUT / 'residual-frontier-results.json').write_text(json.dumps(residual, indent=2, ensure_ascii=False) + '\n')
    dependencies = query_dependency_report()
    if dependencies is not None:
        (OUT / 'query-dependency-results.json').write_text(json.dumps(dependencies, indent=2, ensure_ascii=False) + '\n')
    if args.render:
        for path in sorted(OUT.glob('*.dot')):
            subprocess.run(['dot', '-Tsvg', str(path), '-o', str(path.with_suffix('.svg'))],
                           check=True, timeout=10)
    print(json.dumps({
        'graphs': summary['graphs'],
        'universe_events': summary['common_universe_events'],
        'windows': {n: {m: {k: v for k, v in d.items() if k not in ['minimal_supports', 'region_cardinalities']} for m, d in w.items()} for n, w in summary['windows'].items()},
        'theory_classes': {n: len(v) for n, v in summary['theory_quotients'].items()},
        'full_event_frontiers': summary['full_event_frontiers'],
        'extreme_theory_counts': None if extremes is None else extremes['theory_counts'],
    }, indent=2))


if __name__ == '__main__':
    main()
