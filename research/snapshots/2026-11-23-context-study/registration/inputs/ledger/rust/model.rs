#[derive(Clone)] pub enum Event {Credit(i64),Debit(i64),Lock,Unlock}
pub struct Summary {pub balance:i64,pub rejected:i64,pub peak:i64,pub audit:Vec<i64>}
