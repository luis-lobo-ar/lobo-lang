struct State { kind: i64, amount: i64 }
fn valid(amount: i64) -> bool { amount >= 0 && amount <= 100 }
fn decode(kind: i64, amount: i64) -> State {
    match kind {
        0 => State { kind: 0, amount },
        1 => State { kind: 1, amount },
        2 => State { kind: 2, amount },
        _ => State { kind: -1, amount: 0 },
    }
}
fn step(state: State, action: i64) -> State {
    match (state.kind, action) {
        (0, 0) => State { kind: 1, amount: state.amount },
        (1, 1) => State { kind: 2, amount: state.amount },
        (2, 2) => State { kind: 0, amount: state.amount },
        _ => State { kind: -1, amount: 0 },
    }
}
fn code(state: State) -> i64 {
    match state.kind {
        0 => state.amount,
        1 => 1000 + state.amount,
        2 => 2000 + state.amount,
        _ => -900,
    }
}
fn run(kind: i64, amount: i64, action: i64) -> i64 {
    if !valid(amount) { -100 } else { code(step(decode(kind, amount), action)) }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("{}", run(args[0].parse().unwrap(), args[1].parse().unwrap(), args[2].parse().unwrap()));
}
