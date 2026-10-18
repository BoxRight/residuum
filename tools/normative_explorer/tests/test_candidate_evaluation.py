"""Stage I after pilot 003: the archived scripts are the ones the module
builds, and the archived worker output yields the archived table.
Run individually through ``scripts/test_safe.py --python-test``."""
import json
import unittest
from pathlib import Path

from tools.normative_explorer import candidate_evaluation as ce

ROOT = Path(__file__).resolve().parents[3]
PILOT = ROOT / "docs" / "legal-geometry" / "exploration" / "pilot-003"
LEDGER = PILOT / "run" / "ledger"
EVAL = PILOT / "evaluation"
VOCABULARY = "59ea0b266407558db7e63002695b5b6f6f584de35ae38d5ad69807eb37b55e64"
CHANNEL = "hypotheticalSuccessionImpediment"


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines()]


class CandidateEvaluationTests(unittest.TestCase):
    def test_archived_scripts_outputs_and_table_agree(self):
        # The scripts the worker received are the ones rebuilt from the ledger
        # and from the candidate text, envelope for envelope.
        self.assertEqual(ce.reproduction_requests(LEDGER), lines(EVAL / "reproduction-requests.jsonl"))
        worker = json.loads((EVAL / "evaluation-manifest.json").read_text())
        candidate = (EVAL / "candidate.formal").read_text()
        self.assertEqual(ce.evaluation_requests(worker, candidate), lines(EVAL / "evaluation-requests.jsonl"))

        # Reproduction: every archived response of pilot 003 again.
        reproduction = json.loads((EVAL / "reproduction-manifest.json").read_text())
        archived = json.loads((LEDGER / "worker-manifest.json").read_text())
        rows = ce.compare_reproduction(lines(EVAL / "reproduction-output.jsonl"), ce.expected_responses(LEDGER),
                                       reproduction["workdir"], archived["workdir"])
        self.assertEqual([r["action_id"] for r in rows], ["t1", "p1", "p2", "p2", "p3", "a1", "a2", "a3"])
        self.assertTrue(all(r["identical"] for r in rows))
        self.assertEqual(rows, json.loads((EVAL / "reproduction-comparison.json").read_text()))

        # Evaluation: the archived table follows from the archived output.
        vocabulary = json.loads((LEDGER / "artifacts" / "sha256" / VOCABULARY).read_text())
        verbs = {v["name"] if isinstance(v, dict) else v for v in vocabulary["verbs"]}
        table = ce.summary(ce.analyse(lines(EVAL / "evaluation-output.jsonl"), worker, CHANNEL, verbs - {CHANNEL}))
        self.assertEqual(table, json.loads((EVAL / "summary.json").read_text()))
        dossiers = table["dossiers"]
        self.assertEqual(len(dossiers), 6)
        for name, row in dossiers.items():
            self.assertTrue(row["baseline"]["complete"] and row["candidate"]["complete"], name)
            self.assertFalse(row["baseline"]["channel"], name)
            self.assertTrue(row["reference_preserved"], name)
        coverage = {name for name, row in dossiers.items() if row["candidate"]["channel"]}
        self.assertEqual(coverage, {"historicalRiggs"})
        self.assertEqual(dossiers["historicalRiggs"]["candidate"]["rules_reaching_channel"], ["impediment"])

        # The two evaluation runs were byte-identical.
        digests = {line.split()[0] for line in (EVAL / "evaluation-output.sha256").read_text().splitlines()}
        self.assertEqual(len(digests), 1)


if __name__ == "__main__":
    unittest.main()
