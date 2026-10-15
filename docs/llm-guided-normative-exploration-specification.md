# LLM-guided exploration of normative generalizations

Status: **PROPOSED EXPERIMENT SPECIFICATION**. Repository inspection baseline:
`main` at `7af8d39`, 2026-10-14. This document adds no engine operations,
ontology declarations, candidate rules, learning implementation or legal
valuations. It uses the project's interpretation of recalcitrance; it is not
a literature review.

## 1. Research question and scope

Can an LLM, operating through bounded and independently verified Residuum
tools, construct ontologically admissible generalizations that satisfy
independently grounded normative observations? Does access to additional
inferential and geometric feedback improve that search?

The LLM supplies an exploratory policy:

\[
\pi_{\mathrm{LLM}}: \text{available observations and search history}
\longrightarrow \text{next investigative action}.
\]

Actions may investigate a derivation, construct a candidate generalization,
propose a specialization or generalization, or request assessment of a
discriminating case. Candidate construction is a substantive outcome distinct
from choosing among previously enumerated hypotheses.

Residuum supplies elaboration, executable consequences and derivation records.
The harness must enforce the search contract, retain observations, compare
candidates and apply an explicitly declared decision contract. Neither the
LLM nor successful execution establishes normative authority.

The reference theory remains immutable. Hypothetical evaluation does not
approve a revision or install it in the reference theory.

## 2. Fixed objects and the recalcitrant experience

Each experimental manifest fixes:

- A reference theory `T`, resolved module graph, declaration origins and hashes.
- A typed vocabulary and structural search profile, independent of Riggs.
- Admissible evidence inputs `A`, an observable consequence interface `Q`,
  and bounded comparison domains when exhaustive verification is intended.
- An executable decision contract `D`, its version, and a comparison criterion
  for each external valuation.
- Grounded observations and their source, grounds, scope, historical or
  hypothetical character, and assessment status.
- The model configuration, action permissions, search budgets and condition
  A, B or C.

For a dossier `S`, completed execution returns events and provenance:

\[
\operatorname{Run}_T(S)=(C_T(S),\mathcal P_T(S)),\qquad
N_T(S)=D(C_T(S),\mathcal P_T(S),Q).
\]

Whether `D` uses provenance is a declared choice. It must not acquire that
dependency merely because the explorer can inspect explanations. `D` can
return a structured outcome or a set of consequences; it need not be Boolean.

An experience is:

\[
\varepsilon=(S,Q,N_T(S),y^*(S),\kappa_\varepsilon,m),
\]

where `m` retains the assessment's provenance and `kappa` states how the
executable outcome is compared with the external assessment. Recalcitrance is
a grounded discrepancy, `kappa(N_T(S)) = false`, after a completed and
comparable evaluation. Partial or pending assessments remain pending. External
labels are not inserted as Seeded facts.

A query that returns no explanation, lacks bindings, reaches a step limit or
fails to elaborate supplies computational feedback. These events do not by
themselves constitute recalcitrance. Completion of residual search establishes
its result only within the selected scope, bindings, admissible hypotheses and
supported fragment; it does not establish general residual completeness.

The unknown justification `J*`, a candidate generalization `G_Gamma`, its formal
rules, its hypothetical theory and its assessed normative outcome remain
different objects. No known target justification is constructed for the LLM to
recover. Compatibility with finite assessments is the initial learning claim.

## 3. Search language: configurations from the ontology

The starting contracts are
[the relational search-space contract](legal-relational-search-space-contract.md)
and [its certified structural slice](legal-structural-generation-enumeration.md).
The former supplies signatures and proposed restrictions; the latter also
identifies limits of the original finiteness claim. Its `compliance`-only slice
is a technical control, not the search language of this experiment.

The general vocabulary includes the existing typed performances `give`, `do`
and `notDo`; historical evidence `Act` and `~Act`; and normative qualifications
for obligations, rights, permissions, prohibitions and prohibitions of omission.
The documented Seeded and Derived channels remain distinct. The full reference
theory computes consequences beyond that candidate vocabulary.

Abstractly, an executable configuration retains:

\[
\Gamma=(parameters,rules,RuleUses,module,bindings,installation,origins).
\]

Its components use the existing `Rule`, `Program`, `RuleUse` and typed terms:

