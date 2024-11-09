use std::fmt::Display;

use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::utils;

const FILE_NAME: &str = "spend";
const APP_NAME: &str = "balance";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Payment {
    pub name: String,
    pub amount: Decimal,
    pub day_paid: isize,
}

impl Payment {
    #[cfg(test)]
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

impl PartialEq for Payment {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Eq for Payment {}

impl PartialOrd for Payment {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Payment {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

pub type Payments = Vec<Payment>;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub payments: Payments,
}

pub fn get_config() -> Result<Config> {
    let config: Config = confy::load(APP_NAME, Some(FILE_NAME))?;

    Ok(config)
}

pub fn store_config(config: &Config) -> Result<()> {
    confy::store(APP_NAME, Some(FILE_NAME), config)?;
    Ok(())
}

pub fn edit_config() -> Result<()> {
    edit::edit_file(confy::get_configuration_file_path(
        APP_NAME,
        Some(FILE_NAME),
    )?)?;
    Ok(())
}

#[derive(Debug, Clone)]
pub struct PaymentManager {
    balance: Decimal,
    reset_day: isize,
    payments: Payments,
}

impl PaymentManager {
    pub const fn new(balance: Decimal, reset_day: isize, payments: Payments) -> Self {
        Self {
            balance,
            reset_day,
            payments,
        }
    }

    pub fn remaining_balance(&self, current_day: NaiveDate) -> Decimal {
        let rd = self.reset_day;
        let day = current_day.day() as isize;
        let days_in_month = utils::days_in_month(current_day);

        let rebased_cd = utils::modulo(day - rd, days_in_month);

        let leftover_payments: Decimal = self
            .payments
            .iter()
            .map(|p| (p.amount, utils::modulo(p.day_paid - rd, days_in_month)))
            .filter(|p| p.1 > rebased_cd)
            .map(|p| p.0)
            .sum();

        self.balance - leftover_payments
    }
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

    #[test]
    fn payments_are_sorted() {
        let mut payments = vec![
            Payment::new("Water".to_owned(), Decimal::new(2000, 2), 3),
            Payment::new("Phone".to_owned(), Decimal::new(1000, 2), 28),
        ];
        payments.sort();

        assert_eq!(
            payments,
            vec![
                Payment::new("Phone".to_owned(), Decimal::new(1000, 2), 28),
                Payment::new("Water".to_owned(), Decimal::new(2000, 2), 3),
            ]
        );
    }
}
