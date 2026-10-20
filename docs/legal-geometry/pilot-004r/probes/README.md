# Pilot 004-R exploration-question probe

`tools/pilot004r/probe.py` replays the exact first request of run-004, the
Brookside explore view before any feedback or hypothesis. It uses the same
system instructions, the same eleven tools and the same model parameters, and
appends one neutral question:

> Before taking any action in this phase: list the questions about this
> dossier and the theory that you would investigate, if any. For each one,
> say which tool you would use and what result would change your next step.
> If you see nothing worth investigating, say why. Do not call any tool now;
> answer in text.

Tool calls are disabled (`tool_choice: none`). Each replica is one
independent request; at most 5 replicas, 3 by default. Nothing runs in
Residuum and no valuation is involved. The question names no rule, head,
hypothesis or tool.

Purpose: to tell apart "the model cannot formulate investigable questions"
from "it can, but the information contract gives it no reason to act on
them" (see the analysis after run-004).

Planned command (paid, from a terminal with the key and workspace):

    cd /tmp/residuum-pilot004-documentary && python3 -B tools/pilot004r/probe.py \
        --ledger docs/legal-geometry/pilot-004r/runs/run-004/run/ledger/turns.jsonl \
        --manifest docs/legal-geometry/pilot-004r/runs/run-004/trajectory.json \
        --out target/pilot004r/probe-001 --replicas 3

Estimate: run-004's first request was about 13.9k input tokens, so about
US$0.03 input plus output per replica, roughly US$0.15 for three replicas.
