mod types;

pub use types::*;

use std::io;

use crate::errors::Error;

pub struct TransactionIterator<R> {
    reader: csv::Reader<R>,
}

impl<R> TransactionIterator<R> {
    pub fn new(reader: csv::Reader<R>) -> Self {
        Self { reader }
    }
}

impl<R: io::Read> Iterator for TransactionIterator<R> {
    type Item = Result<Transaction, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.records().next().map(|result| {
            result.map_err(Error::Csv).and_then(|row| {
                let transaction_type = row.get(0).ok_or(Error::Input)?;

                let client: u16 = row
                    .get(1)
                    .ok_or(Error::Input)?
                    .parse()
                    .map_err(|_| Error::Input)?;
                let tx: u32 = row
                    .get(2)
                    .ok_or(Error::Input)?
                    .parse()
                    .map_err(|_| Error::Input)?;

                match transaction_type {
                    "deposit" => {
                        let amount: f64 = row
                            .get(3)
                            .ok_or(Error::Input)?
                            .parse()
                            .map_err(|_| Error::Input)?;
                        if amount < 0.0 {
                            return Err(Error::Input);
                        }
                        Ok(Transaction::Deposit(Deposit {
                            client,
                            tx,
                            amount: amount.try_into().map_err(|_| Error::Calculation)?,
                        }))
                    }
                    "withdrawal" => {
                        let amount: f64 = row
                            .get(3)
                            .ok_or(Error::Input)?
                            .parse()
                            .map_err(|_| Error::Input)?;
                        if amount < 0.0 {
                            return Err(Error::Input);
                        }
                        Ok(Transaction::Withdrawal(Withdrawal {
                            client,
                            tx,
                            amount: amount.try_into().map_err(|_| Error::Calculation)?,
                        }))
                    }
                    "dispute" => Ok(Transaction::Dispute(Dispute { client, tx })),
                    "resolve" => Ok(Transaction::Resolve(Resolve { client, tx })),
                    "chargeback" => Ok(Transaction::Chargeback(Chargeback { client, tx })),
                    _ => Err(Error::Input),
                }
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}
