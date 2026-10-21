# Pilot 004-R run-008: autonomous planning before explore

Question: does separating the deliberation about what to investigate from
the execution of actions lead the model to use Residuum's capabilities
before it receives feedback?

## The only intervention (manifest flag `planning`)

Before each explore phase the driver adds one planning request to the same
conversation. The request carries the same view the explore phase receives
and nothing else: no valuations, no feedback beyond what the view already
holds, no reference to any dossier, theory or hypothesis. Tool calls are
disabled (`tool_choice: none`) and no action reaches Residuum. The plan
stays in the conversation. Execution then continues with the existing tools,
and the model may follow the plan, change it or abandon it.

Planning request (literal):

> Planning step: no action is taken now. Before this explore phase, write a
> provisional investigation plan from the information in the current view.
> Reply with one JSON object with these fields: "uncertainties" (the
> uncertainties or relations you consider investigable now), "operations" (a
> list of {"tool", "purpose"} with the available tools that could inform
> them), "distinguishing_results" (results that would tell alternatives
> apart), and "conditions" (when you would continue, change the plan or
> abstain). The plan does not change the committed set, is not scored and
> does not oblige you to commit anything. Do not call any tool now.

Execution request after the plan (literal):

> Your plan is recorded. Now carry out the explore phase with the tools. You
> may follow the plan, change it or abandon it in light of the results.

- The system prompt, tools, budgets, loss, corpus, valuations, model and
  effort are those of run-007 (`trajectory.json` differs only in id, workdir
  and `"planning": true`).
- No gate, no reward and no minimum number of actions.
- The planning call is a model request and is charged in the usage of the
  first execution turn of the phase. It does not consume a harness turn.
- The harness, the bridge and the engine are unchanged. The exact diff is in
  `driver.diff`.

## What is recorded (driver ledger `turns.jsonl`)

- **`plan`**: the dossier, the plan's raw text and the parsed JSON object, or
  `null` if the text holds none.
- **`request_options`** on the planning request: `{"tool_choice": {"type": "none"}}`.
- **`plan_trace`**, at the end of each explore phase:
  - tools planned and tools executed;
  - planned but not executed, and executed but not planned;
  - for each deviation, the observable antecedent: the last result and the
    last navigation reference in the view. This is the antecedent, not a
    motive;
  - consultations with a result (evaluations, and navigation responses other
    than an opened session or an error) and without one (opened sessions,
    errors, pending evaluations).

  A consultation without a result is not counted as exploration with a
  result.
- **Findings, as in run-007:** verifiable results before feedback are the
  findings with verdict `match`. Whether a matched finding cites a record from
  this phase is shown by the novelty diagnostic.

## Local verification (simulated provider, no cost)

`../run-008-local-check/` runs the scripted provider through the real bridge
and driver:

- 3 plans, one per explore phase. Flowers gets an empty plan, which is
  recorded as such.
- Brookside:
  - plan `propose_rule`, `evaluate_rule`, `compare_partition`;
  - executed `propose_rule` ×2, `evaluate_rule`, `begin_forward`,
    `compare_partition`;
  - deviation `begin_forward`, preceded by `h1 on brookside: certified`;
  - 2 consultations with a result and 1 without (the session opened and not
    advanced);
  - 1 finding with verdict `match`.
- Casa Fiesta: plan `evaluate_rule`, which was not executed.
- Calls: 12 harness turns plus 3 plans, 15 in total.

With a fake `_call`, the real provider's message sequence was checked to be
valid: alternating roles, `tool_choice: none` only on planning calls, and on
later phases the pending `tool_result` followed by the planning text.

With this code, the run-006 and run-007 local checks reproduce their reports,
ledger entries, system prompts and tool catalogues exactly.
