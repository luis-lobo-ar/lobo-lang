# Shared missing-resource handling in MVC

Date 2026-11-22. Verified with the existing compiler 1.3.0. No language, compiler, library or editor behavior changes.

A generic `Lookup<T> { Missing, Found(T) }` and `respond_json<T>` adapter share missing-to-404 handling across
controllers. The adapter takes a pure `Fn(T) -> Json` renderer, called only in the Found branch. Models return
lookup values independently of HTTP. Controllers supply a concrete type and named view function.

The executable probe uses User and Article records with different renderers. Eight cases pass across C and
direct LLVM: found user, missing user, found article, and a missing value with a renderer whose precondition
always fails. The last case confirms that a missing result does not invoke its renderer. Substituting an
Article renderer for a User lookup fails with `compile.type.mismatch`, as independently expected.

Source, explicit metadata, driver, native binaries, compiler identity and exact observations are preserved
under `benchmarks/private-evidence/generic-mvc-20261122`. The public [result data](2026-11-22-generic-mvc.json)
retains source/compiler hashes and all observations. The command was
`python3 target/generic-mvc-probe-20261122/check.py`, exit 0. The driver requires fresh native output paths.
The existing helper library admits response statuses 200, 404 and 405 only. This probe adds no error taxonomy,
middleware, network operations or new response statuses. Full compiler/editor suites were not rerun for this
isolated demonstration of existing behavior. No agent-quality, comprehension, token or speed effect was measured.
