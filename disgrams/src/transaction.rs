use crate::errors::{DisgramsError, Result};

pub const TRANSACTION_LEN: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transaction {
    pub account_id: u32,
    pub amount: f32,
    pub tx_type: TransactionType,
}

impl Transaction {
    pub fn new(account_id: u32, amount: f32, tx_type: TransactionType) -> Self {
        Transaction {
            account_id,
            amount,
            tx_type,
        }
    }

    pub fn to_byte_stream(self) -> [u8; 9] {
        let mut out = [0u8; 9];
        out[0..4].copy_from_slice(&self.account_id.to_be_bytes());
        out[4..8].copy_from_slice(&self.amount.to_be_bytes());
        out[8] = match self.tx_type {
            TransactionType::Deposit => 0,
            TransactionType::Withdrawal => 1,
            TransactionType::Transfer => 2,
        };
        out
    }

    pub fn from_byte_stream(bytes: [u8; 9]) -> Result<Self> {
        let account_id = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let amount = f32::from_be_bytes(bytes[4..8].try_into().unwrap());
        let tx_type = match bytes[8] {
            0 => TransactionType::Deposit,
            1 => TransactionType::Withdrawal,
            2 => TransactionType::Transfer,
            _ => return Err(DisgramsError::InvalidTransactionType(bytes[8])),
        };
        Ok(Self {
            account_id,
            amount,
            tx_type,
        })
    }

    pub fn get_transaction_type_as_number(&self) -> u8 {
        match self.tx_type {
            TransactionType::Deposit => 0,
            TransactionType::Withdrawal => 1,
            TransactionType::Transfer => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Transaction, TransactionType};
    use crate::errors::DisgramsError;

    #[test]
    fn transaction_round_trips_through_bytes() {
        let transaction = Transaction::new(123, 45.5, TransactionType::Withdrawal);

        let decoded = Transaction::from_byte_stream(transaction.to_byte_stream()).unwrap();

        assert_eq!(decoded, transaction);
    }

    #[test]
    fn invalid_transaction_type_is_rejected() {
        let mut bytes = [0; 9];
        bytes[8] = 99;

        let err = Transaction::from_byte_stream(bytes).unwrap_err();

        assert!(matches!(err, DisgramsError::InvalidTransactionType(99)));
    }
}
