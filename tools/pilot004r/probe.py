"""Pilot 004-R exploration-question probe (no action, no trajectory).

Replays the exact first request of an archived trajectory: its system
instructions, tool catalogue and first user message (opening plus the first
explore view, before any feedback). Appends one neutral question and forbids
tool calls (`tool_choice: none`), so the answer is text only. Each replica is
an independent single request. Nothing is executed in Residuum and no
valuation is involved: the first view precedes all scoring.

Usage: python3 -B tools/pilot004r/probe.py --ledger <archived turns.jsonl>
           --manifest <trajectory.json> --out <empty dir under target/> [--replicas N]
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from driver import API_URL, api_key  # noqa: E402

QUESTION = (
    "Before taking any action in this phase: list the questions about this dossier and the theory that you "
    "would investigate, if any. For each one, say which tool you would use and what result would change your "
    "next step. If you see nothing worth investigating, say why. Do not call any tool now; answer in text.")

MAX_REPLICAS = 5


def first_request(ledger: Path) -> tuple[str, list[dict], dict]:
    system = tools = message = None
    for line in ledger.read_text(encoding="utf-8").splitlines():
        entry = json.loads(line)
        if entry["kind"] == "start":
            system, tools = entry["payload"]["system"], entry["payload"]["tools"]
        elif entry["kind"] == "request" and entry["payload"]["n"] == 1:
            message = entry["payload"]["new_message"]
            break
    if system is None or message is None:
        raise SystemExit(f"{ledger}: no start entry or no first request")
    return system, tools, message


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--ledger", type=Path, required=True)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    parser.add_argument("--replicas", type=int, default=3)
    args = parser.parse_args(argv)
    if not 1 <= args.replicas <= MAX_REPLICAS:
        raise SystemExit(f"replicas must be in 1..{MAX_REPLICAS}")
    if "target" not in args.out.resolve().parts or (args.out.exists() and any(args.out.iterdir())):
        raise SystemExit("--out must be an empty directory under target/")
    args.out.mkdir(parents=True, exist_ok=True)
    cfg = json.loads(args.manifest.read_text(encoding="utf-8"))["driver"]["model"]
    system, tools, message = first_request(args.ledger)
    content = list(message["content"]) + [{"type": "text", "text": QUESTION}]
    body = {"model": cfg["model_id"], "max_tokens": cfg["max_tokens"], "system": system, "tools": tools,
            "tool_choice": {"type": "none"}, "messages": [{"role": "user", "content": content}]}
    body.update(cfg.get("parameters") or {})
    headers = {"x-api-key": api_key(), "anthropic-version": "2023-06-01", "content-type": "application/json"}
    workspace = os.environ.get("ANTHROPIC_WORKSPACE_ID", "").strip()
    if workspace:
        headers["anthropic-workspace-id"] = workspace
    p = cfg["pricing"]
    total = 0.0
    for replica in range(1, args.replicas + 1):
        record = {"replica": replica, "time": time.time(), "question": QUESTION,
                  "source_ledger": str(args.ledger), "model": cfg["model_id"]}
        request = urllib.request.Request(API_URL, data=json.dumps(body).encode(), method="POST", headers=headers)
        try:
            with urllib.request.urlopen(request, timeout=cfg.get("timeout_seconds", 180)) as r:
                response = json.loads(r.read())
        except urllib.error.HTTPError as e:
            record["http_error"] = {"status": e.code, "body": e.read().decode(errors="replace")[:4000]}
            (args.out / f"probe-{replica}.json").write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n")
            print(json.dumps({"replica": replica, "http_error": e.code}))
            return 4
        u = response.get("usage", {})
        cost = (u.get("input_tokens", 0) * p["input_per_mtok"]
                + u.get("cache_creation_input_tokens", 0) * p["input_per_mtok"] * p["cache_write_multiplier"]
                + u.get("cache_read_input_tokens", 0) * p["cache_read_per_mtok"]
                + u.get("output_tokens", 0) * p["output_per_mtok"]) / 1e6
        total += cost
        record.update({"response": response, "cost_usd": cost})
        (args.out / f"probe-{replica}.json").write_text(json.dumps(record, indent=2, ensure_ascii=False) + "\n")
        print(json.dumps({"replica": replica, "cost_usd": round(cost, 6)}))
    summary = {"replicas": args.replicas, "cost_usd": round(total, 6)}
    (args.out / "probe-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
