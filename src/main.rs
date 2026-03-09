use csv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: u16,
    tx: u32,
    amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Account {
    client: u16,
    available: f64,
    held: f64,
    total: f64,
    locked: bool,
}

impl Account {
    fn new(client: u16) -> Self {
        Account {
            client,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    fn update_available(&mut self, amount: f64) {
        self.available += amount;
        self.total = self.available + self.held;
    }

    fn update_held(&mut self, amount: f64) {
        self.held += amount;
        self.total = self.available + self.held;
    }
}

// Helper struct to store transaction details for dispute tracking
#[derive(Debug, Clone)]
struct TransactionDetails {
    client: u16,
    amount: f64,
}

fn process_transaction(
    transaction: &Transaction,
    accounts: &mut HashMap<u16, Account>,
    disputed_txs: &mut HashMap<u32, TransactionDetails>,
    all_transactions: &mut HashMap<u32, TransactionDetails>,
) -> Result<(), Box<dyn Error>> {
    match transaction.transaction_type {
        TransactionType::Deposit => {
            let account = accounts
                .entry(transaction.client)
                .or_insert_with(|| Account::new(transaction.client));
            if let Some(amount) = transaction.amount {
                account.update_available(amount);
                // Track this for dispute resolution
                all_transactions.insert(
                    transaction.tx,
                    TransactionDetails {
                        client: transaction.client,
                        amount,
                    },
                );
            }
        },
        TransactionType::Withdrawal => {
            let account = accounts
                .entry(transaction.client)
                .or_insert_with(|| Account::new(transaction.client));
            if let Some(amount) = transaction.amount {
                if account.locked || account.available < amount {
                    // Withdrawal fails due to insufficient funds or locked account
                    return Ok(());
                }
                account.update_available(-amount);
                // Track this for dispute resolution
                all_transactions.insert(
                    transaction.tx,
                    TransactionDetails {
                        client: transaction.client,
                        amount,
                    },
                );
            }
        },
        TransactionType::Dispute => {
            // A dispute refers to a tx by ID (not the amount)
            if let Some(tx_details) = all_transactions.get(&transaction.tx) {
                if let Some(account) = accounts.get_mut(&tx_details.client) {
                    if !account.locked {
                        // The disputed amount is the full amount of that transaction
                        let dispute_amount = tx_details.amount;

                        // Check if we have enough available to move to held
                        if account.available >= dispute_amount {
                            account.update_available(-dispute_amount);
                            account.update_held(dispute_amount);

                            // Track this as disputed
                            disputed_txs.insert(transaction.tx, tx_details.clone());
                        }
                    }
                }
            }
        },
        TransactionType::Resolve => {
            // A resolve refers to a tx by ID (not the amount)
            if let Some(tx_details) = disputed_txs.get(&transaction.tx) {
                if let Some(account) = accounts.get_mut(&tx_details.client) {
                    if !account.locked && account.held >= tx_details.amount {
                        // Release funds back to available
                        account.update_held(-tx_details.amount);
                        account.update_available(tx_details.amount);

                        // Remove from disputed transactions
                        disputed_txs.remove(&transaction.tx);
                    }
                }
            }
        },
        TransactionType::Chargeback => {
            // A chargeback refers to a tx by ID (not the amount) and locks account
            if let Some(tx_details) = disputed_txs.get(&transaction.tx) {
                if let Some(account) = accounts.get_mut(&tx_details.client) {
                    if !account.locked {
                        // Deduct held funds and lock the account
                        let chargeback_amount = tx_details.amount;
                        account.held -= chargeback_amount;
                        account.total -= chargeback_amount;
                        account.locked = true;

                        // Remove from disputed transactions
                        disputed_txs.remove(&transaction.tx);
                    }
                }
            }
        },
    }

    Ok(())
}

fn process_transactions(input_path: &str) -> Result<Vec<Account>, Box<dyn Error>> {
    let file = File::open(input_path)?;
    let mut reader = csv::Reader::from_reader(BufReader::new(file));

    let mut accounts: HashMap<u16, Account> = HashMap::new();
    let mut disputed_txs: HashMap<u32, TransactionDetails> = HashMap::new();
    let mut all_transactions: HashMap<u32, TransactionDetails> = HashMap::new();

    // Skip header row
    let _header = reader.headers()?;

    for result in reader.deserialize() {
        let transaction: Transaction = result?;
        process_transaction(&transaction, &mut accounts, &mut disputed_txs, &mut all_transactions)?;
    }

    Ok(accounts.into_iter().map(|(_, account)| account).collect())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} transactions.csv", args[0]);
        std::process::exit(1);
    }

    let input_path = &args[1];
    let accounts = process_transactions(input_path)?;

    // Write output to stdout
    let mut writer = csv::Writer::from_writer(io::stdout());
    writer.write_record(&["client", "available", "held", "total", "locked"])?;

    for account in accounts {
        writer.serialize(account)?;
    }

    writer.flush()?;
    Ok(())
}