```text
Term[T]    ::= typed parameter | declared assignable constant
             | none, if T permits it
             | after(Term[PropositionTime])
             | declared record with correctly typed fields
Time       ::= temporal constant | temporal parameter | after(Time)
Literal    ::= declared verb(Term[T1], ..., Term[Tn]) @ Time
             | permitted evidentially negative literal
Body       ::= I | Literal | Body & Body
Clause     ::= Body <= declared observable/intermediate head
RuleUse    ::= existing or experimental RuleRef + typed partial bindings
Program    ::= shared parameters + bounded RuleUses + defining module
```

This is a harness search grammar, not new Formal syntax. Exact signatures,
categories and permitted polarities come from the elaborated module and the
published profile. The initial historical negative literal is `~Act`; allowing
other negative literals requires an explicit profile, not an implicit default.
Records retain their declared schemas; candidates vary values and bindings.

Structural admission checks declared types, arities, roles as argument
positions, identity links, temporal terms, binding sources for ground outputs,
scope hygiene, references and installation/activation. Nominal types alone do
not establish identity of people or unequal roles. Distinct variables do not
assert inequality. A connectivity restriction, if used, is separately declared
as a search restriction rather than sufficient legal relevance.

No structural filter decides which admissible patterns are sufficient reasons
for the externally valued result. That is what the experiment investigates.
Compiling successfully is necessary but not sufficient for this admission
contract or for completing evaluation.

A problem can supply typed constants and a declared output interface. That
does not permit a Riggs-specific list of antecedent templates, obligatory
homicide/causality/purpose features, or its old six-condition catalogue to define
the general grammar. Problem-specific declarations and additions to the
vocabulary must be identified as such. The pre-existing Riggs impediment is an
experimental output declaration, not a new universal legal primitive.

The grammar must support its declared goals with intermediate heads, rather
than fixing `compliance` as the sole learned consequence. The exact finite
profile must be published before a run and reused across conditions. This
specification does not claim that such a broader generator is implemented.

## 4. Finiteness, search operations and installation

The profile bounds clauses, RuleUses, literals, binders, constructor and record
depth, body-tree nodes, constants, modules and activation plans. Variable names
are canonized consistently for search indexing, with original formulations
retained. Candidate IDs do not create new structural possibilities.

Bounding literals alone is insufficient: arbitrarily nested `I & ... & I`
creates infinitely many source bodies. A finite profile must bound those nodes
or specify a finite body representation, recording the chosen normalization.
It must preserve ordered antecedents and the original candidate text for
provenance. Equal coverage does not authorize discarding distinct formulations.

Under these finite bounds and finite signatures, the syntactic search space
is finite by induction on term and body construction. This does not guarantee
that any candidate's closure terminates: symbolic successors can generate an
unbounded chain.

Possible search moves include adding/removing literals, identifying or
separating binders, specializing constants or constructor patterns, changing
typed bindings, and assembling RuleUses into a Program. Every proposal is
admitted and evaluated anew. These moves are not certified complete minimal
generalization/specialization operators; Candidate Elimination is not assumed.

Each candidate records an installation plan. Ordinary clause addition,
explicit Program compilation and an eligible associated activation use
different existing execution paths. Declaring a Program or importing a module
does not authorize activating its members. Associated guards, hygienic
RuleUses and selected-program isolation must survive evaluation.

The first exploration must declare exactly which existing installation paths
it permits. Adding ordinary consequences is supported; exceptions, retraction,
rule replacement and effective exclusion are not silently interchangeable
with addition. Defaults and `State` execution are outside the initial tool
contract. No change to the selector or effect semantics is authorized here.

## 5. Hypothetical execution and the decision contract

For an admitted candidate:

\[
T_\Gamma=T\oplus\Gamma,\qquad
N_\Gamma(S)=D(C_{T_\Gamma}(S),\mathcal P_{T_\Gamma}(S),Q).
\]

`oplus` records one explicit installation operation, not a generic promise to
revise law. A private candidate module must preserve the reference declarations,
seeds and origins. Hypotheses from residual search enter only a private
sufficiency check, never the actual dossier or reference Known.

Completed evaluations retain event arguments, types, polarities, proposition
times, operational metadata, ordered witnesses, canonical substitutions,
rule/program/module origins and multiplicity of justifications. Logical
`Event` equality alone is insufficient for this audit. Only the concrete
`derivedAt` stamp may vary between independently clocked executions; source
origin differences require their explicit installation mapping.

