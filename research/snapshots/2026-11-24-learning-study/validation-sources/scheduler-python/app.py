import sys,json
from dataclasses import asdict
import model,logic
a=sys.argv
budget=int(a[1]);jobs=[]
for s in a[2].split(';'):
 if s:
  v=s.split(':');jobs.append(model.Job(int(v[0]),int(v[1]),[int(x) for x in v[2].split(',') if x]))
if len(a)>3:
 total=0
 for i in range(int(a[4])):
  v=logic.run(jobs,budget+i%3);total+=v.spent+v.blocked+sum(v.order)
 print(total)
else:
 v=logic.run(jobs,budget);value=asdict(v)
 print(json.dumps(value))
