----------------------------- MODULE Operations ------------------------------
EXTENDS Naturals, TLC
CONSTANTS Fuel, Bytes, ByteQuota, Deadline, Mutant
VARIABLES pc, fd, pending, registered, connected, peerValid, fuel, now, deadline,
          originalDeadline, sent, received, writeClosed, result, requested, budget, reserved
vars == <<pc, fd, pending, registered, connected, peerValid, fuel, now, deadline,
          originalDeadline, sent, received, writeClosed, result, requested, budget, reserved>>
Waiting == {"connecting", "sending", "receiving"}
Init == /\ pc = "entry" /\ fd = FALSE /\ pending = FALSE /\ registered = FALSE
        /\ connected = FALSE /\ peerValid = FALSE /\ fuel = Fuel /\ now = 0
        /\ deadline = Deadline /\ originalDeadline = Deadline
        /\ sent = 0 /\ received = 0 /\ writeClosed = FALSE /\ result = "none"
        /\ requested = 0 /\ budget = ByteQuota /\ reserved = 0
Acquire == /\ pc = "entry" /\ pc' = "connecting" /\ fd' = TRUE
           /\ pending' = (Mutant # "untracked-fault")
           /\ UNCHANGED <<registered, connected, peerValid, fuel, now, deadline, originalDeadline,
                          sent, received, writeClosed, result, requested, budget, reserved>>
Connect(complete) ==
 /\ pc = "connecting" /\ fuel > 0 /\ pc' = "verify" /\ connected' = complete /\ fuel' = fuel - 1
 /\ UNCHANGED <<fd, pending, registered, peerValid, now, deadline, originalDeadline,
                sent, received, writeClosed, result, requested, budget, reserved>>
Validate == /\ pc = "verify" /\ (connected \/ Mutant = "publish")
            /\ peerValid' = connected /\ pc' = "published" /\ pending' = FALSE /\ registered' = TRUE
            /\ UNCHANGED <<fd, connected, fuel, now, deadline, originalDeadline,
                           sent, received, writeClosed, result, requested, budget, reserved>>
RejectPeer == /\ pc = "verify" /\ ~connected /\ pc' = "cleanup" /\ result' = "closed"
              /\ fd' = FALSE /\ pending' = FALSE
              /\ UNCHANGED <<registered, connected, peerValid, fuel, now, deadline, originalDeadline,
                             sent, received, writeClosed, requested, budget, reserved>>
StartIO == /\ pc = "published" /\ pc' = "ready"
           /\ UNCHANGED <<fd, pending, registered, connected, peerValid, fuel, now, deadline,
                          originalDeadline, sent, received, writeClosed, result, requested, budget, reserved>>
BeginIO(direction, count) ==
 /\ pc = "ready" /\ count \in 1..Bytes /\ count <= budget
 /\ IF direction = "sending" THEN ~writeClosed /\ count <= Bytes - sent ELSE TRUE
 /\ pc' = direction /\ requested' = count /\ budget' = budget - count /\ reserved' = reserved + count
 /\ deadline' = now + Deadline /\ originalDeadline' = now + Deadline
 /\ UNCHANGED <<fd, pending, registered, connected, peerValid, fuel, now, sent, received, writeClosed, result>>
Retry(advance) ==
 /\ pc \in Waiting /\ fuel > 0 /\ now < deadline /\ now + advance <= Fuel
 /\ fuel' = fuel - 1 /\ now' = now + advance
 /\ deadline' = (IF Mutant = "deadline" THEN now + advance + Deadline ELSE deadline)
 /\ UNCHANGED <<pc, fd, pending, registered, connected, peerValid, originalDeadline,
                sent, received, writeClosed, result, requested, budget, reserved>>
Timeout == /\ pc \in Waiting /\ now >= deadline
           /\ pc' = "cleanup" /\ result' = "timeout"
           /\ fd' = (IF registered THEN fd ELSE FALSE) /\ pending' = FALSE
           /\ UNCHANGED <<registered, connected, peerValid, fuel, now, deadline, originalDeadline,
                          sent, received, writeClosed, requested, budget, reserved>>
Fault == /\ pc \in Waiting /\ fuel = 0
         /\ pc' = "cleanup" /\ result' = "fuel"
         /\ UNCHANGED <<fd, pending, registered, connected, peerValid, fuel, now, deadline,
                        originalDeadline, sent, received, writeClosed, requested, budget, reserved>>
