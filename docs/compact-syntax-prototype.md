# Build the MVC controller with native Lobo syntax

The [compact MVC example](../examples/mvc.compact.lobo) uses concise functions, grouped imports, qualified
calls and a Text-valued JSON object. Compiler 1.7 accepts this file directly and reads its tracked identity sidecar.
The [older MVC example](../examples/mvc.lobo) remains a deprecated-spelling comparison fixture.

## Compile and run the controller

Run these commands from the repository root with compiler 1.7.2 and Clang installed.
Choose a new executable path because the compiler refuses to overwrite output files.

```sh
./target/release/lobo check examples/mvc.compact.lobo \
  --source libraries/http/http.lobo \
  --source libraries/json/json.lobo

./target/release/lobo compile examples/mvc.compact.lobo \
  --source libraries/http/http.lobo \
  --source libraries/json/json.lobo \
  -o target/mvc-controller-demo

./target/mvc-controller-demo \
  --entry example.mvc.show_user --arg I64:1 --fuel 10000 --heap-cells 10000
```

The native executable returns a typed Response with status 200 and body `{"name":"Luis"}`.
Use `--arg I64:42` to obtain status 404 and `Not found` followed by a newline.
The example is a pure controller. The separate Rust web host owns sockets.

To compile and run in one command, use `run`.

```sh
./target/release/lobo run examples/mvc.compact.lobo \
  --source libraries/http/http.lobo \
  --source libraries/json/json.lobo \
  --entry example.mvc.show_user --arg I64:1
```

Use `--backend llvm` to select the optional direct LLVM route when its documented tools are installed.
Both routes emit machine code. Compiler diagnostics refer to the original source.
No Node.js process, prototype marker or projection command is required.

## Share missing-resource responses

The [generic MVC](../examples/mvc.generic.lobo) shares the 404 branch through `respond_json<T>`.
Its controller calls `respond_json(find_user(id), render_user)`. Compiler 1.7 infers User and the named
view reference. The source's tracked metadata remains unchanged during compilation.

```sh
./target/release/lobo check examples/mvc.generic.lobo \
  --source libraries/http/http.lobo --source libraries/json/json.lobo

./target/release/lobo run examples/mvc.generic.lobo \
  --source libraries/http/http.lobo --source libraries/json/json.lobo \
  --entry "$(jq -r '.identities.show_user' examples/mvc.generic.lobo.ids.json)" --arg I64:1
```

The response has status 200 and body `{"name":"Luis"}`. Argument 42 returns status 404.
The model's `Lookup<User>` return type now identifies the constructors, so it writes:

```lobo
fn find_user(id: I64) -> Lookup<User> {
  if id == 1 { Found(User { name: "Luis" }) } else { Missing }
}
```

Compiler 1.7 infers constructor arguments and the case owner. Read
[the constructor rules](../spec/core-v1.5.md) for contexts and ambiguous cases.
The generic example uses a distinct module and tracked IDs. The simpler MVC and frozen legacy comparison
retain their original source. Read [the inference contract](../spec/core-v1.4.md) for explicit-type fallbacks.

## JSON builder dependencies

The current generic MVC selects only `use json { Json, json_response };`. Its view uses:

```lobo
fn render_user(user: User) -> Json {
  json { "name": user.name }
}
```

Compiler 1.7 supplies the `object` and `field` dependencies by stable ID. Source still selects `Json`
and functions it calls. Supply the JSON and HTTP files explicitly; the compiler never discovers libraries.
Generated dependencies remain visible in checked module imports. An empty literal needs only `object`.
Explicit builder imports remain supported for direct calls and existing source. The compact comparison
fixture retains them; the generic MVC removes those two source metadata keys while preserving every other ID.
See [the JSON literal contract](../spec/core-v1.6.md).

## Edit the source

Use extension 1.7.2 for highlighting, Ctrl-click and native compiler squiggles with managed IDs.
The repository's explicit source list selects the compact MVC file plus its libraries.
In another workspace, list those three sources in `lobo.sourceFiles` and configure the compiler path.
Supply either MVC representation in a bundle because both intentionally share declaration IDs.

The MVC's twelve bindings live in `examples/mvc.compact.lobo.ids.json`. Track that file with the source.
Checks and builds never create IDs. For new source, run `lobo identities init FILE` with its explicit
`--source` dependencies. After adding declarations, run `lobo identities sync FILE` with those dependencies.
These authoring commands allocate IDs once and retain existing values. They do not edit source text.

After renaming `find_user` and its references to `lookup_user`, retarget its metadata with:

```sh
lobo identities rename examples/mvc.compact.lobo find_user lookup_user \
  --source libraries/http/http.lobo --source libraries/json/json.lobo
```

The ID `example.mvc.find_user` remains unchanged. Move a source and its sidecar together.
Stale metadata fails instead of guessing a rename. F2 source refactoring is not implemented.
Restore missing identity state from version control for an established module. Initialization cannot recover lost IDs.

Read [the 1.3 reference](../spec/core-v1.3.md) for metadata schemas, explicit snapshots and interrupted-write recovery.
The `json` shorthand accepts Text fields. Use explicit Json constructors for other value types.
Old annotation-based source remains accepted automatically as deprecated read compatibility.
Mixed-source bundles let existing libraries participate without immediate migration.
Write new code in the modern spelling. The old JavaScript tool is an optional compatibility export utility.
