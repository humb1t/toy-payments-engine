use std::io::Read;

use crate::{ClientId, errors::Error};

pub use types::*;

mod types;

const TYPE_COLUMN_HEADER: &str = "type";
const CLIENT_COLUMN_HEADER: &str = "client";
const TX_COLUMN_HEADER: &str = "tx";
const AMOUNT_COLUMN_HEADER: &str = "amount";

const DEPOSIT_TRANSACTION_TYPE: &str = "deposit";
const WITHDRAWAL_TRANSACTION_TYPE: &str = "withdrawal";
const DISPUTE_TRANSACTION_TYPE: &str = "dispute";
const RESOLVE_TRANSACTION_TYPE: &str = "resolve";
const CHARGEBACK_TRANSACTION_TYPE: &str = "chargeback";

#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}

pub struct TransactionIterator<R> {
    reader: csv::Reader<R>,
    type_column_index: usize,
    client_column_index: usize,
    transaction_id_column_index: usize,
    amount_column_index: usize,
}

impl Transaction {
    pub fn client(&self) -> ClientId {
        match &self {
            Transaction::Deposit(d) => d.client,
            Transaction::Withdrawal(w) => w.client,
            Transaction::Dispute(d) => d.client,
            Transaction::Resolve(r) => r.client,
            Transaction::Chargeback(c) => c.client,
        }
    }
}

impl<R: Read> TransactionIterator<R> {
    pub fn new(mut reader: csv::Reader<R>) -> Result<Self, Error> {
        let [
            type_column_index,
            client_column_index,
            transaction_id_column_index,
            amount_column_index,
        ] = if let Ok(headers) = reader.headers() {
            let mut result = [0; 4];
            for (index, header) in headers.iter().enumerate() {
                let column_index = match header {
                    TYPE_COLUMN_HEADER => 0,
                    CLIENT_COLUMN_HEADER => 1,
                    TX_COLUMN_HEADER => 2,
                    AMOUNT_COLUMN_HEADER => 3,
                    _ => return Err(Error::Input),
                };
                result[column_index] = index;
            }
            result
        } else {
            [0, 1, 2, 3]
        };
        Ok(Self {
            reader,
            type_column_index,
            client_column_index,
            transaction_id_column_index,
            amount_column_index,
        })
    }
}

impl<R: Read> Iterator for TransactionIterator<R> {
    type Item = Result<Transaction, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.records().next().map(|result| {
            result.map_err(Error::Csv).and_then(|row| {
                let transaction_type = row.get(self.type_column_index).ok_or(Error::Input)?;
                let client: u16 = row
                    .get(self.client_column_index)
                    .ok_or(Error::Input)?
                    .parse()
                    .map_err(|_| Error::Input)?;
                let tx: u32 = row
                    .get(self.transaction_id_column_index)
                    .ok_or(Error::Input)?
                    .parse()
                    .map_err(|_| Error::Input)?;
                match transaction_type {
                    DEPOSIT_TRANSACTION_TYPE => {
                        let amount: f64 = row
                            .get(self.amount_column_index)
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
                    },
                    WITHDRAWAL_TRANSACTION_TYPE => {
                        let amount: f64 = row
                            .get(self.amount_column_index)
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
                    },
                    DISPUTE_TRANSACTION_TYPE => Ok(Transaction::Dispute(Dispute { client, tx })),
                    RESOLVE_TRANSACTION_TYPE => Ok(Transaction::Resolve(Resolve { client, tx })),
                    CHARGEBACK_TRANSACTION_TYPE => Ok(Transaction::Chargeback(Chargeback { client, tx })),
                    _ => Err(Error::Input),
                }
            })
        })
    }
}
