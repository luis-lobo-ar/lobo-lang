use lobo_core::{compile, Value};

#[test]
fn unused_generic_bodies_and_all_branches_are_checked_parametrically() {
    for body in [
        "x + x",
        "if true { x } else { 0 }",
        "x.missing",
        "let y: Bool = x; x",
    ] {
        let source = format!("module m @id(\"m\"); function bad<T> @id(\"bad\")(x: T) -> T {{ {body} }} function f @id(\"f\")() -> I64 {{ 0 }}");
        assert!(
            compile(&source).is_err(),
            "invalid unused generic accepted: {body}"
        );
    }
    let source = "module m @id(\"m\"); function bad<T> @id(\"bad\")(x: T) -> T ensures { result + x == x } { x } function f @id(\"f\")() -> I64 { 0 }";
    assert_eq!(compile(source).unwrap_err().code, "compile.type.mismatch");
}

#[test]
fn type_variables_are_distinct_and_support_uniform_list_and_equality_operations() {
    let source = r#"module m @id("m");
record Pair<T,U> @id("pair") { left: T, right: U }
function good<T,U> @id("good")(x: T, y: U) -> Pair<T,U> {
  let values: List<T> = List::prepend(x, List::empty<T>());
  let a = if List::head(values) == x { x } else { List::head(values) };
  Pair::<T,U> { left: a, right: y }
}
function f @id("f")() -> Bool { good::<I64,Bool>(3,true).right }
"#;
    let artifact = compile(source).unwrap();
    assert_eq!(artifact.run("f", vec![], 100).unwrap(), Value::Bool(true));
    assert!(compile(&source.replace("right: y", "right: x")).is_err());
    assert!(compile(&source.replace("List::head(values) == x", "x == y")).is_err());
}

#[test]
fn unused_generic_nominals_are_validated() {
    for declaration in [
        "record Bad<T> @id(\"bad\") { a: T, a: T }",
        "variant Bad<T> @id(\"bad\") { A(T), A(T) }",
        "record Bad<T> @id(\"bad\") { cycle: Bad<T> }",
    ] {
        let source = format!("module m @id(\"m\"); {declaration} function f @id(\"f\")() -> I64 {{ 0 }}");
        assert!(compile(&source).is_err(), "{declaration}");
    }
}

#[test]
fn concrete_function_specializations_admit_256_and_reject_257() {
    let mut source =
        String::from("module m @id(\"m\"); function choose<T,U> @id(\"choose\")(a:T,b:U) -> T { a }");
    for n in 0..17 {
        source.push_str(&format!("record R{n} @id(\"r{n}\") {{ value:I64 }}"));
    }
    source.push_str("function f @id(\"f\")() -> I64 {");
    for n in 0..257 {
        let (a, b) = (n / 17, n % 17);
        source.push_str(&format!(
            "let v{n} = choose::<R{a},R{b}>(R{a} {{ value:0 }},R{b} {{ value:0 }});"
        ));
        if n == 255 {
            assert_eq!(compile(&format!("{source} 0 }}")).unwrap().entries().len(), 257);
        }
    }
    assert_eq!(
        compile(&format!("{source} 0 }}")).unwrap_err().code,
        "resource.frontend.limit"
    );
}
