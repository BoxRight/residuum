import json
from collections import Counter
R = {}
for l in open('output.jsonl'):
    r = json.loads(l); r = r.get('response', r); R[r['action_id']] = r
idx = json.load(open('index.json'))
print('statuses', Counter(r['status'] for r in R.values()))
def fmt(e):
    a = ', '.join(t.get('const', str(t)) for t in e['args'])
    neg = '~' if e['polarity'] != 'positive' else ''
    t = e['time']; t = t.get('at', t)
    return f"{neg}{e['verb']}({a}) @ {t}"
def known(label):
    last = None
    for a in idx[label]:
        p = R[a]['payload']
        if 'forward' in p: last = p['forward']
    assert last and last.get('complete'), (label, last and last.get('complete'))
    return {fmt(e) for e in last['known'] if e['kind'] != 'effect'}
dossiers = ["brookside", "casaFiesta", "cf_filed", "cf_no_agreeU", "cf_nonperf", "cf_filed_nonperf"]
K = {(d, th): known(f"{th} on {d}") for d in dossiers for th in ("O", "O+h1", "O+h3")}
seeds = json.load(open('manifest.json'))['dossiers']
for d in dossiers:
    S = {fmt(e) for e in seeds[d]}
    print(f"\n== {d}: |S|={len(S)} |C_O|={len(K[(d,'O')])}")
    for e in sorted(K[(d,'O')] - S): print("   O derives:", e)
    for th in ("O+h1", "O+h3"):
        diff = K[(d, th)] - K[(d, 'O')]
        print(f"  {th} adds {len(diff)}:")
        for e in sorted(diff): print("     +", e)
print("\n== world diffs (same theory, delta on facts)")
for th in ("O", "O+h1", "O+h3"):
    for b in ["cf_filed", "cf_no_agreeU", "cf_nonperf", "cf_filed_nonperf"]:
        a = "casaFiesta"
        print(f"  [{th}] {b} vs {a}:")
        for e in sorted(K[(b, th)] - K[(a, th)]): print("      +", e)
        for e in sorted(K[(a, th)] - K[(b, th)]): print("      -", e)
print("\n== certified evaluations")
for k, a in idx.items():
    if k.startswith('certified'):
        p = R[a]['payload']
        print(' ', k, R[a]['status'], p.get('session_opened') or p.get('candidate', {}).get('report', {}).get('status'))
print("\n== supports and partitions")
for k, a in idx.items():
    if k.startswith('supports') or k.startswith('partition'):
        r = R[a]; p = r['payload']
        if 'supports' in p:
            s = p['supports']; print(f"  {k}: in_universe={s['goal_in_universe']}")
            for f in s['frontier']: print("      {" + ', '.join(fmt(e) for e in f) + "}")
        elif 'partition' in p:
            q = p['partition']
            print(f"  {k}: discriminants={q['discriminants']} covers={q['covers']}")
            for i, reg in enumerate(q['pool']):
                sig = {c: q['signatures'][j][i] for j, c in enumerate(q['partition']['hypotheses'])} if isinstance(q.get('partition'), dict) and 'hypotheses' in q['partition'] else [s[i] for s in q['signatures']]
                print(f"      [{i}] {{{', '.join(fmt(e) for e in reg)}}} h1,h3={sig}")
        else: print(' ', k, r['status'], json.dumps(p)[:200])
