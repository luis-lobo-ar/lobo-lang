import json,time,uuid
from pathlib import Path
root=Path('.check-io');root.mkdir(exist_ok=True)
key=uuid.uuid4().hex
(root/(key+'.request.json')).write_text('{}')
response=root/(key+'.response.json')
end=time.monotonic()+40
while not response.exists() and time.monotonic()<end:time.sleep(.05)
if not response.exists():raise SystemExit('controller timeout')
print(response.read_text())
