## Simple transaction engine

Floating point data types are unsuitable for keeping account balances, so we use the
`rust_decimal`crate for fixed point integer representation and arithmetic. A more
detailed assessment (and dependency management impact evaluation) is needed before
deeming the crate secure and performant enough for real-world usage (writing a custom
implementation ourselves can be a better approach, depending on the requirements
of each use case). The `rust_decimal` crate supports more than four decimal places,
but no more than four will appear in the output if the same holds true for the input
as well.

We assume the underlying representation of the `Decimal` type is wide enough such that
the aggregated debit and credit operations cannot lead to arithmetic overflow. We can
switch to using checked arithmetic (i.e. `checked_add` and `checked_sub`), or even
create/leverage wider or boundless integer implementations, if that's not a safe
assumption to make.


### Transaction processing

Here are some of the salient points about the implementation of transaction processing,
together with additional semantics and simplifying assumptions, based on the original
problem description:

- Since we can assume input transactions occur in chronological order, this means
  disputes can only refer to transactions that have been previously processed.

- Only `deposit` transactions can be disputed, based on the definitions from the
  problem statement, and such disputes may cause the amount of available funds
  for an account to become negative. There are a number of options to implement
  disputes for withdrawals as well, if we want to support that. For example,
  a `dispute` here could freeze the client account (for protection purposes maybe,
  and without altering the balance), a `resolve` would then merely unfreeze the
  account, while a `chargeback` also increases available funds before unfreezing.
  
- Transactions can only be disputed once, and are removed from the history of past
  transactions after a dispute gets settled via either a `resolve` or a `chargeback`.
  
- Invalid transactions, as well as invalid CSV records from the input, are ignored. 

- The `client` field for `dispute`, `resolve`, and `chargeback` transactions is ignored
  (the affected client account is identified based on the `client` field from the
  referenced transaction).
  
- Deposits, withdrawals, and disputes become invalid for accounts that are frozen,
  but resolves or chargebacks associated with previous disputes can still go through.
  We can easily change the behaviour if this operating assumption is wrong.
  

### Testing

The transaction handling logic was validated by running the application on a test input
file (`test.csv`). Thorough and diversified (unit, integration, etc.) testing is
required for code that would run in an actual production setting.
