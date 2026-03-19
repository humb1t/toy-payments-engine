use cgp::prelude::*;

use crate::errors::Error;

#[cgp_component {
    provider: ProcessTransaction,
    derive_delegate: UseDelegate<Tx>,
}]
pub trait CanProcessTransaction<Tx> {
    fn process_transaction(&mut self, transaction: &Tx) -> Result<(), Error>;
}
