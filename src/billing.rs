const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub struct BillingPeriod {
    pub months_to_fetch: Vec<(i32, i32)>, // (year, month_1_indexed)
    pub end: String,
}

pub fn compute_billing_period(
    now_day: i32,
    now_month: i32,
    now_year: i32,
    sub_day: i32,
) -> BillingPeriod {
    if now_day < sub_day {
        let (prev_month, prev_year) = if now_month == 1 {
            (12, now_year - 1)
        } else {
            (now_month - 1, now_year)
        };
        BillingPeriod {
            months_to_fetch: vec![(prev_year, prev_month), (now_year, now_month)],
            end: format!(
                "{} {} {}",
                sub_day,
                MONTH_NAMES[(now_month - 1) as usize],
                now_year
            ),
        }
    } else {
        let (next_month, next_year) = if now_month == 12 {
            (1, now_year + 1)
        } else {
            (now_month + 1, now_year)
        };
        BillingPeriod {
            months_to_fetch: vec![(now_year, now_month)],
            end: format!(
                "{} {} {}",
                sub_day,
                MONTH_NAMES[(next_month - 1) as usize],
                next_year
            ),
        }
    }
}
