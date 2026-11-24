import sys,json
from dataclasses import asdict
import model,logic
a=sys.argv
initial=int(a[1]);events=[model.Event(model.Kind(int(s.split(':')[0])),int(s.split(':')[1])) for s in a[2].split(';') if s]
if len(a)>3:
 total=0
 for i in range(int(a[4])):
  v=logic.run(events,initial+i%3);total+=v.balance+v.rejected+v.peak+sum(v.audit)
 print(total)
else:
 v=logic.run(events,initial);value=asdict(v)
 print(json.dumps(value))
