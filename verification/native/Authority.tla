------------------------------ MODULE Authority ------------------------------
EXTENDS Naturals, FiniteSets, TLC
CONSTANTS Slots, MaxGeneration, HandleLimit, CallBudget, Grants, Mutant
VARIABLES kind, generation, rights, parentRights, root, calls, phase, violation
vars == <<kind, generation, rights, parentRights, root, calls, phase, violation>>
Indices == 1..Slots
Kinds == {"free", "capability", "listener", "stream", "retired"}
Permissions == {"clock", "random", "listen", "connect"}
Sockets == {i \in Indices: kind[i] \in {"listener", "stream"}}
Live == {i \in Indices: kind[i] \in {"capability", "listener", "stream"}}
Space == Cardinality(Live) < HandleLimit \/ Mutant = "quota"
Init == /\ kind = [i \in Indices |-> "free"]
        /\ generation = [i \in Indices |-> 1]
        /\ rights = [i \in Indices |-> {}]
        /\ parentRights = [i \in Indices |-> {}]
        /\ root = 0 /\ calls = CallBudget /\ phase = "active" /\ violation = "none"
Root(depth) ==
 /\ phase = "active" /\ (depth = 1 \/ Mutant = "root")
 /\ IF root = 0 THEN
      /\ Space /\ kind[1] = "free"
      /\ kind' = [kind EXCEPT ![1] = "capability"]
      /\ rights' = [rights EXCEPT ![1] = Grants]
      /\ parentRights' = [parentRights EXCEPT ![1] = Grants]
      /\ root' = 1
    ELSE UNCHANGED <<kind, rights, parentRights, root>>
 /\ violation' = IF depth # 1 THEN "root" ELSE violation
 /\ UNCHANGED <<generation, calls, phase>>
Derive(p, i, mask) ==
 /\ phase = "active" /\ kind[p] = "capability" /\ kind[i] = "free"
 /\ calls > 0 /\ Space
 /\ (mask \subseteq rights[p] \/ Mutant = "amplify")
 /\ kind' = [kind EXCEPT ![i] = "capability"]
 /\ rights' = [rights EXCEPT ![i] = mask]
 /\ parentRights' = [parentRights EXCEPT ![i] = rights[p]]
 /\ calls' = calls - 1
 /\ UNCHANGED <<generation, root, phase, violation>>
Allocate(p, i, k) ==
 LET permission == IF k = "listener" THEN "listen" ELSE "connect" IN
 /\ phase = "active" /\ kind[p] = "capability" /\ kind[i] = "free"
 /\ calls > 0 /\ Space
 /\ (permission \in rights[p] \/ Mutant = "denied")
 /\ kind' = [kind EXCEPT ![i] = k]
 /\ calls' = calls - 1
 /\ violation' = IF permission \notin rights[p] THEN "denied" ELSE violation
 /\ UNCHANGED <<generation, rights, parentRights, root, phase>>
Use(i, g, wanted) ==
 /\ phase = "active" /\ kind[i] \in {"capability", "listener", "stream"} /\ calls > 0
 /\ g <= generation[i]
 /\ (g = generation[i] \/ Mutant = "stale")
 /\ (wanted = kind[i] \/ Mutant = "kind")
 /\ calls' = calls - 1
 /\ violation' = IF g # generation[i] THEN "stale"
                  ELSE IF wanted # kind[i] THEN "kind" ELSE violation
 /\ UNCHANGED <<kind, generation, rights, parentRights, root, phase>>
Release(i, cleanup) ==
 /\ i \in Sockets
 /\ IF cleanup THEN phase = "cleanup" ELSE phase = "active" /\ calls > 0
 /\ kind' = [kind EXCEPT ![i] = IF generation[i] = MaxGeneration /\ Mutant # "wrap" THEN "retired" ELSE "free"]
 /\ generation' = [generation EXCEPT ![i] = IF generation[i] = MaxGeneration
                                                 THEN IF Mutant = "wrap" THEN 1 ELSE @
                                                 ELSE @ + 1]
 /\ rights' = [rights EXCEPT ![i] = {}]
 /\ parentRights' = [parentRights EXCEPT ![i] = {}]
 /\ calls' = IF cleanup THEN calls ELSE calls - 1
 /\ violation' = IF generation[i] = MaxGeneration /\ Mutant = "wrap" THEN "wrap" ELSE violation
 /\ UNCHANGED <<root, phase>>
Exit == /\ phase = "active" /\ phase' = "cleanup"
        /\ UNCHANGED <<kind, generation, rights, parentRights, root, calls, violation>>
Finished == /\ phase = "cleanup" /\ (Sockets = {} \/ Mutant = "cleanup")
            /\ phase' = "done"
            /\ UNCHANGED <<kind, generation, rights, parentRights, root, calls, violation>>
Done == /\ phase = "done" /\ UNCHANGED vars
Next == \/ \E depth \in {1, 2}: Root(depth)
        \/ \E p, i \in Indices, mask \in SUBSET Permissions: Derive(p, i, mask)
        \/ \E p, i \in Indices, k \in {"listener", "stream"}: Allocate(p, i, k)
        \/ \E i \in Indices, g \in 1..MaxGeneration, wanted \in {"capability", "listener", "stream"}: Use(i, g, wanted)
        \/ \E i \in Indices: Release(i, FALSE) \/ Release(i, TRUE)
        \/ Exit \/ Finished \/ Done
Spec == Init /\ [][Next]_vars
TypeOK == /\ kind \in [Indices -> Kinds] /\ generation \in [Indices -> 1..MaxGeneration]
          /\ rights \in [Indices -> SUBSET Permissions] /\ parentRights \in [Indices -> SUBSET Permissions]
          /\ root \in 0..Slots /\ calls \in 0..CallBudget /\ phase \in {"active", "cleanup", "done"}
NoAmplification == \A i \in Indices: kind[i] = "capability" => rights[i] \subseteq parentRights[i]
EntryAuthority == violation # "root"
ValidGeneration == violation # "stale"
ValidKind == violation # "kind"
NoDeniedSyscall == violation # "denied"
WithinQuota == Cardinality(Live) <= HandleLimit
NoGenerationWrap == violation # "wrap"
ExitCloses == phase = "done" => Sockets = {}
=============================================================================