The unchanged reference execution must be reproducible independently. Candidate
evaluation is compared with that baseline, including effects of activation or
package installation that could otherwise change available reference rules.
Here independent evaluation means a separately executed, isolated check of the
claimed consequence, not agreement with the LLM's prose or a reused trace.
Shared engine helpers mean this is not automatically an independently implemented
semantic oracle or a proof of correctness of the engine itself.

`D` is fixed within an episode and shared across A/B/C. The LLM cannot redefine
its accepted outcome or use provenance as an undeclared priority. If the
proposed operation cannot achieve a valued result under `D`, record that
boundary instead of changing the assessment.

In Riggs, the old additive experiment derives a proposed impediment while
retaining attribution, asset, title and capacity. No current documented
decision contract turns this automatically into effective exclusion. Until
that contract is specified and verified, this is an inferential control, not
certified normative success on the historical exclusion outcome. The historical
assessment must not be weakened to make the marker count as the full remedy.

## 6. Three independent kinds of progress

Each action gets an explicit progress record, allowing multiple kinds or none:

| Kind | Evidence required | Claim licensed |
|---|---|---|
| Inferential | Certified consequence, sufficient support or derivation within a declared scope | Establishes the stated relation under that execution contract. |
| Geometric | Verified coverage inclusion, distinction of hypotheses, frontier or partition relation on a declared domain | Establishes a structural distinction at that scope. |
| Normative | Completed candidate execution and satisfaction of an independently grounded assessment under fixed `D` | Establishes compatibility with that assessment, not legal approval or discovery of `J*`. |

Intermediate steps may establish observed positive consequences. An incomplete
prefix cannot certify non-derivability, a complete minimal frontier, or a
normative comparison requiring the full result. Failed proposals and redundant
observations can achieve no progress; every action need not be credited.

A new partition distinction is geometric progress without necessarily adding
a new legal valuation. A derived impediment is inferential progress without
necessarily excluding an acquisition. A single action can achieve more than
one kind, with separate certificates.

## 7. Generality, partitions and changing candidate universes

For fixed `T`, admissible dossiers `A`, observation interface and installation
semantics, define relational coverage:

\[
Cov(\Gamma)=\{(S,q):S\in A,\ q\in Q_{ground},\ q\in C_{T_\Gamma}(S)\},
\qquad
\Gamma_s\preceq_g\Gamma_g\iff Cov(\Gamma_s)\subseteq Cov(\Gamma_g).
\]

This is a preorder relative to the declared domain. A finite comparison is not
universal subsumption. Coverage-equivalent configurations remain separate
records when their formulations, producers or argument meanings differ. A
structural move is called a generalization/specialization only with the required
certificate or an explicit unverified claim.

For a fixed finite candidate set `H` and completed observation `o`:

\[
\Gamma\equiv_o\Gamma'\iff o(\Gamma)=o(\Gamma').
\]

Order partitions by finer <= coarser. Accumulating observations on that same
set yields:

\[
\pi_{n+1}=\pi_n\wedge\pi_{o_{new}}.
\]

The equality follows from intersection of the observation kernels. It does
not mean the reference law changes. Outcomes, provenance and assessment
compatibility define different observation channels and must be identified.
Resource exhaustion must not masquerade as a negative legal prediction;
partitions of execution statuses are computational partitions.

An external answer selects compatible candidates; it does not automatically
create a finer prediction partition if that case was already evaluated.
Restricting to survivors restricts the existing partitions to their carrier.
Constructing a new candidate changes the carrier itself. The archive records
generation and filtering separately:

\[
\mathcal B_{n+1}=\mathcal B_n\cup\{\text{new candidates}\},\qquad
V_{n+1}=\{\Gamma\in\mathcal B_{n+1}:\Gamma\text{ passes all resolved observations}\}.
\]

Here `B_n` is the candidate archive, distinct from the dossier domain `A`.
Pending evaluations stay in a separately recorded unresolved set. A newly
generated candidate must be checked against the full observation history,
not only the most recent objection. Without generation and with all comparisons
resolved, filtering gives the familiar decreasing version space.

No cross-carrier meet is asserted. Old observations must be extended to new
candidates or remain explicitly incomplete. No lattice quotient, congruence,
convex version interval, anti-exchange preservation or unique justification is
assumed. Candidate geometry is checked on bounded domains where feasible.

## 8. Experimental conditions A, B and C

