# Lobo research record

Review date: **2026-11-03**. This is a targeted scoping review, not a systematic literature review.
It supports a future paper; it does not establish novelty, replicate the cited experiments, or
demonstrate that Lobo reduces agent cost.

- [Native SQLite implementation](../benchmarks/results/2026-11-23-sqlite-driver.md): pinned engine, typed SQL, separate authority, lifecycle verification and Linux/Wine native applications. No measured agent-effect claim.
- [Database access proposal](../docs/database-access-proposal.md): SQLite-first SQL adapter recommendation, separate storage families and test-first gates; [eight primary sources](database-access-2026-11-23.json). Historical proposal; superseded for SQLite by decision 0055.
- [Maintained Core 1.8 source migration](../benchmarks/results/2026-11-22-modernize-source.md): current constructs, IDs, native equivalence, editor checks and preserved failures; no agent-effect measurement.
- [Standalone Windows snapshot](../benchmarks/results/2026-11-22-windows-snapshot.md): bounded Windows profile, actual PE execution under Wine and explicit hardware/formal-coverage limits.
- [Native formal-verification results](../benchmarks/results/2026-11-22-native-formal-verification.md): two finite models, deliberate model/C defects, and a reproduced pending-descriptor cleanup fix.
- [Native state-machine/TLA+ proposal](native-formal-verification-plan-2026-11-22.md): two bounded verification targets and acceptance gates; no formal model has been checked.
- [Native boundary implementation](../benchmarks/results/2026-11-22-native-boundary.md): compiler 1.7 closed OS slice, borrowed-test provenance, adversarial findings and explicit limits.
- [Native library and OS interface review](native-interop-2026-11-22.md): primary-documentation comparison of TIOBE November 2026
  top ten and an unimplemented Lobo boundary proposal, with [sources and limits](native-interop-2026-11-22.json).
- [Related work](related-work.md): 36 annotated papers and artifacts, their relevance and evidential limits.
- [Local paper evidence](local-evidence.md): claim-to-evidence map, recursive/generic/LLVM results and limits,
  with a [hashed register](local-evidence.json) and [preservation receipt](verification-local-evidence.json).
- [Lobo comparison](lobo-comparison.md): current cross-language observations and explicitly unknown agent outcomes.
- [Comparison readiness](comparison-readiness.md): which comparisons fit Core 0.9 and which language gaps need development workloads first.
- [Code comprehension](code-comprehension.md): prediction, dependencies, localization, impact and executable grading.
- [Construction quality](construction-quality.md): outcome measures, existing-language constructs and proposed comparisons.
- [Three hypotheses](hypotheses.md): checked representations, task-aligned abstractions and semantic feedback; H1 recommended as the lead.
- [Findings register](findings.json): eight selected external results/hypotheses, with conditions and limitations.
- [Paper plan](paper-plan.md): research questions, existing evidence and proposed experiments.
- [Bibliography](references.bib): working, version-specific BibTeX.
- [Source manifest](sources.json): search queries, selection, review depth and access limitations.
- [Primary metadata](catalog.json): authors, titles, versions, dates, source URLs and metadata hashes.
- [Verification record](verification.json): commands, outcomes and validation scope.
- [Construction-quality follow-up checks](verification-construction-quality.json): expanded collection and source/result consistency.
- [Comprehension and calibration checks](verification-comprehension.json): 36-source collection, executable observations and grader verification.
- [Initial-review snapshot](snapshots/2026-11-03-initial-review): the original 26-source catalog, bibliography, verification and framing.
- [Construction-quality snapshot](snapshots/2026-11-03-construction-quality): the preceding 33-source collection and findings.
- [Local result register](../benchmarks/results/README.md): our experiments, including negative outcomes.

Start with Tokenmaxxing, TypeFly/MiniSpec, Typed Holes, SWE-agent and CodeAct. Read Synoema as close
design prior art with unverified evaluation claims. Then compare the positive and negative findings
on skills and repository instructions, and the LSP-versus-grep preliminary study. Type-constrained
generation supplies another concrete compiler/interface mechanism. These works address different layers: application language,
agent action interface, context retrieval, verification and model-serving runtime.

## Search and review method

We searched the public web for agent-oriented programming languages, language/token costs, semantic
context retrieval, coding-agent interfaces, skills, verification and evaluation methodology. The
follow-up adds language constructs, software quality, matched requirements and formal guarantees.
Selection favored direct relevance and primary papers, proceedings, author artifacts and institutional
records. Aggregators supplied discovery leads, not numerical evidence. We did not enumerate every
database, screen a complete citation graph, assess every full paper or preregister inclusion rules.
Historical program synthesis, language workbenches and incremental build systems need a further pass
before making broad novelty claims.

Further discovery leads include AILANG and Aver, named by VeraBench, and the older specification,
refinement-type and behavior-system work around Eiffel, F*, SPARK/Ada, Idris and Koka. They have not
received source-level review in this collection. Do not treat that list as additional reviewed evidence.

The manifest distinguishes selected full-text review from abstract/metadata review. Most papers have
not been read end to end. None were independently replicated. Publication status is recorded separately
from availability: an arXiv posting or Zenodo DOI does not establish peer review. The ETH AGENTS.md paper
is an ICLR workshop paper, not a main-conference paper.

Versions matter. SkillsBench v1/v4 and SWE-Skills-Bench v1/v2 differ substantially. TypeFly's later
journal title and evaluation must not be silently merged with the arXiv version reviewed here.
Synoema's selected-task reduction must not substitute for its revised aggregate source-token table.

## Metadata reproduction

From the repository root:

```sh
python3 research/collect_metadata.py
```

The standard-library collector fetches arXiv/Zenodo metadata and rewrites the catalog and BibTeX.
Institutional, formal-paper and pinned software records are curated manually, with author source
links. The collector carries those fields forward; it does not independently fetch/validate them.
It retains retrieval failures and exits nonzero if any occur. Public HTML/JSON and the Synoema PDF are
cached under ignored `research/.cache/`; this cache is not a committed or archival copy. The manifest
also retains access failures, including ACM access restrictions and an OpenReview verification page.
Rerunning metadata collection does not update the prose review or establish that changed papers were read.
Update the manifest's review date/depth deliberately when conducting another review.

Vera artifact READMEs are pinned by commit and SHA-256 in the manifest. Their bytes were checked
locally. We did not obtain their ignored raw benchmark attempts or run their toolchains. Claims about
absence from proprietary training sets remain unverified. Preserve old source and result snapshots
when extending the collection. Append corrections and resulting decisions rather than erase history.

BibTeX entries are working `@misc` records for the versions actually reviewed. For arXiv entries, the
year is the first submission year, which may precede the publication year. The note records a verified
venue where available. Before submission, obtain publisher BibTeX, resolve author-list discrepancies,
check licensing and archive the exact public artifacts needed to reproduce claims.

## Verification of this review

The expanded catalog contains 36 records: 29 arXiv records and one Zenodo record were fetched
successfully, and six author/institutional/publisher records are manually curated. Key/version consistency,
findings-source references, JSON and local links were checked. The initial review passed all 145
tests with `cargo test --locked`. No compiler code changed afterward, so those tests were not repeated
for this documentation follow-up. No new model trials were run. Verification records and the
[ledger](../CHANGELOG.md) retain outcomes and limits. Structural bibliography checks do not substitute
for a publisher-ready BibTeX build.

The initial summary's wording about all 26 metadata records being retrieved included one manually
curated institutional record. The actual initial network collection fetched 25 records; the snapshot
and follow-up verification retain this clarification without rewriting the original evidence.
