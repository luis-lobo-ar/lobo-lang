from model import Job,Summary

def deps_complete(job,done):
 return all(x in done for x in job.deps)

def budget_allows(cost,remaining):
 return cost<=remaining

def next_job(jobs,done,remaining):
 best=None
 for item in jobs:
  if item.id not in done and deps_complete(item,done) and budget_allows(item.cost,remaining):
   if best is None or item.id>best.id:best=item
 return best

def run(jobs,budget):
 done=[];remaining=budget
 while True:
  job=next_job(jobs,done,remaining)
  if job is None:break
  done.append(job.id);remaining-=job.cost
 return Summary(done,budget-remaining,len(jobs)-len(done))
