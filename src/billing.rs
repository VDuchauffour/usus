use chrono::{Datelike, Local, NaiveDate};

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

pub struct BillingPeriod {
    pub months_to_fetch: Vec<(i32, i32)>,
    pub end: String,
    /// Inclusive start of the current billing period (UTC midnight assumed).
    pub start: NaiveDate,
}

pub fn current_period(sub_day: u32) -> BillingPeriod {
    let now = Local::now();
    compute_billing_period(
        now.day() as i32,
        now.month() as i32,
        now.year(),
        sub_day as i32,
    )
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
        let start_day = clamp_to_month(sub_day, prev_year, prev_month);
        BillingPeriod {
            months_to_fetch: vec![(prev_year, prev_month), (now_year, now_month)],
            end: format!(
                "{} {} {}",
                sub_day,
                MONTH_NAMES[(now_month - 1) as usize],
                now_year
            ),
            start: NaiveDate::from_ymd_opt(prev_year, prev_month as u32, start_day as u32)
                .expect("valid start date"),
        }
    } else {
        let (next_month, next_year) = if now_month == 12 {
            (1, now_year + 1)
        } else {
            (now_month + 1, now_year)
        };
        let start_day = clamp_to_month(sub_day, now_year, now_month);
        BillingPeriod {
            months_to_fetch: vec![(now_year, now_month)],
            end: format!(
                "{} {} {}",
                sub_day,
                MONTH_NAMES[(next_month - 1) as usize],
                next_year
            ),
            start: NaiveDate::from_ymd_opt(now_year, now_month as u32, start_day as u32)
                .expect("valid start date"),
        }
    }
}

/// Clamp sub_day to the last valid day of a given month (e.g. 31 -> 28/29 in Feb).
fn clamp_to_month(sub_day: i32, year: i32, month: i32) -> i32 {
    sub_day.min(last_day_of_month(year, month))
}

fn last_day_of_month(year: i32, month: i32) -> i32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    NaiveDate::from_ymd_opt(next_year, next_month as u32, 1)
        .expect("valid month start")
        .pred_opt()
        .expect("not min date")
        .day() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_month_only_when_past_sub_day() {
        let p = compute_billing_period(20, 5, 2026, 15);
        assert_eq!(p.months_to_fetch, vec![(2026, 5)]);
        assert_eq!(p.end, "15 Jun 2026");
        assert_eq!(p.start, NaiveDate::from_ymd_opt(2026, 5, 15).unwrap());
    }

    #[test]
    fn fetches_previous_month_when_before_sub_day() {
        let p = compute_billing_period(10, 5, 2026, 15);
        assert_eq!(p.months_to_fetch, vec![(2026, 4), (2026, 5)]);
        assert_eq!(p.end, "15 May 2026");
        assert_eq!(p.start, NaiveDate::from_ymd_opt(2026, 4, 15).unwrap());
    }

    #[test]
    fn january_before_sub_day_wraps_to_previous_december() {
        let p = compute_billing_period(5, 1, 2026, 20);
        assert_eq!(p.months_to_fetch, vec![(2025, 12), (2026, 1)]);
        assert_eq!(p.end, "20 Jan 2026");
        assert_eq!(p.start, NaiveDate::from_ymd_opt(2025, 12, 20).unwrap());
    }

    #[test]
    fn december_past_sub_day_wraps_end_to_next_january() {
        let p = compute_billing_period(25, 12, 2026, 20);
        assert_eq!(p.months_to_fetch, vec![(2026, 12)]);
        assert_eq!(p.end, "20 Jan 2027");
        assert_eq!(p.start, NaiveDate::from_ymd_opt(2026, 12, 20).unwrap());
    }

    #[test]
    fn boundary_now_day_equals_sub_day_uses_current_month_only() {
        let p = compute_billing_period(15, 5, 2026, 15);
        assert_eq!(p.months_to_fetch, vec![(2026, 5)]);
        assert_eq!(p.end, "15 Jun 2026");
        assert_eq!(p.start, NaiveDate::from_ymd_opt(2026, 5, 15).unwrap());
    }

    #[test]
    fn sub_day_31_in_february_clamps_to_feb_28() {
        let p = compute_billing_period(15, 2, 2026, 31);
        assert_eq!(p.months_to_fetch, vec![(2026, 1), (2026, 2)]);
        assert_eq!(p.start, NaiveDate::from_ymd_opt(2026, 1, 31).unwrap());
    }

    #[test]
    fn sub_day_31_after_february_clamps_start_to_feb_28() {
        let p = compute_billing_period(28, 3, 2026, 31);
        assert_eq!(p.months_to_fetch, vec![(2026, 2), (2026, 3)]);
        assert_eq!(p.start, NaiveDate::from_ymd_opt(2026, 2, 28).unwrap());
    }
}
