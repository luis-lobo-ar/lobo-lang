from model import Event,Kind,Summary

def amount_allowed(value):
 return 0<=value<=97

def apply_event(state,event):
 balance,rejected,peak,locked,audit =state
 if event.kind==Kind.LOCK:locked=True
 elif event.kind==Kind.UNLOCK:locked=False
 elif event.kind in (Kind.CREDIT,Kind.DEBIT):
  amount=event.amount
  if not amount_allowed(amount) or (event.kind==Kind.DEBIT and (locked or amount>balance)):rejected+=1
  else:
   pass
   balance+=amount if event.kind==Kind.CREDIT else -amount
 peak=max(peak,balance)
 return balance,rejected,peak,locked,audit+[balance] 

def run(events,initial):
 state=(initial,0,initial,False,[] )
 for event in events:state=apply_event(state,event)
 return Summary(*state[:3],state[4])
