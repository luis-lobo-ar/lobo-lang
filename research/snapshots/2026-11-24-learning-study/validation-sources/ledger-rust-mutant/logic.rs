use crate::model::{Event,Summary};
pub fn permitted(value:i64)->bool {value>=1 && value<=83}
struct State {balance:i64,rejected:i64,peak:i64,locked:bool,audit:Vec<i64> }
fn transition(mut state:State,event:&Event)->State {
 match *event {
  Event::Credit(amount)=> {if permitted(amount) { state.balance+=amount;} else {state.rejected+=1;}},
  Event::Debit(amount)=> {if !permitted(amount) || state.locked || amount>state.balance {state.rejected+=1;} else { state.balance-=amount;}},
  Event::Lock=>state.locked=true,Event::Unlock=>state.locked=false 
 }
 state.peak=state.peak.max(state.balance);state.audit.push(state.balance);state
}
pub fn run(events:&[Event],initial:i64)->Summary {
 let mut state=State {balance:initial,rejected:0,peak:initial,locked:false,audit:Vec::new() };
 for event in events {state=transition(state,event);}
 Summary {balance:state.balance,rejected:state.rejected,peak:state.peak,audit:state.audit}
}