| Condition | Information presented to the explorer |
|---|---|
| A — Basic | Same typed ontology and search profile; typed queries, event results, binding/type errors, step progress and completion status. |
| B — Provenance | A plus derivation records, ordered witnesses, origins and certified minimal supports within their declared domain. |
| C — Geometric | B plus certified generality comparisons, coverage relations, hypothesis partitions and discriminating cases. |

Minimal supports are already geometric objects. Thus B/A measures the added
provenance-and-support bundle, while C/B measures added order/partition/case
feedback. The design cannot attribute C/A gains to a single isolated operation
or claim B is devoid of geometry.

The same model/version, generation settings, ontology, candidate grammar,
decision contract, valuation access and search budgets apply to all conditions.
Episodes are separate; tool transcripts and hypotheses found in C must not
leak into A or B. Valuations come from the same independent source. For adaptive
case requests, that source must answer consistently across conditions; its
requests and cost are logged. Held-out assessments stay unavailable during search.

The information difference is access to supplied certified feedback. A has
the same ontology and may infer relationships itself. Its results must omit
proof fields rather than merely hiding them in a UI. Detailed source records
and reference facts still available through the common ontology cannot be
described as secret. All conditions have private full audit records that the
explorer cannot read outside its authorized view.

Search budgets include model tokens, investigative actions, engine transitions,
valuation requests and maximum candidate size. Disclose the additional engine
work and tokens needed to compute and present richer feedback. Equal query
counts alone do not establish equal computational cost. Use both a common
opportunity budget and measured actual resource expenditure.

For the initial comparison, evaluations access the same stepwise interface in
all conditions. Geometric computations consume certified completed results;
they are not described as engine steps. A/B/C differ in feedback, not the
underlying execution semantics or decision operation.

## 9. Action and evaluation record

Each action records its episode/condition, history and candidate IDs; chosen
tool and typed inputs; stated rationale and expected discrimination; scope,
bindings and module hashes; execution budget and status; outputs and certificate
references; progress kind with its grounds; and any resulting proposal.

The rationale is the LLM's stated explanation, not a guaranteed transcript of
internal reasoning. The step trace is evaluator computation. Legal provenance,
external valuation provenance and the search action history are separate records.

Each candidate additionally records original source/AST, source of generation,
structural admission checks, activation plan, parent/search-move relationship,
full resolved and unresolved assessment history, and bounded geometric results.

Premise compatibility, external legal assessment and version-space membership
remain separate. `Compatible`, `Incompatible` and `Indeterminate` must state
which comparison they describe. Missing evidence is not treated as negative
evidence. Execution limits yield an unresolved evaluation, not elimination for
normative incompatibility. Structural rejection is recorded as such.

Discriminating cases must be typed and realizable under the declared Seeded
input protocol. A mathematical discriminant requiring a Derived premise is
marked accordingly; it is not silently presented as an admissible historical
dossier. An LLM-proposed hypothetical assessment is identified as hypothetical,
not a historical decision or independently obtained legal answer.
Such an assessment cannot certify the primary normative success criterion.
That requires a grounded assessment supplied independently of the explorer's
candidate proposal. The explorer may generate a case for external assessment
without also supplying the answer against which its candidate is certified.

## 10. Implementation correspondence and pending work

| Object | Existing repository support | Status for this experiment |
|---|---|---|
| Typed module and declaration origins | `src/modules.rs::load_formal_module`, `src/typed.rs`, `src/elab.rs` | Implemented; read-only manifest adapter needed. |
| Rule/Program and partial specialization | `typed::{Rule, Program, RuleUse}`, `programs::{specialize_program, compile_program}` | Implemented components; bounded general search policy remains proposed. |
| Completed program closure and comparison oracle | `derive::{close_program, close_program_reference}` | Implemented; independent bounded comparisons already exist in research tests. |
| Stepwise forward observation | `derive/steps.rs::{ForwardSession, ForwardStep}` | Implemented; explorer tool projection and logging are proposed. |
| Scoped bindings and active residual sessions | `residual/bindings.rs`, `residual/steps.rs::{ResidualSession, ActiveResidualSession}` | Implemented; source and CLI paths are integrated. |
| Minimal supports and extreme bases | `src/bin/residuum/cli/legal_geometry_frontier_tests.rs`, `legal_geometry_extreme_tests.rs` and certificates | Finite harness results; no general public support tool claimed. |
| Structural candidate generation | `legal_geometry_structural_generation_tests.rs::{classify, complete_role_shared_slice_has_structural_certificates}` | Implemented certified slice only; broader generator/validator adapter pending. |
| Generality and semantic equivalence | Relational search-space contract, bounded closure comparisons | Definitions and restricted results; no complete general subsumption service. |
| Observation kernels and meet/join | `legal_geometry_partition_tests.rs::Partition`, conversational/Riggs harness tests | Implemented bounded/private test helpers; general dynamic-carrier service pending. |
| Recalcitrant observations and candidate filtering | Riggs/conversational fixtures, source tables and tests | Existing bounded experiments; generic episode controller pending. |
| Effective exclusion decision `D` for Riggs | `legal-riggs-normative-rule-learning.md` documents additive impediment boundary | Unresolved; cannot certify historical-outcome success yet. |
| LLM policy, A/B/C isolation and search metrics | No implementation established by this specification | Proposed harness work. |

