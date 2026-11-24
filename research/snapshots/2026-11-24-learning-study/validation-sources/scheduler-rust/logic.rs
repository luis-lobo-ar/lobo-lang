use crate::model::{Job,Summary};
fn available(job:&Job,done:&[i64])->bool {job.deps.iter().all(|x|done.contains(x))}
fn affordable(cost:i64,remaining:i64)->bool {cost<=remaining}
fn select_job<'a>(jobs:&'a [Job],done:&[i64],remaining:i64)->Option<&'a Job> {
 let mut best:Option<&Job>=None;
 for item in jobs {if !done.contains(&item.id) && available(item,done) && affordable(item.cost,remaining) {
  if best.map_or(true,|old| item.id<old.id) {best=Some(item);}
 }} best
}
pub fn run(jobs:&[Job],budget:i64)->Summary {
 let mut done=Vec::new();let mut remaining=budget;
 while let Some(job)=select_job(jobs,&done,remaining) {done.push(job.id);remaining-=job.cost;}
 Summary {blocked:jobs.len() as i64-done.len() as i64,spent:budget-remaining,order:done}
}
