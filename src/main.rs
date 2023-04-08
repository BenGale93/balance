#![warn(clippy::all, clippy::nursery)]
use std::fmt::Display;

use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Payment {
    name: String,
    amount: Decimal,
    day_paid: isize,
}

impl Payment {
    pub const fn new(name: String, amount: Decimal, day_paid: isize) -> Self {
        Self {
            name,
            amount,
            day_paid,
        }
    }
}

impl Display for Payment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Bill: {}\nAmount: £{}\nDay paid: {}",
            self.name, self.amount, self.day_paid
        )
    }
}

pub type Payments = Vec<Payment>;

pub const fn modulo(a: isize, b: isize) -> isize {
    ((a % b) + b) % b
}

pub const fn days_in_month(month: u32) -> isize {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        2 => 28,
        _ => 30,
    }
}

#[derive(Debug)]
pub struct PaymentManager {
    balance: Decimal,
    reset_day: isize,
    payments: Payments,
}

impl PaymentManager {
    pub fn new(balance: Decimal, reset_day: isize, payments: Payments) -> Self {
        Self {
            balance,
            reset_day,
            payments,
        }
    }

    fn remaining_balance(&self, current_day: NaiveDate) -> Decimal {
        let rd = self.reset_day;
        let day = current_day.day() as isize;
        let days_in_month = days_in_month(current_day.month());

        let rebased_cd = modulo(day - rd, days_in_month);

        let leftover_payments: Decimal = self
            .payments
            .iter()
            .map(|p| (p.amount, modulo(p.day_paid - rd, days_in_month)))
            .filter(|p| p.1 > rebased_cd)
            .map(|p| p.0)
            .sum();

        self.balance - leftover_payments
    }
}

fn main() -> Result<()> {
    let balance = std::env::args().nth(1).context("No argument provided")?;
    let balance =
        Decimal::from_str_exact(&balance).context("Couldn't parse balance into Decimal")?;
    let reset_day: isize = 18;

    let payments = std::fs::read_to_string("spend.yml")?;
    let payments: Payments = serde_yaml::from_str(&payments)?;

    let payment_manager = PaymentManager::new(balance, reset_day, payments);

    let current_day = chrono::Utc::now().date_naive();

    println!("£{}", payment_manager.remaining_balance(current_day));
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use rust_decimal::Decimal;

    use super::*;

    #[test]
    fn start_of_period() {
        let payments = vec![
            Payment::new("Phone".to_owned(), Decimal::new(1000, 2), 28),
            Payment::new("Water".to_owned(), Decimal::new(2000, 2), 3),
        ];

        let payment_manager = PaymentManager::new(Decimal::new(10000, 2), 18, payments);

        let remaining =
            payment_manager.remaining_balance(NaiveDate::from_str("2023-01-19").unwrap());
        assert_eq!(remaining, Decimal::new(7000, 2));
    }

    #[test]
    fn midway_through() {
        let payments = vec![
            Payment::new("Phone".to_owned(), Decimal::new(1000, 2), 28),
            Payment::new("Water".to_owned(), Decimal::new(2000, 2), 3),
        ];

        let payment_manager = PaymentManager::new(Decimal::new(10000, 2), 18, payments);

        let remaining =
            payment_manager.remaining_balance(NaiveDate::from_str("2023-01-01").unwrap());
        assert_eq!(remaining, Decimal::new(8000, 2));
    }

    #[test]
    fn same_day() {
        let payments = vec![
            Payment::new("Phone".to_owned(), Decimal::new(1000, 2), 28),
            Payment::new("Water".to_owned(), Decimal::new(2000, 2), 3),
        ];

        let payment_manager = PaymentManager::new(Decimal::new(10000, 2), 18, payments);

        let remaining =
            payment_manager.remaining_balance(NaiveDate::from_str("2023-01-28").unwrap());
        assert_eq!(remaining, Decimal::new(8000, 2));
    }

    #[test]
    fn end_of_month() {
        let payments = vec![
            Payment::new("Phone".to_owned(), Decimal::new(1000, 2), 28),
            Payment::new("Water".to_owned(), Decimal::new(2000, 2), 3),
        ];
        let payment_manager = PaymentManager::new(Decimal::new(10000, 2), 18, payments);

        let remaining =
            payment_manager.remaining_balance(NaiveDate::from_str("2023-01-31").unwrap());

        assert_eq!(remaining, Decimal::new(8000, 2));
    }

    #[test]
    fn reset_day() {
        let payments = vec![
            Payment::new("Phone".to_owned(), Decimal::new(1000, 2), 28),
            Payment::new("Water".to_owned(), Decimal::new(2000, 2), 3),
        ];
        let payment_manager = PaymentManager::new(Decimal::new(10000, 2), 18, payments);

        let remaining =
            payment_manager.remaining_balance(NaiveDate::from_str("2023-01-18").unwrap());

        assert_eq!(remaining, Decimal::new(7000, 2));
    }
}
