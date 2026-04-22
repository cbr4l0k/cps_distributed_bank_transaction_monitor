use anyhow::{Context, Error, Result};
use disgrams::transaction::{Transaction, TransactionType};
use getrandom::fill;

const DEFAULT_SUSPICIOUS_EVERY: u64 = 25;
const NORMAL_MIN_AMOUNT: f32 = 10.0;
const NORMAL_MAX_AMOUNT: f32 = 500.0;
const SUSPICIOUS_MIN_AMOUNT: f32 = 5_001.0;
const SUSPICIOUS_MAX_AMOUNT: f32 = 25_000.0;
const MIN_ACCOUNT_ID: u32 = 1;
const MAX_ACCOUNT_ID: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuspicionReason {
    HugeAmount,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GeneratedTransaction {
    pub transaction: Transaction,
    pub is_suspicious: bool,
    pub reason: Option<SuspicionReason>,
}

pub struct TransactionGenerator {
    rng: Lcg,
    suspicious_every: u64,
    generated_count: u64,
}

impl TransactionGenerator {
    pub fn new() -> Result<Self, Error> {
        let mut seed = [0u8; 8];
        fill(&mut seed).context("failed to seed transaction generator")?;
        Ok(Self::with_seed(u64::from_be_bytes(seed)))
    }

    fn with_seed(seed: u64) -> Self {
        Self::with_seed_and_suspicious_every(seed, DEFAULT_SUSPICIOUS_EVERY)
    }

    fn with_seed_and_suspicious_every(seed: u64, suspicious_every: u64) -> Self {
        Self {
            rng: Lcg::new(seed),
            suspicious_every,
            generated_count: 0,
        }
    }

    pub fn next(&mut self) -> GeneratedTransaction {
        self.generated_count += 1;
        if self.suspicious_every > 0 && self.generated_count % self.suspicious_every == 0 {
            self.weird_transaction()
        } else {
            self.normal_transaction()
        }
    }

    fn normal_transaction(&mut self) -> GeneratedTransaction {
        GeneratedTransaction {
            transaction: Transaction::new(
                self.account_id(),
                self.amount_between(NORMAL_MIN_AMOUNT, NORMAL_MAX_AMOUNT),
                self.transaction_type(),
            ),
            is_suspicious: false,
            reason: None,
        }
    }

    fn weird_transaction(&mut self) -> GeneratedTransaction {
        GeneratedTransaction {
            transaction: Transaction::new(
                self.account_id(),
                self.amount_between(SUSPICIOUS_MIN_AMOUNT, SUSPICIOUS_MAX_AMOUNT),
                self.transaction_type(),
            ),
            is_suspicious: true,
            reason: Some(SuspicionReason::HugeAmount),
        }
    }

    fn account_id(&mut self) -> u32 {
        self.rng.range_u32(MIN_ACCOUNT_ID, MAX_ACCOUNT_ID)
    }

    fn amount_between(&mut self, min: f32, max: f32) -> f32 {
        let amount = min + self.rng.unit_f32() * (max - min);
        (amount * 100.0).round() / 100.0
    }

    fn transaction_type(&mut self) -> TransactionType {
        match self.rng.range_u32(0, 2) {
            0 => TransactionType::Deposit,
            1 => TransactionType::Withdrawal,
            _ => TransactionType::Transfer,
        }
    }
}

struct Lcg {
    state: u64,
}

impl Lcg {
    fn new(seed: u64) -> Self {
        Self { state: seed.max(1) }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.state >> 32) as u32
    }

    fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        let span = max - min + 1;
        min + self.next_u32() % span
    }

    fn unit_f32(&mut self) -> f32 {
        self.next_u32() as f32 / u32::MAX as f32
    }
}

#[cfg(test)]
mod tests {
    use super::{SuspicionReason, TransactionGenerator};
    use disgrams::transaction::TransactionType;

    #[test]
    fn normal_transaction_follows_expected_pattern() {
        let mut generator = TransactionGenerator::with_seed(7);

        let generated = generator.normal_transaction();

        assert!(!generated.is_suspicious);
        assert_eq!(generated.reason, None);
        assert!((10.0..=500.0).contains(&generated.transaction.amount));
        assert!((1..=20).contains(&generated.transaction.account_id));
        assert!(matches!(
            generated.transaction.tx_type,
            TransactionType::Deposit | TransactionType::Withdrawal | TransactionType::Transfer
        ));
    }

    #[test]
    fn weird_transaction_is_marked_suspicious() {
        let mut generator = TransactionGenerator::with_seed(7);

        let generated = generator.weird_transaction();

        assert!(generated.is_suspicious);
        assert_eq!(generated.reason, Some(SuspicionReason::HugeAmount));
        assert!(generated.transaction.amount > 5000.0);
    }

    #[test]
    fn generator_injects_suspicious_transactions_periodically() {
        let mut generator = TransactionGenerator::with_seed_and_suspicious_every(11, 4);

        let suspicious_flags = (0..8)
            .map(|_| generator.next().is_suspicious)
            .collect::<Vec<_>>();

        assert_eq!(
            suspicious_flags,
            vec![false, false, false, true, false, false, false, true]
        );
    }
}
