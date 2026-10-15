"""Checks over the archived runner outputs of the temporal-activation fixtures.

Reads only runs/*/output.txt (forward mode); prints and writes summary.json.
No engine call: every event comes from Residuum's own closure output."""
import json
import re
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
NORMATIVE = ("prohibition", "right", "permission", "obligation", "violation",
             "patrimonialLiability", "annulled", "compliance", "enforceability")


def split_events(text):
    events, depth, cur = [], 0, ""
    for ch in text:
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        if ch == "," and depth == 0:
            events.append(cur.strip())
            cur = ""
        else:
            cur += ch
    if cur.strip():
        events.append(cur.strip())
    return events


def closures(run):
    out, name = {}, None
    for line in (HERE / "runs" / run / "output.txt").read_text().splitlines():
        if line.startswith("experiment "):
            name = line.split()[1]
        m = re.match(r"\s+Known after close: \{(.*)\}$", line)
        if m:
            out[name] = set(split_events(m.group(1)))
    return out


def verb(e):
    return e.lstrip("~").split("(")[0]


def args(e):
    inner = e[e.index("(") + 1:e.rindex(")")] if "@" not in e else e[e.index("(") + 1:e.rindex(")", 0, e.index("@"))]
    return [a.strip() for a in split_events(inner)]


def normative(known):
    return sorted(e for e in known if verb(e) in NORMATIVE)


def by_epoch(known, epochs):
    """Normative consequences whose last (effective) argument is an epoch."""
    table = {ep: [] for ep in epochs}
    for e in normative(known):
        a = args(e)
        if a and a[-1] in table:
            table[a[-1]].append(e)
    return table


# (run, experiment, event, expected membership in the closure)
P = "prohibition(dogBan, {o}, enterWith, {d}, host, administrative, enact0) @ {t}"
EXPECT = [
    ("f1_indexing", "indexing", P.format(o="ana", d="rex", t="t0"), True),
    ("f1_indexing", "indexing", P.format(o="ana", d="rex", t="t1"), True),
    ("f1_indexing", "indexing", P.format(o="ana", d="rex", t="after(t0)"), True),
    ("f1_indexing", "indexing", "violation(dogBan, visitAna, ana, rex, host, administrative, enact0) @ t1", True),
    ("f1_indexing", "afterSteps", "reminded(ana, rex) @ after(after(t0))", True),
    ("f1_indexing", "afterSteps", "twoStepProbe(ana, rex) @ t0", False),
    ("f1_indexing", "retroBase", "incidentAtVisit(ana, rex) @ t0", False),
    ("f1_indexing", "retroLate", "incidentAtVisit(ana, rex) @ t0", True),
    ("f3_program_activation", "selection", "right(serviceEntry, ana, enterWith, rex, host, legislative, enact3) @ t3", True),
    ("f3_program_activation", "selection", "right(serviceEntry, ana, enterWith, rex, host, legislative, enact3) @ t0", False),
    ("f3_program_activation", "selection", "right(serviceEntry, bea, enterWith, fido, host, legislative, enact3) @ t3", False),
    ("f3_program_activation", "selection", P.format(o="ana", d="rex", t="t3"), True),
    ("f3_program_activation", "passive", "right(serviceEntry, ana, enterWith, rex, host, legislative, enact3) @ t3", False),
    ("f5_default_temporal-run", "blockerLater", P.format(o="ana", d="rex", t="t0"), False),
    ("f5_default_temporal-run", "blockerLater", P.format(o="bea", d="fido", t="t1"), False),
    ("f5_default_temporal-run", "noBlocker", P.format(o="ana", d="rex", t="t0"), True),
    ("f5_default_temporal-run", "noBlocker", P.format(o="bea", d="fido", t="t1"), True),
    ("f6_constant_time", "twoTimes", "pinnedProbe(ana, rex) @ t0", True),
    ("f6_constant_time", "twoTimes", "pinnedProbe(ana, rex) @ t1", False),
    ("f6_constant_time", "twoTimes", "prohibition(templateBan, ana, enterWith, rex, host, administrative, enact0) @ t1", True),
    ("f7_wrapped_generalization", "history", P.format(o="ana", d="rex", t="t0"), False),
    ("f7_wrapped_generalization", "twoDogs", P.format(o="ana", d="rex", t="t1"), False),
    ("f7_wrapped_generalization", "twoDogs", P.format(o="bea", d="fido", t="t1"), True),
    ("f8_wrapped_with_vigencia", "history", P.format(o="ana", d="rex", t="t0"), True),
    ("f8_wrapped_with_vigencia", "historyWithoutOrder", P.format(o="ana", d="rex", t="t0"), False),
    ("f8_wrapped_with_vigencia", "twoDogs", P.format(o="ana", d="rex", t="t1"), False),
    ("f8_wrapped_with_vigencia", "twoDogs", P.format(o="bea", d="fido", t="t1"), True),
    ("f8_wrapped_with_vigencia", "aBeforeRevision", P.format(o="ana", d="rex", t="t0"), True),
]


def main():
    summary = {}
    cache = {}
    rows = []
    for run, experiment, event, expected in EXPECT:
        known = cache.setdefault(run, closures(run))[experiment]
        rows.append({"run": run, "experiment": experiment, "event": event,
                     "expected": expected, "holds": (event in known) == expected})
    summary["expectations"] = rows
    # F2: adding G1 never removes; annulled carries no subject.
    base, ext = closures("f2_base"), closures("f2_extended")
    summary["f2"] = {name: {"base_subset_of_extended": base[name] <= ext[name],
                            "removed": sorted(base[name] - ext[name]),
                            "added": sorted(ext[name] - base[name])} for name in base}
    # F4: theory chain and per-epoch history.
    epochs = ["e0", "e1", "e2", "e3"]
    runs = [closures(f"f4_t{k}") for k in range(4)]
    f4 = {"inclusion": {}, "history": {}, "assessments": {}}
    for k in range(3):
        f4["inclusion"][f"T{k}<=T{k + 1}"] = all(runs[k][n] <= runs[k + 1][n] for n in runs[k])
    for name in runs[0]:
        tables = [by_epoch(r[name], epochs) for r in runs]
        f4["assessments"][name] = {f"T{k}": {ep: tables[k][ep] for ep in epochs if tables[k][ep]}
                                   for k in range(4)}
        # Epoch e_j is learned at T_j: from T_j on its assessments must not change.
        f4["history"][name] = {ep: all(tables[k][ep] == tables[j][ep] for k in range(j, 4))
                               for j, ep in enumerate(epochs)}
    summary["f4"] = f4
    text = json.dumps(summary, indent=2, ensure_ascii=False, sort_keys=True)
    (HERE / "summary.json").write_text(text + "\n")
    print(text)
    ok = all(r["holds"] for r in rows) and all(v["base_subset_of_extended"] for v in summary["f2"].values()) \
        and all(f4["inclusion"].values()) \
        and all(all(h.values()) for h in f4["history"].values())
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
