"""Build a worker manifest + request script for the retrieval/counterfactual
feasibility simulation. Existing signatures and constants only."""
import json, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = Path("/home/maitreya/residuum-pilot004-documentary")
P4R = REPO / "docs/legal-geometry/pilot-004r"

def ev(verb, args, t, neg=False, kind="seeded"):
    return {"polarity": "evidential_not" if neg else "positive", "verb": verb,
            "args": [{"const": a} for a in args], "kind": kind,
            "time": {"at": t}, "transition": None}

B = "private d20210216".split()
brookside = [
    ev("Act", ["saB_j", "jant", "agree", "saDocB", "usa", *B], "tB"),
    ev("Act", ["saB_u", "usa", "agree", "saDocB", "jant", *B], "tB"),
    *[ev("obligationFiled", ["saB_j", "jant", c, o, "usa", *B], "tB")
      for c, o in [("adoptPolicy", "policyA"), ("distribute", "policyA"), ("post", "policyA"),
                   ("keepAvailable", "policyA"), ("post", "signB")]],
    ev("prohibition", ["saB_u", "usa", "sue", "premisesB", "jant", *B], "tB"),
]
C = "private dCF".split()
agreeC = ev("Act", ["saCF_c", "casaFiesta", "agree", "saDocCF", "usa", *C], "tCF")
agreeU = ev("Act", ["saCF_u", "usa", "agree", "saDocCF", "casaFiesta", *C], "tCF")
adopt = ev("Act", ["cfA1", "casaFiesta", "adoptPolicy", "policyCF", "usa", *C], "tCF")
post = ev("Act", ["cfA2", "casaFiesta", "post", "signCF", "usa", *C], "tCF")
# Counterfactual atoms (no valuation; not documented facts).
filed = ev("obligationFiled", ["saCF_c", "casaFiesta", "adoptPolicy", "policyCF", "usa", *C], "tCF")
nonAdopt = ev("Act", ["cfA1", "casaFiesta", "adoptPolicy", "policyCF", "usa", *C], "tCF", neg=True)

casa = [agreeC, agreeU, adopt, post]
dossiers = {
    "brookside": brookside,
    "casaFiesta": casa,
    "cf_filed": casa + [filed],                       # delta: add the written obligation
    "cf_no_agreeU": [agreeC, adopt, post],            # delta: remove usa's agreement
    "cf_nonperf": [agreeC, agreeU, nonAdopt, post],   # delta: replace performance by documented non-performance
    "cf_filed_nonperf": [agreeC, agreeU, nonAdopt, post, filed],
}

prof = {
    "heads": ["obligation", "permission", "prohibition", "omissionProhibition", "right",
              "compliance", "violation", "enforceability", "patrimonialLiability"],
    "body_verbs": ["Act", "right", "obligationFiled", "prohibition", "omissionProhibition",
                   "permissionFiled", "patrimonialHolding", "powerGranted", "legislationText",
                   "obligation", "permission", "compliance", "violation", "enforceability",
                   "patrimonialAsset", "patrimonialLiability", "transferred", "owns", "capacity", "law"],
    "evidential_not_verbs": ["Act"], "max_atoms": 4, "max_binders": 16,
    "max_after_depth": 1, "max_unit_nodes": 1, "require_connected": False,
}
EPI, HASH = "feasibility-sim", "h-feasibility-sim"
manifest = {
    "wire_version": "residuum-wire/1", "episode_id": EPI, "manifest_hash": HASH,
    "imports": [str(P4R / "vocabulary.res")],
    "workdir": str(HERE / "work"),
    "profile": prof,
    # Placeholder decision as in the 004-R navigator: an arbitrary seed, never evaluated.
    "decision": {"components": [brookside[0]], "criterion": {"component": 0}},
    "dossiers": dossiers,
    "budgets": {"max_actions": 400, "max_steps_per_action": 64, "max_steps_total": 20000,
                "max_sessions": 60, "max_candidates": 4, "max_clauses": 2,
                "search_limits": {"max_depth": 32, "max_nodes": 4096, "max_branches": 4096}},
    "transport": None,
}
H1 = json.load(open(P4R / "runs/run-013/run/ledger/trajectory-report.json"))["hypotheses"]
texts = {h["id"]: h["text"] for h in H1}

reqs, n, sessions = [], [0], [0]
def send(r):
    n[0] += 1
    reqs.append({"action_id": f"a{n[0]}", "episode_id": EPI, "manifest_hash": HASH,
                 "protocol_version": "residuum-worker/0.1", "request": r, "wire_version": "residuum-wire/1"})
    return f"a{n[0]}"
index = {}
index["propose h1"] = send({"propose_candidate": {"text": texts["h1"]}})
index["propose h3"] = send({"propose_candidate": {"text": texts["h3"]}})
def run_session(label, opener):
    send(opener); sessions[0] += 1; s = f"s{sessions[0]}"
    index[label] = [send({"advance_session": {"session": s, "max_steps": 64}}) for _ in range(3)]
for d in dossiers:
    run_session(f"O on {d}", {"begin_forward": {"dossier": d, "theory": None}})
    run_session(f"O+h1 on {d}", {"begin_forward": {"dossier": d, "theory": "c1"}})
    run_session(f"O+h3 on {d}", {"begin_forward": {"dossier": d, "theory": "c2"}})
# One certified evaluation per candidate, to record what check_scope accepts.
for c in ("c1", "c2"):
    for d in ("brookside", "casaFiesta", "cf_filed"):
        index[f"certified {c} on {d}"] = send({"begin_candidate_evaluation": {"candidate": c, "dossier": d}})
goals = {
    "obligation(cfA1)": ev("obligation", ["cfA1", "casaFiesta", "adoptPolicy", "policyCF", "usa", *C], "tCF", kind="derived"),
    "compliance(saCF_c,cfA1)": ev("compliance", ["saCF_c", "cfA1", "casaFiesta", "policyCF", "usa", *C], "tCF", kind="derived"),
    "compliance(cfA1,cfA1)": ev("compliance", ["cfA1", "cfA1", "casaFiesta", "policyCF", "usa", *C], "tCF", kind="derived"),
    "violation(saCF_c,cfA1)": ev("violation", ["saCF_c", "cfA1", "casaFiesta", "policyCF", "usa", *C], "tCF", kind="derived"),
    "enforceability(saCF_c,cfA1)": ev("enforceability", ["saCF_c", "cfA1", "casaFiesta", "policyCF", "usa", *C], "tCF", kind="derived"),
}
for g, e in goals.items():
    index[f"supports h1 {g}"] = send({"minimal_supports": {"candidate": "c1", "goal": e}})
    index[f"supports h3 {g}"] = send({"minimal_supports": {"candidate": "c2", "goal": e}})
    index[f"partition {g}"] = send({"compare_partition": {"candidates": ["c1", "c2"], "goal": e}})

(HERE / "manifest.json").write_text(json.dumps(manifest, indent=1) + "\n")
(HERE / "requests.jsonl").write_text("".join(json.dumps(r) + "\n" for r in reqs))
(HERE / "index.json").write_text(json.dumps(index, indent=1) + "\n")
print(len(reqs), "requests")
