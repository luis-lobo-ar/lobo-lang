import sys,json
from dataclasses import asdict
import model,logic
a=sys.argv
limit=int(a[1]);data=bytes.fromhex(a[2])
if len(a)>3:
 total=0
 for i in range(int(a[4])):
  v=logic.run(data,limit+i%3);total+=v.consumed+v.error+sum(len(b)+sum(b) for b in v.frames)
 print(total)
else:
 v=logic.run(data,limit);value=asdict(v)
 value['frames']=[b.hex() for b in v.frames]
 print(json.dumps(value))
