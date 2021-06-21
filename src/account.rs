use std::result;

use rust_decimal::Decimal;

use crate::transaction::Error;

type Result<T> = result::Result<T, Error>;

// Contains data associated with a client account, and implements helper methods.
#[derive(Default)]
pub struct Account {
    available: Decimal,
    held: Decimal,
    frozen: bool,
}

impl Account {
    // The methods below return a `&mut Self` so they can be chained when appropriate/useful.
    // Also we assume the input has been validated beforehand (i.e. then `amount` is a
    // positive value).

    pub fn increase_available(&mut self, amount: Decimal) -> &mut Self {
        self.available += amount;
        self
    }

    pub fn decrease_available(&mut self, amount: Decimal) -> &mut Self {
        self.available -= amount;
        self
    }

    pub fn increase_held(&mut self, amount: Decimal) -> &mut Self {
        self.held += amount;
        self
    }

    pub fn decrease_held(&mut self, amount: Decimal) -> &mut Self {
        self.held -= amount;
        self
    }

    pub fn withdraw(&mut self, amount: Decimal) -> Result<&mut Self> {
        // A withdrawal cannot take place if the specified `amount` is greater than
        // the currently available funds.
        if self.available >= amount {
            self.available -= amount;
            Ok(self)
        } else {
            Err(Error::InsufficientFunds)
        }
    }

    pub fn freeze(&mut self) -> &mut Self {
        self.frozen = true;
        self
    }

    pub fn check_frozen_mut(&mut self) -> Result<&mut Self> {
        if self.frozen {
            return Err(Error::AccountFrozen);
        }
        Ok(self)
    }

    pub fn available(&self) -> Decimal {
        self.available
    }

    pub fn held(&self) -> Decimal {
        self.held
    }

    pub fn frozen(&self) -> bool {
        self.frozen
    }
}
