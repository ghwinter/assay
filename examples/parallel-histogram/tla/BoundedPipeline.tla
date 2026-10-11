---- MODULE BoundedPipeline ----
EXTENDS Naturals, Sequences, FiniteSets

CONSTANTS Jobs, Workers, Capacity, NoJob

ASSUME /\ Jobs \subseteq Nat
       /\ Workers \subseteq Nat
       /\ Workers # {}
       /\ Capacity \in Nat
       /\ Capacity > 0
       /\ NoJob \notin Jobs

VARIABLES queue, running, completed, submitted, closed
vars == <<queue, running, completed, submitted, closed>>

Queued == {queue[index] : index \in 1..Len(queue)}
Running == {running[worker] : worker \in Workers} \ {NoJob}

Init ==
    /\ queue = <<>>
    /\ running = [w \in Workers |-> NoJob]
    /\ completed = {}
    /\ submitted = {}
    /\ closed = FALSE

Submit(job) ==
    /\ ~closed
    /\ job \in Jobs \ submitted
    /\ Len(queue) < Capacity
    /\ queue' = Append(queue, job)
    /\ submitted' = submitted \cup {job}
    /\ UNCHANGED <<running, completed, closed>>

Start(worker) ==
    /\ queue # <<>>
    /\ running[worker] = NoJob
    /\ queue' = Tail(queue)
    /\ running' = [running EXCEPT ![worker] = Head(queue)]
    /\ UNCHANGED <<completed, submitted, closed>>

Complete(worker) ==
    /\ running[worker] # NoJob
    /\ completed' = completed \cup {running[worker]}
    /\ running' = [running EXCEPT ![worker] = NoJob]
    /\ UNCHANGED <<queue, submitted, closed>>

Close ==
    /\ ~closed
    /\ submitted = Jobs
    /\ closed' = TRUE
    /\ UNCHANGED <<queue, running, completed, submitted>>

Next ==
    \/ \E job \in Jobs : Submit(job)
    \/ \E worker \in Workers : Start(worker)
    \/ \E worker \in Workers : Complete(worker)
    \/ Close

TypeOK ==
    /\ queue \in Seq(Jobs)
    /\ Len(queue) <= Capacity
    /\ running \in [Workers -> (Jobs \cup {NoJob})]
    /\ submitted \subseteq Jobs
    /\ completed \subseteq Jobs
    /\ closed \in BOOLEAN

Partition ==
    /\ submitted = Queued \cup Running \cup completed
    /\ Queued \cap Running = {}
    /\ Queued \cap completed = {}
    /\ Running \cap completed = {}

Spec ==
    /\ Init
    /\ [][Next]_vars
    /\ \A job \in Jobs : WF_vars(Submit(job))
    /\ \A worker \in Workers : WF_vars(Start(worker))
    /\ \A worker \in Workers : WF_vars(Complete(worker))
    /\ WF_vars(Close)

Drain == closed ~> completed = Jobs
====
