#![warn(clippy::all, clippy::nursery)]

use std::ops::RangeInclusive;

use clap::{Args, Parser, Subcommand};
use payment::{payments_to_file, Payments};
use rust_decimal::Decimal;

mod payment;
mod utils;

use anyhow::anyhow;

use crate::payment::{payments_from_file, PaymentManager};

const FILE_NAME: &str = "spend.yml";

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Compute { balance: Decimal },
    Adjust(AdjustArgs),
    List(ListArgs),
}

fn compute_balance(balance: Decimal, payments: Payments) -> Decimal {
    let reset_day: isize = 18;

    let payment_manager = PaymentManager::new(balance, reset_day, payments);

    let current_day = chrono::Utc::now().date_naive();

    payment_manager.remaining_balance(current_day)
}

#[derive(Args)]
struct AdjustArgs {
    name: String,
    #[arg(short, long, value_parser = amount_validation)]
    amount: Option<Decimal>,
    #[arg(short, long, value_parser = days_paid_in_range)]
    day_paid: Option<isize>,
}

const DAYS_PAID_RANGE: RangeInclusive<isize> = 1..=28;

fn days_paid_in_range(s: &str) -> Result<isize, String> {
    let days_paid: isize = s.parse().map_err(|_| format!("`{s}` isn't an integer"))?;

    if DAYS_PAID_RANGE.contains(&days_paid) {
        Ok(days_paid)
    } else {
        Err(format!(
            "days paid not in range {}-{}",
            DAYS_PAID_RANGE.start(),
            DAYS_PAID_RANGE.end()
        ))
    }
}

fn amount_validation(s: &str) -> Result<Decimal, String> {
    let amount: Decimal = s.parse().map_err(|_| format!("`{s}` isn't a Decimal"))?;

    if amount > Decimal::new(0, 2) {
        Ok(amount)
    } else {
        Err("amount not greater than zero".to_string())
    }
}

fn adjust_entry(args: &AdjustArgs, mut payments: Payments) -> anyhow::Result<Payments> {
    for payment in payments.iter_mut() {
        if payment.name != args.name {
            continue;
        }
        if let Some(a) = args.amount {
            payment.amount = a;
        }
        if let Some(d) = args.day_paid {
            payment.day_paid = d;
        }
        return Ok(payments);
    }
    Err(anyhow!("{} not found", args.name))
}

#[derive(Args)]
struct ListArgs {
    #[arg(short, long)]
    amount: bool,
    #[arg(short, long)]
    day_paid: bool,
}

fn list_payments(args: &ListArgs, payments: &mut Payments) {
    payments.sort();
    for payment in payments {
        let ListArgs { amount, day_paid } = args;
        let output = match (amount, day_paid) {
            (true, true) => {
                format!(
                    "{} £{}, day paid: {}",
                    payment.name, payment.amount, payment.day_paid
                )
            }
            (true, false) => format!("{} £{}", payment.name, payment.amount),
            (false, true) => format!("{}, day_paid: {}", payment.name, payment.day_paid),
            (false, false) => payment.name.clone(),
        };

        println!("{output}");
    }
}

fn main() -> anyhow::Result<()> {
    let args = App::parse();

    let mut payments = payments_from_file(FILE_NAME)?;

    match &args.command {
        Commands::Compute { balance } => {
            let balance = compute_balance(*balance, payments);
            println!("£{}", balance);
            Ok(())
        }
        Commands::Adjust(args) => {
            let payments = adjust_entry(args, payments)?;
            payments_to_file(FILE_NAME, &payments)
        }
        Commands::List(args) => {
            list_payments(args, &mut payments);
            Ok(())
        }
    }
}
