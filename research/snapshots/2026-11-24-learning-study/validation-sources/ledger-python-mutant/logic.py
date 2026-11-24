from model import Event,Kind,Summary

def permitted(value):
 return 1<=value<=83

def transition(state,event):
 balance,rejected,peak,locked,audit =state
 if event.kind==Kind.LOCK:locked=True
 elif event.kind==Kind.UNLOCK:locked=False
 elif event.kind in (Kind.CREDIT,Kind.DEBIT):
  amount=event.amount
  if not permitted(amount) or (event.kind==Kind.DEBIT and (locked or amount>balance)):rejected+=1
  else:
   pass
   balance+=amount if event.kind==Kind.CREDIT else -amount
 peak=max(peak,balance)
 return balance,rejected,peak,locked,audit+[balance] 

def run(events,initial):
 state=(initial,0,initial,False,[] )
 for event in events:state=transition(state,event)
 return Summary(*state[:3],state[4])
