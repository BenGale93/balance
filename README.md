# Balance

A tool for checking whether your current account balance will drop below 0 in
the coming month.

## Installation

Clone this repository using git and run the following command using cargo:

```bash
cargo build --release
```

Then move the resulting binary `./target/release/balance to `~/.cargo/bin/`.
You will then be able to run the program using the `balance` command.

## Setup

You need to place a `spend.yml` file in `~/.config/balance/` and it should look like
below. Include all your regular bills.

```yaml
payments:
  - name: Gas # Name of the bill
    amount: '20' # Amount the bill will be
    day_paid: 6 # Day of the month the bill generally is paid out
```

## Example

Say you get paid on the 18th of the month. To compute the balance left in your
account on the 17th of next month given you have £300.

```bash
$ balance compute 300 -r 18
£-100.00
```

This tells you that you won't have enough to cover all your bills and you might
want to move some money in from your savings.

### Adjusting

Some bills, like a credit card, will be different each month. Prior to
computing your balance, update the bill like so:

```bash
$ balance adjust "Credit Card" -a 454.23
```

### Editing

You can edit the entire config file in your favourite text editor by running:

```bash
$ balance edit
```
