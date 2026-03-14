use derive_more::Constructor;
use primitive_fixed_point_decimal::ConstScaleFpdec;

pub type Balance = ConstScaleFpdec<i64, 4>;

#[derive(Clone, Debug, PartialEq, Constructor)]
pub struct Deposit {
    pub client: u16,
    pub tx: u32,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Constructor)]
pub struct Withdrawal {
    pub client: u16,
    pub tx: u32,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Constructor)]
pub struct Dispute {
    pub client: u16,
    pub tx: u32,
}

#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Resolve {
    pub client: u16,
    pub tx: u32,
}

#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Chargeback {
    pub client: u16,
    pub tx: u32,
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use primitive_fixed_point_decimal::fpdec;

    use super::*;

    #[test]
    fn deposit_withdrawal() {
        let client = 1;
        let deposit_amount = fpdec!(50.1234);
        let withdrawal_amount = fpdec!(30.1234);
        let _expected_balance: Balance = fpdec!(20.0000);
        let tx_seq = AtomicU32::new(0);
        let deposit = Deposit {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: deposit_amount,
        };
        let withdrawal = Withdrawal {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: withdrawal_amount,
        };
        assert_eq!(deposit.amount, deposit_amount);
        assert_eq!(withdrawal.amount, withdrawal_amount);
        assert!(deposit.amount.is_pos());
        assert!(withdrawal.amount.is_pos());
    }

    #[test]
    fn deposit_withdrawal_dispute() {
        let client = 1;
        let deposit_amount = fpdec!(50.1234);
        let withdrawal_amount = fpdec!(30.1234);
        let tx_seq = AtomicU32::new(0);
        let deposit = Deposit {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: deposit_amount,
        };
        let withdrawal = Withdrawal {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: withdrawal_amount,
        };
        let dispute = Dispute {
            client,
            tx: withdrawal.tx,
        };
        assert_eq!(deposit.amount, deposit_amount);
        assert_eq!(withdrawal.amount, withdrawal_amount);
        assert_eq!(dispute.tx, withdrawal.tx);
    }
}