OsFailure == /\ pc \in Waiting \cup {"verify"}
             /\ pc' = "cleanup" /\ result' = "os"
             /\ fd' = (IF registered THEN fd ELSE FALSE) /\ pending' = FALSE
             /\ UNCHANGED <<registered, connected, peerValid, fuel, now, deadline, originalDeadline,
                            sent, received, writeClosed, requested, budget, reserved>>
Write(n) == /\ pc = "sending" /\ fuel > 0 /\ n \in 0..requested
            /\ sent' = sent + n /\ fuel' = fuel - 1 /\ pc' = "ready"
            /\ UNCHANGED <<fd, pending, registered, connected, peerValid, now, deadline, originalDeadline,
                           received, writeClosed, result, requested, budget, reserved>>
Read(n) == /\ pc = "receiving" /\ fuel > 0 /\ n \in 1..requested /\ n <= sent - received
           /\ received' = received + n /\ fuel' = fuel - 1 /\ pc' = "ready"
           /\ UNCHANGED <<fd, pending, registered, connected, peerValid, now, deadline, originalDeadline,
                          sent, writeClosed, result, requested, budget, reserved>>
Shutdown == /\ pc = "ready" /\ sent = Bytes /\ ~writeClosed /\ writeClosed' = TRUE
            /\ UNCHANGED <<pc, fd, pending, registered, connected, peerValid, fuel, now, deadline,
                           originalDeadline, sent, received, result, requested, budget, reserved>>
Eof == /\ pc = "receiving" /\ writeClosed /\ received = sent /\ fuel > 0
       /\ pc' = "cleanup" /\ result' = "eof" /\ fuel' = fuel - 1
       /\ UNCHANGED <<fd, pending, registered, connected, peerValid, now, deadline, originalDeadline,
                      sent, received, writeClosed, requested, budget, reserved>>
QuotaFailure == /\ pc = "ready" /\ budget = 0 /\ pc' = "cleanup" /\ result' = "limit"
                /\ UNCHANGED <<fd, pending, registered, connected, peerValid, fuel, now, deadline,
                               originalDeadline, sent, received, writeClosed, requested, budget, reserved>>
Cleanup == /\ pc = "cleanup" /\ pc' = "done"
           /\ fd' = (IF pending \/ registered THEN FALSE ELSE fd)
           /\ pending' = FALSE /\ registered' = FALSE
           /\ UNCHANGED <<connected, peerValid, fuel, now, deadline, originalDeadline,
                          sent, received, writeClosed, result, requested, budget, reserved>>
Done == /\ pc = "done" /\ UNCHANGED vars
Next == Acquire \/ (\E complete \in BOOLEAN: Connect(complete)) \/ Validate \/ RejectPeer \/ StartIO
        \/ (\E direction \in {"sending", "receiving"}, count \in 1..Bytes: BeginIO(direction, count))
        \/ (\E advance \in {0, 1}: Retry(advance)) \/ Timeout \/ Fault \/ OsFailure
        \/ (\E n \in 0..Bytes: Write(n) \/ Read(n)) \/ Shutdown \/ Eof \/ QuotaFailure \/ Cleanup \/ Done
Spec == Init /\ [][Next]_vars /\ WF_vars(Next)
TypeOK == /\ pc \in {"entry", "connecting", "verify", "published", "ready", "sending", "receiving", "cleanup", "done"}
          /\ fd \in BOOLEAN /\ pending \in BOOLEAN /\ registered \in BOOLEAN
          /\ connected \in BOOLEAN /\ peerValid \in BOOLEAN /\ writeClosed \in BOOLEAN
          /\ fuel \in 0..Fuel /\ now \in 0..Fuel /\ deadline \in 0..(Deadline + Fuel)
          /\ originalDeadline \in 0..(Deadline + Fuel)
          /\ sent \in 0..Bytes /\ received \in 0..Bytes /\ requested \in 0..Bytes
          /\ budget \in 0..ByteQuota /\ reserved \in 0..ByteQuota
ValidatedPublication == registered => connected /\ peerValid
FixedDeadline == deadline = originalDeadline
ExitCloses == pc = "done" => ~fd
Ownership == ~(pending /\ registered)
ByteAccounting == received <= sent /\ budget + reserved = ByteQuota
EofAfterDrain == result = "eof" => writeClosed /\ received = sent /\ requested > 0
EventuallyDone == <> (pc = "done")
=============================================================================
