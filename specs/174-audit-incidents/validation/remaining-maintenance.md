# Nonblocking result-slot maintenance

LIFE-SERIAL is confirmed: the former tick waited 5,002 ms on a silent worker before returning. The measured fixture uses three pending tasks and a real Unix socket. A first harness attempt could not create its Unix socket because its temporary path was too long; it is not counted as a reproduction.

The tick now starts at most one result-slot operation in the background. A receiver stored under the daemon mutex is the shared in-flight guard, including concurrent ticks. A later tick consumes the result without waiting. Network deadlines, rotation, permission proof creation and rechecks before dispatch stay unchanged. A failed spawn or dropped sender frees the guard at the next tick. The channel has one sender and one result, not an accumulating queue.

The targeted test passes: each tick returns in less than 500 ms, the first worker request reaches its real five-second timeout outside the tick, concurrent ticks create no duplicate reservation, three tasks are visited in order, and no mission is dispatched without a proof. Further integration checks are recorded in the delivery receipt.

T3 reference: ProviderSessionReaper starts its periodic sweep with forkParked and Schedule.spaced. Its asynchronous effect does not monopolize the other control operations. Bridget retains its existing synchronous RPC implementation in a single bounded background operation because remote result-slot identity and permission proofs have no counterpart in T3's local session reaper.

The SQL pending-task scan remains O(n). Only one candidate performs network operations. This change removes the result-slot wait from the shared tick; it does not claim a total latency bound for every other maintenance operation or increase the one-operation concurrency limit.

Minimalism: no new scheduler, persistent state, schema, dependency or permission. The volatile receiver is needed to bound concurrency and preserve completion.

Final review: the pause/resume notification now uses the existing one-second global writer budget, including lock acquisition. A non-reading connection cannot keep the single reservation slot indefinitely. Durable recovery state and the pre-delivery permission check are unchanged.
