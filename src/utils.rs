use chrono::{Datelike, NaiveDate};

pub const fn modulo(a: isize, b: isize) -> isize {
    ((a % b) + b) % b
}

pub fn is_leap_year(date: NaiveDate) -> bool {
    date.year() % 4 == 0 && (date.year() % 100 != 0 || date.year() % 400 == 0)
}

pub fn days_in_month(date: NaiveDate) -> isize {
    match date.month() {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(date) {
                29
            } else {
                28
            }
        }
        _ => unreachable!(),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn days_in_month_leap_year() {
        assert_eq!(
            super::days_in_month(super::NaiveDate::from_ymd_opt(2020, 2, 1).unwrap()),
            29
        );
    }

    #[test]
    fn days_in_feb_1900() {
        assert_eq!(
            super::days_in_month(super::NaiveDate::from_ymd_opt(1900, 2, 1).unwrap()),
            28
        );
    }
}
