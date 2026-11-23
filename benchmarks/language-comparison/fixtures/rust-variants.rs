enum State { Draft(i64), Approved(i64), Paid(i64), Invalid }
fn valid(amount: i64) -> bool { amount >= 0 && amount <= 100 }
fn decode(kind: i64, amount: i64) -> State {
    match kind {
        0 => State::Draft(amount),
        1 => State::Approved(amount),
        2 => State::Paid(amount),
        _ => State::Invalid,
    }
}
fn step(state: State, action: i64) -> State {
    match state {
        State::Draft(n) if action == 0 => State::Approved(n),
        State::Approved(n) if action == 1 => State::Paid(n),
        State::Paid(n) if action == 2 => State::Draft(n),
        _ => State::Invalid,
    }
}
fn code(state: State) -> i64 {
    match state {
        State::Draft(n) => n,
        State::Approved(n) => 1000 + n,
        State::Paid(n) => 2000 + n,
        State::Invalid => -900,
    }
}
fn run(kind: i64, amount: i64, action: i64) -> i64 {
    if !valid(amount) { -100 } else { code(step(decode(kind, amount), action)) }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    println!("{}", run(args[0].parse().unwrap(), args[1].parse().unwrap(), args[2].parse().unwrap()));
}
