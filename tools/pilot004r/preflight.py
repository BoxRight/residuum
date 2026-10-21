#!/usr/bin/env python3
"""Pilot 004-R model preflight: no generation, no cost.

Reads the model block of a trajectory manifest and the `start` record of a
local check of the same contract (system prompt, opening, tools). It asks the
Models API for the model, then counts the input tokens of system + tools +
opening on that model and on a reference model, so the tokenizer ratio can be
applied to a previous run's usage. Prints JSON; never prints the key.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.request
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from driver import api_key  # noqa: E402

BASE = "https://api.anthropic.com/v1"


def request(path: str, body: dict | None = None) -> dict:
    headers = {"x-api-key": api_key(), "anthropic-version": "2023-06-01", "content-type": "application/json"}
    workspace = os.environ.get("ANTHROPIC_WORKSPACE_ID", "").strip()
    if workspace:
        headers["anthropic-workspace-id"] = workspace
    data = json.dumps(body).encode() if body is not None else None
    r = urllib.request.Request(BASE + path, data=data, method="POST" if body is not None else "GET",
                               headers=headers)
    try:
        with urllib.request.urlopen(r, timeout=60) as resp:
            return {"status": resp.status, "body": json.loads(resp.read())}
    except urllib.error.HTTPError as e:
        return {"status": e.code, "body": e.read().decode(errors="replace")[:2000]}


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--manifest", type=Path, required=True)
    p.add_argument("--start", type=Path, required=True, help="turns.jsonl of a local check of the same contract")
    p.add_argument("--reference-model", default="claude-sonnet-5-5")
    a = p.parse_args()
    model = json.loads(a.manifest.read_text(encoding="utf-8"))["driver"]["model"]
    start = next(json.loads(line)["payload"] for line in a.start.open(encoding="utf-8")
                 if json.loads(line)["kind"] == "start")
    out = {"manifest_model": model, "models_api": request(f"/models/{model['model_id']}")}
    for m in (model["model_id"], a.reference_model):
        body = {"model": m, "system": [{"type": "text", "text": start["system"]}], "tools": start["tools"],
                "messages": [{"role": "user", "content": start["opening"]}]}
        if "thinking" in (model.get("parameters") or {}):
            body["thinking"] = model["parameters"]["thinking"]
        out[f"count_tokens[{m}]"] = request("/messages/count_tokens", body)
    print(json.dumps(out, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main())
