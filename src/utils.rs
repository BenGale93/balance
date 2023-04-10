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
