use std::collections::HashMap;
use std::result;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::account::Account;

type ClientId = u16;
type TransactionId = u32;

// Represents error conditions the prevented a transaction from successfully completing (i.e.
// a withdrawal failing because of insufficient available funds).
#[derive(Debug)]
pub enum Error {
    AccountFrozen,
    AccountNotFound,
    InsufficientFunds,
    InvalidAmount,
    InvalidChargeback,
    InvalidDispute,
    InvalidResolve,
    TransactionNotFound,
}

type Result<T> = result::Result<T, Error>;

#[derive(Debug, Deserialize, PartialEq)]
// Stands for the type of transactions we need to process. Using the `rename_all` attribute to
// seamlessly deserialize from the corresponding lowercase strings.
#[serde(rename_all = "lowercase")]
enum Type {
    Chargeback,
    Deposit,
    Dispute,
    Resolve,
    Withdrawal,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    // Must match the corresponding CSV column name.
    #[serde(rename = "type")]
    type_: Type,
    client: ClientId,
    tx: TransactionId,
    // Using the `#[serde(default)]` attribute tag here instead of `Option` does not
    // appear to work properly with the `csv::ReaderBuilder::flexible(true)` logic.
    // Added an `amount()` accessor method below which simplifies dealing with the
    // `Option` wrapper based on `unwrap_or_default`.
    amount: Option<Decimal>,
    // Only used for internal bookkeeping.
    #[serde(skip)]
    disputed: bool,
}

impl Transaction {
    fn amount(&self) -> Decimal {
        self.amount.unwrap_or_default()
    }
}

// Implements transaction processing logic.
pub struct TransactionEngine {
    client_accounts: HashMap<ClientId, Account>,
    past_transactions: HashMap<TransactionId, Transaction>,
}

impl TransactionEngine {
    pub fn new() -> Self {
        TransactionEngine {
            client_accounts: HashMap::new(),
            past_transactions: HashMap::new(),
        }
    }

    // Returns a mutable handle to the account associated with `client` (and creates a new
    // entry first if one does not already exist).
    fn account_mut(&mut self, client: ClientId) -> &mut Account {
        self.client_accounts.entry(client).or_default()
    }

    // Given a `TransactionId`, returns a mutable handle to the associated transaction record,
    // and another to the corresponding client account. This is useful to avoid restrictions
    // imposed by the borrow checker when needing both mutable handles at the same time.
    // Returns an error if no such transaction record exists.
    fn transaction_mut(&mut self, tx: TransactionId) -> Result<(&mut Transaction, &mut Account)> {
        let transaction = self
            .past_transactions
            .get_mut(&tx)
            .ok_or(Error::TransactionNotFound)?;

        let account = self
            .client_accounts
            .get_mut(&transaction.client)
            .ok_or(Error::AccountNotFound)?;

        Ok((transaction, account))
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<()> {
        match transaction.type_ {
            Type::Deposit => self.process_deposit(transaction),
            Type::Withdrawal => self.process_withdrawal(transaction),
            Type::Dispute => self.process_dispute(transaction),
            Type::Resolve => self.process_resolve(transaction),
            Type::Chargeback => self.process_chargeback(transaction),
        }
    }

    // Handles a `deposit` transaction.
    fn process_deposit(&mut self, transaction: Transaction) -> Result<()> {
        let amount = transaction.amount();

        if amount.is_sign_negative() {
            return Err(Error::InvalidAmount);
        }

        self.account_mut(transaction.client)
            .check_frozen_mut()?
            .increase_available(amount);
        // Inserting after the amount has been updated successfully. Not checking the
        // return value of `insert` because transaction ids are guaranteed to be unique.
        self.past_transactions.insert(transaction.tx, transaction);

        Ok(())
    }

    // Handles a `withdrawal` transaction.
    fn process_withdrawal(&mut self, transaction: Transaction) -> Result<()> {
        let amount = transaction.amount();

        if amount.is_sign_negative() {
            return Err(Error::InvalidAmount);
        }

        self.account_mut(transaction.client)
            .check_frozen_mut()?
            .withdraw(amount)?;

        self.past_transactions.insert(transaction.tx, transaction);

        Ok(())
    }

    // Handles a `dispute` transaction.
    fn process_dispute(&mut self, transaction: Transaction) -> Result<()> {
        let (t, a) = self.transaction_mut(transaction.tx)?;

        // Only `deposit` transactions can be disputed with this dummy
        // transaction engine.
        if t.disputed || t.type_ != Type::Deposit {
            return Err(Error::InvalidDispute);
        }

        let amount = t.amount();

        // We assume disputes cannot take place while an account is frozen.
        a.check_frozen_mut()?
            .decrease_available(amount)
            .increase_held(amount);

        t.disputed = true;

        Ok(())
    }

    // Handles a `resolve` transaction. We assume `resolve` and `chargeback` operations
    // for disputes that happened before an account got frozen can still go through.
    // It's straightforward to change this behaviour if the assumption is wrong.
    fn process_resolve(&mut self, transaction: Transaction) -> Result<()> {
        let (t, a) = self.transaction_mut(transaction.tx)?;

        if !t.disputed {
            return Err(Error::InvalidResolve);
        }

        let amount = t.amount();
        a.decrease_held(amount).increase_available(amount);

        let id = t.tx;
        // We assume transactions can only be disputed once. Remove the resolved transaction
        // from the current history, so it cannot be disputed again.
        self.past_transactions.remove(&id);

        Ok(())
    }

    fn process_chargeback(&mut self, transaction: Transaction) -> Result<()> {
        let (t, a) = self.transaction_mut(transaction.tx)?;

        if !t.disputed {
            return Err(Error::InvalidChargeback);
        }

        a.decrease_held(t.amount()).freeze();

        let id = t.tx;
        self.past_transactions.remove(&id);

        Ok(())
    }

    // Simple method to print the resulting account data. Could have used the `csv` crate
    // for output as well, but this was quicker.
    pub fn print_accounts(&self) {
        println!("client,available,held,total,locked");

        for (client, account) in self.client_accounts.iter() {
            println!(
                "{},{},{},{},{}",
                client,
                account.available(),
                account.held(),
                account.available() + account.held(),
                account.frozen()
            );
        }
    }
}