Public query support is documented in
[source bindings](residual-query-source-bindings.md) and
[stepwise evaluation](stepwise-evaluation.md). The CLI supports scoped and active
source queries with `--steps N --trace`; each invocation starts a fresh session.
Rust sessions support continued advances. The explorer must not claim CLI
invocations resume state unless a harness explicitly maintains a session.

Residual queries with defaults are unsupported. Scoped source queries have
conservative acyclic checks; unresolved parameters are not automatically
existentially enumerated. Active completion performs hypothetical certification
through the original full `close_program`; one residual step is therefore not
a guarantee of uniformly small work. Record-order and subtype boundaries in
the binding APIs remain as documented. These are tool-contract limitations,
not reasons to modify the engine during this stage.

## 11. Acceptance criteria and interpretation

Before running an episode, publish the finite search profile, exact action
schemas, budgets, outcome interface, fixed executable `D`, assessment protocol
and experimental conditions. Any missing prerequisite is a reported boundary.

A valid episode must demonstrate all of the following:

1. The original theory and dossiers retain their recorded baseline executions.
2. Every admitted candidate has structural checks and an explicit installation
   path; rejected proposals cannot bypass them through natural-language output.
3. Completed candidate claims have independent executable certificates; limits
   and unresolved bindings are not converted into successful or negative results.
4. Each action identifies inferential, geometric and normative progress separately.
5. A new candidate is assessed against earlier observations; distinct equivalent
   formulations and justification multiplicity remain reconstructible.
6. A/B/C apply the same semantics and budgets with auditable feedback separation.
7. Reported normative success satisfies the declared assessment under `D`, rather
   than merely deriving a marker or providing a plausible explanation.

Compare actions/tokens/transitions to the first independently verified success,
admission rates, surviving distinct candidates, informative distinctions,
coverage of solutions on a separately bounded oracle domain, and outcomes on
held-out independently valued cases. Report failures and censored runs, not
just successful episodes. Count syntactic variants and coverage classes
separately. Minimality and solution coverage claims require their own bounded
certificates, not the LLM's declaration.

Repeated matched episodes are needed to assess a stochastic policy. A single
success is a feasibility result. A favorable C/B comparison supports the value
of the additional feedback bundle under the tested conditions; it does not
identify the unknown justification. Previously documented Riggs and its
catalogue are unsuitable as undisclosed held-out discoveries. Model familiarity
with the historical case must be acknowledged.

## 12. Next specification gates

The immediate open gates, in order, are:

1. Instantiate the reusable finite grammar and complete structural admission
   manifest without a Riggs-specific feature catalogue or `compliance`-only head.
2. Establish a fixed decision interface capable of expressing the assessed
   alternative outcome, preserving the original inference theory.
3. Define bounded tool adapters and feedback projections for existing sessions
   and the required harness computations; specify exact failure statuses.
4. Fix independent valuation and held-out protocols, budgets and audit schemas.
5. Implement and verify the smallest episode controller only after those
   contracts are reviewable. No such implementation is authorized by this document.

No Candidate Elimination, new ontology semantics, automatic normative approval,
general residual completeness, global anti-exchange theorem or convergence to
`J*` is claimed. Future compilations/tests must use individual selections through
`python3 scripts/test_safe.py`, retaining 1 GiB, zero swap, one Cargo job and one
harness thread. This documentation stage runs no Rust compilation or tests and
does not change `main`, `v1`, the engine or existing research artifacts.
