use lobo_core::{compile, Value};
use serde_json::json;
use std::hint::black_box;
use std::io::Write;
use std::time::Instant;

#[inline(always)]
fn increment(x: i64) -> i64 {
    assert!(x < i64::MAX);
    let next = x.checked_add(1).expect("overflow");
    assert_eq!(next, x.checked_add(1).expect("overflow"));
    next
}

#[inline(always)]
fn absolute(x: i64) -> i64 {
    assert!(x != i64::MIN);
    let result = if x < 0 {
        x.checked_neg().expect("overflow")
    } else {
        x
    };
    assert!(result >= 0);
    result
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args[0] == "--help" {
        println!(
            "{}",
            json!({"usage": ["compare emit-c NEW_FILE", "compare rust|interpreted increment|absolute ITERATIONS"]})
        );
        return;
    }
    if args[0] == "emit-c" && args.len() == 2 {
        let artifact = compile(include_str!("../examples/core.lobo")).unwrap();
        assert_eq!(
            artifact.entries(),
            vec!["example.increment", "example.absolute"],
            "native harness function order must match Core example"
        );
        let source = artifact.emit_c().unwrap();
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&args[1])
            .unwrap();
        file.write_all(source.as_bytes()).unwrap();
        return;
    }
    assert_eq!(args.len(), 3, "expected mode, workload, and iterations");
    let interpreted = match args[0].as_str() {
        "interpreted" => true,
        "rust" => false,
        _ => panic!("unknown mode"),
    };
    let absolute_mode = match args[1].as_str() {
        "absolute" => true,
        "increment" => false,
        _ => panic!("unknown workload"),
    };
    let iterations: u64 = args[2].parse().unwrap();
    assert!((1..=1_000_000_000).contains(&iterations));
    // Compilation is deliberately outside the timed region for both execution modes.
    let artifact = compile(include_str!("../examples/core.lobo")).unwrap();
    let entry = if absolute_mode {
        "example.absolute"
    } else {
        "example.increment"
    };
    let mut checksum = 0u64;
    let start = Instant::now();
    if interpreted {
        for i in 0..iterations {
            let input = black_box((i & 2047) as i64 - 1024);
            let value = artifact.run(entry, vec![Value::I64(input)], 100000).unwrap();
            let Value::I64(output) = black_box(value) else {
                panic!("unexpected output type")
            };
            checksum = checksum.wrapping_add(output as u64);
        }
    } else {
        for i in 0..iterations {
            let input = black_box((i & 2047) as i64 - 1024);
            let output = black_box(if absolute_mode {
                absolute(input)
            } else {
                increment(input)
            });
            checksum = checksum.wrapping_add(output as u64);
        }
    }
    let elapsed_ns = start.elapsed().as_nanos();
    println!(
        "{}",
        json!({
            "ns_per_call": elapsed_ns as f64 / iterations as f64,
            "elapsed_ns": elapsed_ns.to_string(),
            "checksum": checksum.to_string(),
            "iterations": iterations
        })
    );
}
