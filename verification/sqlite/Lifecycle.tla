---------------------------- MODULE Lifecycle ----------------------------
EXTENDS Naturals, FiniteSets
CONSTANT Mutation
VARIABLES phase, connected, cursor, active, pending, committed, stored,
          rights, ceiling, connectionRights, budget, badCommit
vars == <<phase,connected,cursor,active,pending,committed,stored,rights,ceiling,connectionRights,budget,badCommit>>
Rights == {"read","write","schema"}
Init == /\ phase="running" /\ connected=FALSE /\ cursor="none" /\ active=FALSE
        /\ pending=0 /\ committed=0 /\ stored=0 /\ rights=Rights /\ ceiling=Rights
        /\ connectionRights={} /\ budget=8 /\ badCommit=FALSE
Available == phase="running" /\ budget>0
Open == /\ Available /\ ~connected /\ "read" \in rights
        /\ connected'=TRUE /\ connectionRights'=rights /\ budget'=budget-1
        /\ UNCHANGED <<phase,cursor,active,pending,committed,stored,rights,ceiling,badCommit>>
Prepare == /\ Available /\ connected /\ cursor="none"
           /\ cursor'="ready" /\ budget'=budget-1
           /\ UNCHANGED <<phase,connected,active,pending,committed,stored,rights,ceiling,connectionRights,badCommit>>
ReadRow == /\ Available /\ connected /\ cursor \in {"ready","row"}
           /\ cursor'="row" /\ budget'=budget-1
           /\ UNCHANGED <<phase,connected,active,pending,committed,stored,rights,ceiling,connectionRights,badCommit>>
DoneStatement == /\ Available /\ connected /\ cursor \in {"ready","row"}
                 /\ cursor'="done" /\ budget'=budget-1
                 /\ UNCHANGED <<phase,connected,active,pending,committed,stored,rights,ceiling,connectionRights,badCommit>>
Finalize == /\ phase="running" /\ connected /\ cursor#"none"
            /\ cursor'="none"
            /\ UNCHANGED <<phase,connected,active,pending,committed,stored,rights,ceiling,connectionRights,budget,badCommit>>
Begin == /\ Available /\ connected /\ cursor="none" /\ ~active
         /\ active'=TRUE /\ budget'=budget-1
         /\ UNCHANGED <<phase,connected,cursor,pending,committed,stored,rights,ceiling,connectionRights,badCommit>>
Write == /\ Available /\ connected /\ cursor="ready" /\ "write" \in connectionRights
         /\ pending+committed<2 /\ cursor'="done" /\ budget'=budget-1
         /\ pending'=IF active THEN pending+1 ELSE pending
         /\ committed'=IF active THEN committed ELSE committed+1
         /\ stored'=IF active THEN stored ELSE stored+1
         /\ UNCHANGED <<phase,connected,active,rights,ceiling,connectionRights,badCommit>>
Commit == /\ Available /\ connected /\ cursor="none" /\ (active \/ Mutation="commit")
          /\ committed'=committed+pending /\ stored'=committed+pending /\ pending'=0
          /\ active'=FALSE /\ badCommit'=~active /\ budget'=budget-1
          /\ UNCHANGED <<phase,connected,cursor,rights,ceiling,connectionRights>>
Rollback == /\ Available /\ connected /\ cursor="none" /\ active
            /\ active'=FALSE /\ pending'=0 /\ budget'=budget-1
            /\ UNCHANGED <<phase,connected,cursor,committed,stored,rights,ceiling,connectionRights,badCommit>>
Derive(take) == /\ Available /\ take \in SUBSET Rights
               /\ (take \subseteq rights \/ Mutation="amplify")
               /\ rights'=take /\ ceiling'=rights /\ budget'=budget-1
               /\ UNCHANGED <<phase,connected,cursor,active,pending,committed,stored,connectionRights,badCommit>>
Exit == /\ phase="running"
        /\ phase'="exited" /\ connected'=FALSE /\ connectionRights'={}
        /\ cursor'=IF Mutation="cleanup" THEN cursor ELSE "none"
        /\ stored'=IF Mutation="persist" THEN committed+pending ELSE stored
        /\ active'=FALSE /\ pending'=0
        /\ UNCHANGED <<committed,rights,ceiling,budget,badCommit>>
Finished == /\ phase="exited" /\ UNCHANGED vars
Next == Open \/ Prepare \/ ReadRow \/ DoneStatement \/ Finalize \/ Begin \/ Write \/ Commit \/ Rollback \/ (\E take \in SUBSET Rights:Derive(take)) \/ Exit \/ Finished
TypeOK == /\ phase \in {"running","exited"} /\ connected \in BOOLEAN /\ active \in BOOLEAN
          /\ cursor \in {"none","ready","row","done"} /\ pending \in 0..2
          /\ committed \in 0..2 /\ stored \in 0..2 /\ budget \in 0..8
          /\ rights \subseteq Rights /\ ceiling \subseteq Rights /\ connectionRights \subseteq Rights
          /\ badCommit \in BOOLEAN
Ownership == cursor="none" \/ connected
NoAmplification == rights \subseteq ceiling
OnlyCommitPersists == stored=committed
LegalCommit == ~badCommit
Spec == Init /\ [][Next]_vars
=============================================================================
