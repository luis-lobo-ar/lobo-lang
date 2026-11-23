# Lobo research record

Review date: **2026-11-03**. This is a targeted scoping review, not a systematic literature review.
It supports a future paper; it does not establish novelty, replicate the cited experiments, or
demonstrate that Lobo reduces agent cost.

- [Related work](related-work.md): 26 annotated works, their relevance and evidential limits.
- [Paper plan](paper-plan.md): research questions, existing evidence and proposed experiments.
- [Bibliography](references.bib): working, version-specific BibTeX.
- [Source manifest](sources.json): search queries, selection, review depth and access limitations.
- [Primary metadata](catalog.json): authors, titles, versions, dates, source URLs and metadata hashes.
- [Verification record](verification.json): commands, outcomes and validation scope.
- [Local result register](../benchmarks/results/README.md): our experiments, including negative outcomes.

Start with Tokenmaxxing, TypeFly/MiniSpec, Typed Holes, SWE-agent and CodeAct. Read Synoema as close
design prior art with unverified evaluation claims. Then compare the positive and negative findings
on skills and repository instructions, and the LSP-versus-grep preliminary study. Type-constrained
generation supplies another concrete compiler/interface mechanism. These works address different layers: application language,
agent action interface, context retrieval, verification and model-serving runtime.

## Search and review method

We searched the public web for agent-oriented programming languages, language/token costs, semantic
context retrieval, coding-agent interfaces, skills, verification and evaluation methodology.
Selection favored direct relevance and primary papers, proceedings, author artifacts and institutional
records. Aggregators supplied discovery leads, not numerical evidence. We did not enumerate every
database, screen a complete citation graph, assess every full paper or preregister inclusion rules.
Historical program synthesis, language workbenches and incremental build systems need a further pass
before making broad novelty claims.

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

The standard-library collector fetches primary metadata and rewrites the catalog and BibTeX. It retains
retrieval failures and exits nonzero if any occur. Public HTML/JSON and the inspected Synoema PDF are
cached under ignored `research/.cache/`; this cache is not a committed or archival copy. The manifest
also retains access failures, including ACM access restrictions and an OpenReview verification page.
Rerunning metadata collection does not update the prose review or establish that changed papers were read.
Update the manifest's review date/depth deliberately when conducting another review.

BibTeX entries are working `@misc` records for the versions actually reviewed. For arXiv entries, the
year is the first submission year, which may precede the publication year. The note records a verified
venue where available. Before submission, obtain publisher BibTeX, resolve author-list discrepancies,
check licensing and archive the exact public artifacts needed to reproduce claims.

## Verification of this review

All 26 primary metadata records were retrieved successfully. Bibliography/catalog keys, version pins,
JSON syntax and local documentation links were checked. `cargo test --locked` also passed all 145
tests, including native/reference and sanitizer checks; no compiler code changed. No new model trials
were run. The [verification record](verification.json) and [ledger](../CHANGELOG.md) retain the outcome
and its limits. Bibliography structure checks do not substitute for a publisher-ready BibTeX build.
