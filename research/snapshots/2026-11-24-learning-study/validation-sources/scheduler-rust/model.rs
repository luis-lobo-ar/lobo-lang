#[derive(Clone)] pub struct Job {pub id:i64,pub cost:i64,pub deps:Vec<i64>}
pub struct Summary {pub order:Vec<i64>,pub spent:i64,pub blocked:i64}
