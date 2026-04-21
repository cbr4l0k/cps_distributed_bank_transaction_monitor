use bytes::{BufMut, BytesMut};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountId(u32);
#[derive(Serialize, Deserialize, Debug)]
pub struct Amount(f32);
#[derive(Serialize, Deserialize, Debug)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    pub account_id: AccountId,
    pub amount: Amount,
    pub tx_type: TransactionType,
}

impl Transaction {
    pub fn new(account_id: AccountId, amount: Amount, tx_type: TransactionType) -> Self {
        Transaction {
            account_id,
            amount,
            tx_type,
        }
    }

    /// Method to convert transaction into bytes for serialization
    /// and datagram transmission.
    /// Source:
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = BytesMut::with_capacity(9);
        buf.put_u32_le(self.account_id.0);
        buf.put_f32_le(self.amount.0);
        let tx_type_byte = match self.tx_type {
            TransactionType::Deposit => 0,
            TransactionType::Withdrawal => 1,
            TransactionType::Transfer => 2,
        };
        buf.put_u8(tx_type_byte);
        buf.to_vec()
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 9 {
            return None;
        }
        let account_id = AccountId(u32::from_le_bytes(bytes[0..4].try_into().ok()?));
        let amount = Amount(f32::from_le_bytes(bytes[4..8].try_into().ok()?));
        let tx_type = match bytes[8] {
            0 => TransactionType::Deposit,
            1 => TransactionType::Withdrawal,
            2 => TransactionType::Transfer,
            _ => return None,
        };
        Some(Transaction {
            account_id,
            amount,
            tx_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_creation() {
        let account_id = AccountId(123);
        let amount = Amount(100.0);
        let tx = Transaction::new(account_id, amount, TransactionType::Deposit);

        assert_eq!(tx.account_id.0, 123);
        assert_eq!(tx.amount.0, 100.0);
        match tx.tx_type {
            TransactionType::Deposit => (),
            _ => panic!("Expected Deposit transaction type"),
        }
    }

    #[test]
    fn test_transaction_to_bytes() {
        let account_id = AccountId(123);
        let amount = Amount(100.0);
        let tx = Transaction::new(account_id, amount, TransactionType::Deposit);
        let bytes = tx.to_bytes();
        assert_eq!(bytes.len(), 9);
        assert_eq!(&bytes[0..4], &123u32.to_le_bytes());
        assert_eq!(&bytes[4..8], &100.0f32.to_le_bytes());
        assert_eq!(bytes[8], 0);
    }
}
