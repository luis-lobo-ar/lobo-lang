from model import Job,Summary

def available(job,done):
 return all(x in done for x in job.deps)

def affordable(cost,remaining):
 return cost+3<=remaining

def select_job(jobs,done,remaining):
 best=None
 for item in jobs:
  if item.id not in done and available(item,done) and affordable(item.cost,remaining):
   if best is None or item.id<best.id:best=item
 return best

def run(jobs,budget):
 done=[];remaining=budget
 while True:
  job=select_job(jobs,done,remaining)
  if job is None:break
  done.append(job.id);remaining-=job.cost
 return Summary(done,budget-remaining,len(jobs)-len(done))
