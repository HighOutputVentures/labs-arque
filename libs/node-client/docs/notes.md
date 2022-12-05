# an `Aggregate` has the following responsibilities:
- ensure that business invariants are enforced and never violated
- generate events that describe state changes
- act as a consistency boundary for transactions

# `Command` handlers have the following responsibilities:
- ensure that the command is valid
- verify the business invariants
- generate events that describe state changes

# `Event` handlers have the following responsibilities:
- update the state of the aggregate
