use lobo_core::{compile_bundle, Artifact, SourceFile, Value as LoboValue};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

mod process;
use process::Workspace;

type ToolResult<T> = Result<T, Failure>;
struct Failure {
    code: &'static str,
    message: String,
}
impl Failure {
    fn new(code: &'static str, message: impl ToString) -> Self {
        Self {
            code,
            message: message.to_string(),
        }
    }
    fn json(self) -> Value {
        if self.code == "candidate.compile" {
            return json!({"status":"compile_error","code":self.code,"diagnostic":serde_json::from_str::<Value>(&self.message).unwrap_or(Value::Null)});
        }
        json!({"status":if self.code.starts_with("tool.") {"infrastructure_error"} else if self.code.starts_with("test.") {"invalid_candidate"} else {"invalid_request"},"code":self.code,"message":self.message})
    }
}

#[derive(Deserialize)]
#[serde(tag = "op", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    Describe {},
    Open {
        root: String,
        sources: Vec<SourceFile>,
        requirements: Vec<Requirement>,
        tests: Vec<TestCase>,
    },
    Verify {
        base: String,
        root: String,
        sources: Vec<SourceFile>,
        environment: String,
        scope: Scope,
        max_cases: usize,
        wall_ms: u64,
    },
    Explain {
        snapshot: String,
        test: String,
    },
    Replay {
        snapshot: String,
        test: String,
    },
    Close {},
}

