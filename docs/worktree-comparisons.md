# Worktree comparisons

The Git repository is rooted at `lobo-lang`; its primary branch is `main`. The initial native/compiler/evaluator
snapshot is tagged `baseline/core-0.1.1`. This tag records the code as it existed when Git was initialized;
it does not reconstruct earlier interpreted-only releases.

The benchmark harness is committed separately. The comparison checkout at `.worktrees/execution-baseline`
uses branch `compare/execution-baseline`, pinned to the harness commit. It retains its own build artifacts
and benchmark reports. Main's `.gitignore` excludes `.worktrees`, `target`, and `dist`.

Run a reproducible baseline:

```sh
cd .worktrees/execution-baseline
python3 scripts/compare.py
```

From the primary checkout, create a separate candidate from that baseline when testing a compiler change:

```sh
git worktree add -b compare/candidate .worktrees/candidate compare/execution-baseline
```

Commit the candidate change, run correctness checks, and execute the same benchmark command and budgets in
both checkouts. The report's source revision identifies exactly what was measured. Record both reports and
compare distributions as well as medians; tiny improvements can fall within system noise.

Useful inspection commands:

```sh
git worktree list
git status --short
git log --oneline --decorate
```

Worktrees are checkouts, not separate performance implementations. Native and interpreted execution already
coexist in each revision, so the three-way comparison runs from one revision to avoid accidental source drift.
Use separate worktrees for alternative compiler implementations, flags, or optimizations.

Worktree removal is optional cleanup after preserving work and reports; do not force removal of dirty checkouts.
No remote or external publication is configured by this setup.
