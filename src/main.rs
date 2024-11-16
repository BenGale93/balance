#![warn(clippy::all, clippy::nursery)]

use std::ops::RangeInclusive;

use clap::{Args, Parser, Subcommand};
use payment::{edit_config, store_config, Payments};
use rust_decimal::Decimal;

mod payment;
mod utils;

use anyhow::anyhow;

use crate::payment::{get_config, PaymentManager};

#[derive(Parser)]
struct App {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// For computing the balance at the given reset date.
    Compute(ComputeArgs),
    /// For adjusting a bill.
    Adjust(AdjustArgs),
    /// For listing all the bills.
    List(ListArgs),
    /// For editing the bill config.
    Edit(EditArgs),
}

#[derive(Args)]
struct ComputeArgs {
    /// Current balance of your account.
    balance: Decimal,
    /// Day your bill cycle resets, normally pay day. Defaults to 18 as that is the author's pay day.
    #[arg(short, long, default_value_t = 18)]
    reset_day: isize,
}

fn compute_balance(args: &ComputeArgs, payments: Payments) -> Decimal {
    let ComputeArgs { balance, reset_day } = args;

    let payment_manager = PaymentManager::new(*balance, *reset_day, payments);

    let current_day = chrono::Utc::now().date_naive();

    payment_manager.remaining_balance(&current_day)
}

#[derive(Args)]
struct AdjustArgs {
    /// Bill item to adjust.
    name: String,
    /// New bill amount.
    #[arg(short, long, value_parser = amount_validation)]
    amount: Option<Decimal>,
    /// New day that the bill is paid on.
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

    if amount >= Decimal::new(0, 2) {
        Ok(amount)
    } else {
        Err("amount not greater than or equal to zero".to_string())
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
    /// Whether to include the bill amount in the output.
    #[arg(short, long)]
    amount: bool,
    /// Whether to include the day the bill is paid in the output.
    #[arg(short, long)]
    day_paid: bool,
}

fn list_payments(args: &ListArgs, payments: &mut Payments) {
    payments.sort();
    let ListArgs { amount, day_paid } = args;
    for payment in payments {
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

#[derive(Args)]
struct EditArgs {}

fn main() -> anyhow::Result<()> {
    let args = App::parse();

    let mut config = get_config()?;

    match &args.command {
        Commands::Compute(args) => {
            let balance = compute_balance(args, config.payments);
            println!("£{}", balance);
            Ok(())
        }
        Commands::Adjust(args) => {
            let payments = adjust_entry(args, config.payments)?;
            config.payments = payments;
            store_config(&config)
        }
        Commands::List(args) => {
            list_payments(args, &mut config.payments);
            Ok(())
        }
        Commands::Edit(_) => edit_config(),
    }
}
