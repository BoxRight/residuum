"""Python controller of the LLM-guided normative exploration (invocation
protocol §1-§13). It owns the episode manifest, identifiers, budgets and
the append-only ledger; it validates every model action, dispatches it to
an injectable backend that speaks the worker's wire protocol, keeps the full
audit and delivers only the fields of the episode's condition. It computes
nothing semantic: no matching, closure, search or frontier lives here.
"""