#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Affected,
    All,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    id: String,
    description: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TestCase {
    id: String,
    requirement: String,
    entry: String,
    arguments: Vec<String>,
    expect: Expectation,
    fuel: u64,
    heap_cells: u64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Expectation {
    Value {
        value: Value,
    },
    Fault {
        code: String,
        phase: String,
        subject: String,
    },
}

struct Plan {
    requirements: Vec<Requirement>,
    tests: Vec<TestCase>,
}
struct Snapshot {
    root: String,
    sources: Vec<SourceFile>,
    artifact: Artifact,
    canonical: Value,
    environment: String,
    results: Vec<Option<Evidence>>,
    binary: Option<PathBuf>,
}

#[derive(Clone)]
struct Evidence {
    status: String,
    observed: Value,
    origin: String,
}

pub struct Session {
    clang: PathBuf,
    plan: Option<Plan>,
    snapshots: Vec<Snapshot>,
    workspace: Option<Workspace>,
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_.:-".contains(&b))
}

fn canonical(artifact: &Artifact) -> Value {
    serde_json::to_value(artifact).expect("checked artifact serializes")
}

fn check_arguments(artifact: &Artifact, tests: &[TestCase]) -> ToolResult<()> {
    let checked = canonical(artifact);
    for test in tests {
        if !artifact.entries().contains(&test.entry.as_str()) {
            return Err(Failure::new("test.entry", &test.entry));
        }
        let arguments = artifact
            .parse_arguments(
                &test.entry,
                &test.arguments.iter().map(String::as_str).collect::<Vec<_>>(),
            )
            .map_err(|e| Failure::new("test.arguments", e.code))?;
        let function = checked["program"]["functions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|function| function["id"] == test.entry)
            .unwrap();
        let parameters = function["parameters"].as_array().unwrap();
        if arguments.len() != parameters.len() {
            return Err(Failure::new(
                "test.arguments",
                "argument arity does not match entry",
            ));
        }
        for (argument, parameter) in arguments.iter().zip(parameters) {
            let ty = match argument {
                LoboValue::F64(_) => json!("F64"),
                LoboValue::Function(value) => json!({"Function":value.signature}),
                LoboValue::I64(_) => json!("I64"),
                LoboValue::Bool(_) => json!("Bool"),
                LoboValue::Text(_) => json!("Text"),
                LoboValue::Bytes(_) => json!("Bytes"),
                LoboValue::Record(value) => json!({"Record":value.id}),
                LoboValue::Variant(value) => json!({"Variant":value.id}),
                LoboValue::List(value) => json!({"List":value.element}),
            };
            if ty != parameter["ty"] {
                return Err(Failure::new(
                    "test.arguments",
                    "argument type does not match entry",
                ));
            }
        }
    }
    Ok(())
}

fn compile(root: &str, sources: &[SourceFile]) -> ToolResult<Artifact> {
    compile_bundle(root, sources)
        .map_err(|e| Failure::new("candidate.compile", serde_json::to_string(&e).unwrap()))
}

fn dependency_key(program: &Value, entry: &str) -> Option<Value> {
    let functions: BTreeMap<_, _> = program["program"]["functions"]
        .as_array()?
        .iter()
        .map(|f| Some((f["id"].as_str()?, f)))
        .collect::<Option<_>>()?;
    fn calls(value: &Value, found: &mut Vec<String>) -> Option<()> {
        match value {
            Value::Object(object) => {
                if let Some(kind) = object.get("kind").and_then(Value::as_str) {
                    if ![
                        "NativeOp", "BufferOp", "List", "ListOp", "Variant", "Match", "Record", "Field",
                        "Literal", "Name", "Unary", "Binary", "Call", "If", "Fold", "While",
                    ]
                    .contains(&kind)
                    {
                        return None;
                    }
                    if kind == "Call" {
                        found.push(object.get("function")?.as_str()?.into());
                    }
                }
                for child in object.values() {
                    calls(child, found)?;
                }
            }
            Value::Array(array) => {
                for child in array {
                    calls(child, found)?;
                }
            }
            _ => {}
        }
        Some(())
    }
    let mut pending = vec![entry.to_string()];
    let mut closure = BTreeMap::new();
    while let Some(id) = pending.pop() {
        if closure.contains_key(&id) {
            continue;
        }
        let function = *functions.get(id.as_str())?;
        if !function["actions"].as_array()?.is_empty() {
            return None;
        }
        calls(function, &mut pending)?;
        closure.insert(id, function.clone());
    }
    Some(
        json!({"functions":closure,"records":program["program"]["records"],"variants":program["program"]["variants"],"language":program["language"],"format":program["format"]}),
    )
}

fn preview(value: &Value) -> Value {
    let text = value.to_string();
    if text.len() <= 512 {
        return value.clone();
    }
    json!({"preview":text.chars().take(256).collect::<String>(),"bytes":text.len(),"truncated":true})
}

fn difference(expected: &Value, actual: &Value, path: String) -> Option<Value> {
    if expected == actual {
        return None;
    }
    match (expected, actual) {
        (Value::Object(left), Value::Object(right)) if left.keys().eq(right.keys()) => {
            for (key, value) in left {
                if let Some(found) = difference(value, &right[key], format!("{path}.{key}")) {
                    return Some(found);
                }
            }
        }
        (Value::Array(left), Value::Array(right)) if left.len() == right.len() => {
            for (index, (a, b)) in left.iter().zip(right).enumerate() {
                if let Some(found) = difference(a, b, format!("{path}[{index}]")) {
                    return Some(found);
                }
            }
        }
        _ => {}
    }
    Some(json!({"path":path,"expected":preview(expected),"actual":preview(actual)}))
}

fn classify(test: &TestCase, actual: &Value) -> String {
    if actual["kind"] == "resource_exhausted" {
        return "resource_exhausted".into();
    }
    let passed = match &test.expect {
        Expectation::Value { value } => {
            actual["status"] == "ok" && actual["kind"] == "value" && actual["value"] == *value
        }
        Expectation::Fault { code, phase, subject } => {
            actual["kind"] == "runtime_fault"
                && actual["diagnostic"]["code"] == *code
                && actual["diagnostic"]["phase"] == *phase
                && actual["diagnostic"]["subject"] == *subject
        }
    };
    if passed {
        "passed"
    } else if actual["kind"] == "runtime_fault" {
        "runtime_fault"
    } else {
        "failed"
    }
    .into()
}

impl Session {
    pub fn new(clang: PathBuf) -> Self {
        Self {
            clang,
            plan: None,
            snapshots: vec![],
            workspace: None,
        }
    }

    pub fn apply(&mut self, action: Action) -> Value {
        let result = match action {
            Action::Describe {} => Ok(
                json!({"status":"ok","actions":["describe","open","verify","explain","replay","close"],"execution":"native","snapshot_scope":"this foreground session","compiler":env!("CARGO_PKG_VERSION"),"language":lobo_core::LANGUAGE,
                "limits":{"request_bytes":4194304,"response_bytes":8388608,"sources":16,"source_bytes":1048576,"source_tokens":8192,"requirements":64,"snapshots":8,"tests":256,"child_output_bytes_per_stream":1048576,"observation_bytes_per_snapshot":4194304},
                "contracts":{
                    "open":"{op:open, root:string, sources:[{name:string,text:string}], requirements:[{id:string,description:string}], tests:[test]}; pins criteria, checks complete source and invocation shapes; no execution yet",
                    "test":"{id:string,requirement:string,entry:stable_function_id,arguments:[typed_CLI_atom],expect:{kind:value,value:typed_JSON}|{kind:fault,code:runtime_code,phase:execute|precondition|postcondition,subject:function_id},fuel:1..10000000,heap_cells:0..1000000}",
                    "verify":"{op:verify,base:snapshot_id,root:string,sources:[{name:string,text:string}],environment:string,scope:affected|all,max_cases:0..256,wall_ms:1..30000}; environment is host-declared immutable identity; empty disables reuse; all always executes; pending/uncovered is never acceptance",
                    "detail":"{op:explain|replay,snapshot:snapshot_id,test:test_id}; explain returns exact source/check/outcome; replay executes retained code and compares observation, not historical OS state",
                    "close":"{op:close} or EOF; terminates foreground session and removes only its temporary artifacts; handles cannot be reused in another session"
                },
                "workflow":"open freezes requirements/checks; verify an explicit base and candidate; explain/replay by snapshot and test; full scope is a separate gate; close or EOF cleans temporary files"}),
            ),
            Action::Open {
                root,
                sources,
                requirements,
                tests,
            } => self.open(root, sources, requirements, tests),
            Action::Verify {
                base,
                root,
                sources,
                environment,
                scope,
                max_cases,
                wall_ms,
            } => self.verify(base, root, sources, environment, scope, max_cases, wall_ms),
            Action::Explain { snapshot, test } => self.explain(&snapshot, &test),
            Action::Replay { snapshot, test } => self.replay(&snapshot, &test),
            Action::Close {} => Ok(json!({"status":"closed"})),
        };
        result.unwrap_or_else(Failure::json)
    }

    fn open(
        &mut self,
        root: String,
        sources: Vec<SourceFile>,
        requirements: Vec<Requirement>,
        mut tests: Vec<TestCase>,
    ) -> ToolResult<Value> {
        if self.plan.is_some() {
            return Err(Failure::new(
                "plan.frozen",
                "start a separately identified session to change acceptance checks",
            ));
        }
        if requirements.is_empty() || requirements.len() > 64 || tests.is_empty() || tests.len() > 256 {
            return Err(Failure::new(
                "plan.limit",
                "need 1..64 requirements and 1..256 tests",
            ));
        }
        let mut ids = BTreeSet::new();
        for requirement in &requirements {
            if !valid_id(&requirement.id)
                || requirement.description.is_empty()
                || requirement.description.len() > 1024
                || !ids.insert(&requirement.id)
            {
                return Err(Failure::new(
                    "plan.requirement",
                    "invalid/duplicate requirement identity or description",
                ));
            }
        }
        let mut names = BTreeSet::new();
        for test in &mut tests {
            if !valid_id(&test.id)
                || !valid_id(&test.entry)
                || !ids.contains(&test.requirement)
                || !names.insert(test.id.clone())
                || test.fuel == 0
                || test.fuel > 10_000_000
                || test.heap_cells > 1_000_000
                || test.arguments.len() > 64
                || test
                    .arguments
                    .iter()
                    .any(|arg| arg.len() > 32768 || arg.contains('\0'))
                || test.arguments.iter().map(String::len).sum::<usize>() > 65536
            {
                return Err(Failure::new(
                    "plan.test",
                    "invalid/duplicate test, requirement reference, arguments or budgets",
                ));
            }
            match &mut test.expect {
                Expectation::Value { value } => {
                    if value.to_string().len() > 65536 {
                        return Err(Failure::new("plan.expectation", "expected value exceeds 64 KiB"));
                    }
                    let typed: LoboValue = serde_json::from_value(value.clone())
                        .map_err(|e| Failure::new("plan.expectation", e))?;
                    *value = serde_json::to_value(typed).unwrap();
                }
                Expectation::Fault { code, phase, subject }
                    if code.starts_with("runtime.")
                        && valid_id(code)
                        && valid_id(subject)
                        && ["execute", "precondition", "postcondition"].contains(&phase.as_str()) => {}
                _ => {
                    return Err(Failure::new(
                        "plan.expectation",
                        "only typed values or explicitly located runtime faults may be expected",
                    ))
                }
            }
        }
        let artifact = compile(&root, &sources)?;
        check_arguments(&artifact, &tests)?;
        let snapshot = Snapshot {
            root,
            sources,
            canonical: canonical(&artifact),
            artifact,
            environment: String::new(),
            results: vec![None; tests.len()],
            binary: None,
        };
        self.plan = Some(Plan { requirements, tests });
        self.snapshots.push(snapshot);
        Ok(
            json!({"status":"opened","snapshot":"s1","checks":self.plan.as_ref().unwrap().tests.len(),"criteria":"frozen for this session; not external approval"}),
        )
    }

    fn index(&self, id: &str) -> ToolResult<usize> {
        id.strip_prefix('s')
            .and_then(|n| n.parse::<usize>().ok())
            .and_then(|n| n.checked_sub(1))
            .filter(|n| *n < self.snapshots.len())
            .ok_or_else(|| Failure::new("snapshot.unknown", "snapshot is not retained in this session"))
    }

    fn case_index(&self, id: &str) -> ToolResult<usize> {
        self.plan
            .as_ref()
            .and_then(|plan| plan.tests.iter().position(|test| test.id == id))
            .ok_or_else(|| Failure::new("test.unknown", "test is not in the frozen plan"))
    }

    fn workspace(&mut self) -> ToolResult<&mut Workspace> {
        if self.workspace.is_none() {
            self.workspace = Some(Workspace::new()?);
        }
        Ok(self.workspace.as_mut().unwrap())
    }

    fn binary(&mut self, index: usize, deadline: Instant) -> ToolResult<PathBuf> {
        if let Some(binary) = &self.snapshots[index].binary {
            return Ok(binary.clone());
        }
        let code = self.snapshots[index]
            .artifact
            .emit_c()
            .map_err(|e| Failure::new("tool.lowering", e.code))?;
        let clang = self.clang.clone();
        let binary = self.workspace()?.build(&clang, &code, deadline)?;
        self.snapshots[index].binary = Some(binary.clone());
        Ok(binary)
    }

    fn execute(&mut self, index: usize, case: usize, deadline: Instant) -> ToolResult<Evidence> {
        let binary = self.binary(index, deadline)?;
        let test = self.plan.as_ref().unwrap().tests[case].clone();
        let mut command = Command::new(binary);
        command.env_clear().env("LC_ALL", "C").args([
            "--entry",
            &test.entry,
            "--fuel",
            &test.fuel.to_string(),
            "--heap-cells",
            &test.heap_cells.to_string(),
        ]);
        for argument in &test.arguments {
            command.args(["--arg", argument]);
        }
        let output = self
            .workspace()?
            .capture(command, deadline.min(Instant::now() + Duration::from_secs(1)))?;
        if !output.stderr.is_empty() {
            return Err(Failure::new(
                "tool.native_stderr",
                "native execution wrote unexpected stderr",
            ));
        }
        let observed: Value =
            serde_json::from_slice(&output.stdout).map_err(|e| Failure::new("tool.native_json", e))?;
        if output.success != (observed["status"] == "ok")
            || !["value", "runtime_fault", "resource_exhausted"]
                .iter()
                .any(|kind| observed["kind"] == *kind)
        {
            return Err(Failure::new(
                "tool.native_protocol",
                "unexpected native exit/envelope",
            ));
        }
        Ok(Evidence {
            status: classify(&test, &observed),
            observed,
            origin: format!("s{}", index + 1),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn verify(
        &mut self,
        base: String,
        root: String,
        sources: Vec<SourceFile>,
        environment: String,
        scope: Scope,
        max_cases: usize,
        wall_ms: u64,
    ) -> ToolResult<Value> {
        let started = Instant::now();
        let base_index = self.index(&base)?;
        if self.snapshots.len() >= 8
            || max_cases > 256
            || wall_ms == 0
            || wall_ms > 30000
            || environment.len() > 128
        {
            return Err(Failure::new(
                "request.limit",
                "snapshot/case/time/environment limit exceeded",
            ));
        }
        let artifact = compile(&root, &sources)?;
        let plan = self.plan.as_ref().unwrap();
        check_arguments(&artifact, &plan.tests)?;
        let checked = canonical(&artifact);
        let previous = &self.snapshots[base_index];
        let mut reused = BTreeSet::new();
        let mut results = vec![None; plan.tests.len()];
        let mut retained_bytes = 0usize;
        for (i, test) in plan.tests.iter().enumerate() {
            if scope == Scope::All || environment.is_empty() || previous.environment != environment {
                continue;
            }
            let Some(old) = &previous.results[i] else {
                continue;
            };
            let key = dependency_key(&checked, &test.entry);
            if old.status == "passed"
                && key.is_some()
                && key == dependency_key(&previous.canonical, &test.entry)
            {
                let size = old.observed.to_string().len();
                if retained_bytes + size > 4 * 1024 * 1024 {
                    continue;
                }
                results[i] = Some(old.clone());
                reused.insert(i);
                retained_bytes += size;
            }
        }
        let index = self.snapshots.len();
        self.snapshots.push(Snapshot {
            root,
            sources,
            artifact,
            canonical: checked,
            environment,
            results,
            binary: None,
        });
        let deadline = started + Duration::from_millis(wall_ms);
        let mut executed = 0;
        let mut result_bytes = self.snapshots[index]
            .results
            .iter()
            .flatten()
            .map(|e| e.observed.to_string().len())
            .sum::<usize>();
        for i in 0..self.plan.as_ref().unwrap().tests.len() {
            if reused.contains(&i) {
                continue;
            }
            if executed >= max_cases || Instant::now() >= deadline || result_bytes > 4 * 1024 * 1024 {
                break;
            }
            let evidence = match self.execute(index, i, deadline) {
                Ok(evidence) => {
                    executed += 1;
                    evidence
                }
                Err(e) => {
                    self.snapshots[index].results[i] = Some(Evidence {
                        status: "infrastructure_error".into(),
                        observed: json!({"code":e.code,"message":e.message}),
                        origin: format!("s{}", index + 1),
                    });
                    break;
                }
            };
            let size = evidence.observed.to_string().len();
            if result_bytes + size > 4 * 1024 * 1024 {
                self.snapshots[index].results[i] = Some(Evidence {
                    status: "infrastructure_error".into(),
                    observed: json!({"code":"tool.observation_limit","message":"snapshot observation budget exhausted; remaining tests are pending"}),
                    origin: format!("s{}", index + 1),
                });
                break;
            }
            result_bytes += size;
            self.snapshots[index].results[i] = Some(evidence);
        }
        Ok(self.report(
            index,
            executed,
            &reused,
            started.elapsed().as_millis(),
            scope == Scope::All,
        ))
    }

    fn report(
        &self,
        index: usize,
        executed: usize,
        reused: &BTreeSet<usize>,
        elapsed: u128,
        full: bool,
    ) -> Value {
        let plan = self.plan.as_ref().unwrap();
        let snapshot = &self.snapshots[index];
        let pending = snapshot.results.iter().filter(|value| value.is_none()).count();
        let passed = snapshot
            .results
            .iter()
            .filter(|value| value.as_ref().is_some_and(|e| e.status == "passed"))
            .count();
        let failed = snapshot.results.len() - pending - passed;
        let uncovered = plan
            .requirements
            .iter()
            .filter(|req| !plan.tests.iter().any(|test| test.requirement == req.id))
            .map(|r| &r.id)
            .collect::<Vec<_>>();
        let failures = snapshot.results.iter().enumerate().filter_map(|(i, value)| {
            let evidence = value.as_ref()?;
            if evidence.status == "passed" { return None; }
            let test = &plan.tests[i];
            let mut record = json!({"test":test.id,"requirement":test.requirement,"status":evidence.status,"replay":{"snapshot":format!("s{}",index+1),"test":test.id}});
            if let Expectation::Value { value } = &test.expect {
                if evidence.observed["kind"] == "value" { record["difference"] = difference(value, &evidence.observed["value"], "$".into()).unwrap_or(Value::Null); }
                else { record["detail"] = preview(&evidence.observed); }
            } else { record["detail"] = preview(&evidence.observed); }
            Some(record)
        }).take(8).collect::<Vec<_>>();
        let pending_tests = snapshot
            .results
            .iter()
            .enumerate()
            .filter(|(_, evidence)| evidence.is_none())
            .map(|(i, _)| &plan.tests[i].id)
            .take(8)
            .collect::<Vec<_>>();
        json!({"status":if failed>0 {"failed"} else if pending>0 || !uncovered.is_empty() {"incomplete"} else {"checks_satisfied"},
            "snapshot":format!("s{}",index+1),"counts":{"executed":executed,"reused":reused.len(),"passed":passed,"failed":failed,"pending":pending},
            "full_execution":full && executed==plan.tests.len() && pending==0 && reused.is_empty(),"uncovered_requirements":uncovered,"pending_tests":pending_tests,"pending_omitted":pending.saturating_sub(8),"failures":failures,"failures_omitted":failed.saturating_sub(8),"elapsed_ms":elapsed})
    }

    fn explain(&self, snapshot: &str, test: &str) -> ToolResult<Value> {
        let state = &self.snapshots[self.index(snapshot)?];
        let case = self.case_index(test)?;
        let result = state.results[case].as_ref();
        Ok(
            json!({"status":"ok","snapshot":snapshot,"bundle":{"root":state.root,"sources":state.sources},
            "environment":state.environment,"test":self.plan.as_ref().unwrap().tests[case],
            "result":result.map(|e| json!({"status":e.status,"observed":e.observed,"origin":e.origin}))}),
        )
    }

    fn replay(&mut self, snapshot: &str, test: &str) -> ToolResult<Value> {
        let index = self.index(snapshot)?;
        let case = self.case_index(test)?;
        let previous = self.snapshots[index].results[case]
            .clone()
            .ok_or_else(|| Failure::new("test.pending", "no retained observation to replay"))?;
        let fresh = self.execute(index, case, Instant::now() + Duration::from_secs(5))?;
        Ok(
            json!({"status":fresh.status,"snapshot":snapshot,"test":test,"reproduced":fresh.status==previous.status && fresh.observed==previous.observed,"observed":preview(&fresh.observed)}),
        )
    }
}
