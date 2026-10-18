# Temporal history of normative formalizations

Report: `docs/legal-temporal-history.md`.

- `fixtures/`: Formal modules. `stage_k.res` is T_k = O ∪ I_tk(R_k) with its
  isolated dossier; `history1.res` and `history2.res` are the accumulated
  histories; `hypothetical_r1_on_dossier0.res` is an isolated hypothetical run;
  `alternatives/` holds A (pinned rules) and B (specialization);
  `counterexamples/` the four minimal violations of the locality conditions.
- `runs/`: archived `--forward` and `--cli explain` outputs through
  `scripts/test_safe.py`, and `certification-tests.txt` with the report id of
  each certification test.
- Certification: `src/bin/residuum/cli/temporal_history_tests.rs`, one test
  at a time with
  `python3 scripts/test_safe.py --test-target cli --test cli::tests::temporal_history::<name>`.
