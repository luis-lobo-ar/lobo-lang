# 0062: Typed library permissions

Date: 2026-11-23. Status: implemented, verification recorded in the linked result.
Supersedes the unimplemented status of [proposal 0061](0061-typed-native-permissions-proposal.md).
The user selected typed permissions and migration of maintained libraries/examples, with TDD, editor support,
documentation, stable IDs and frozen fixtures preserved.

## Contract

Compiler/extension 1.11.2 keep Core 1.11, checked format 1.0, native ABI 1 and SQLite extension 1.
The OS permission module defines Permission cases ClockRead, RandomRead, Listen and Connect.
Its Permissions record stores four booleans. The SQL library has distinct Permission cases Read, Write and
Schema and a three-boolean Permissions record. Every record state is a valid set, including none().
`one(case)` constructs a singleton. `allow(set, case)` returns an immutable set with that permission enabled.
Repeated additions are idempotent. `abi_mask(set)` is the explicit ABI encoder; only adapters use its result
for delegation. No list literals, bitwise operators, new enums, grammar or type inference are introduced.

The alternative was a List<Permission> folded into a set. It adds traversal/allocation and an additional
collection API for domains with four and three members. The fixed record represents exactly the finite
set without invalid bits. A named HTTP profile is complementary: server_permissions() constructs the
clock/listen set in the library that requires it. Application code still owns root acquisition.

## Boundary and compatibility

Native launch grants remain default-deny. Requests never grant rights. Empty delegation is valid; operations
requiring a right still fail. Provider subset checks reject amplification, including delegation from a child.
Sets allocate no native handles and consume no native call quota. The actual restriction still consumes the
same quota and handle slot as before. Action declarations remain exact static bounds, independent of grants.

access.restrict returns Native::CapabilityResult. os.restrict_capability returns IoResult<Capability>.
SQL restrict_native returns the native SQLite result, while SQL restrict returns SqlResult<SqliteCapability>.
Different permission domains cannot be interchanged. Schema alone does not imply Read/Write. Launch profiles
retain their established masks read=1, write=3, schema=7. Adapters encode individual rights 1/2/4.

The old OS restrict_capability ID remains on restrict_mask with its original integer signature and actions.
The old SQL restrict ID likewise remains on restrict_mask. Typed replacements receive new IDs. New import
bindings are explicit; managed authoring sync allocates new SQL/profile IDs once. Existing declarations
retain all IDs. Maintained source audit rejects raw delegation outside the reviewed adapter files.
Frozen fixtures, release artifacts, provider code, models and previously indexed evidence remain unchanged.

## Failure cases and evidence

Wrong integer/case/domain arguments fail type checking. Missing grants and amplification return Denied.
Out-of-range integer masks remain InvalidInput at the raw boundary. Native quota/handle exhaustion retains
LimitExceeded behavior. Permission construction cannot open files or sockets. Runtime cleanup is unchanged.

Tests cover all 16 OS sets, all 8 SQL sets, duplicates, empty requests, native grant combinations, SQL launch
profiles, child attenuation, raw-ID compatibility and C/LLVM/sanitizer execution. The extension uses existing
variant, record, call and import handling with new navigation/diagnostic fixtures and explicit closures.
See [the verification record](../benchmarks/results/2026-11-23-typed-permissions.md).

No performance, token-efficiency or comprehension benefit has been measured. Actual Windows hardware
execution of this patch is unknown. This decision authorizes no provider/model change or further feature growth.
