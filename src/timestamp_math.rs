use crate::timestamp::normalized_utc;

pub(crate) fn unix_millis(value: &str) -> Option<i64> {
    if !normalized_utc(value) {
        return None;
    }
    let number = |start, end| value.get(start..end)?.parse::<i64>().ok();
    let year = number(0, 4)?;
    let month = number(5, 7)?;
    let day = number(8, 10)?;
    let hour = number(11, 13)?;
    let minute = number(14, 16)?;
    let second = number(17, 19)?;
    let millis = number(20, 23)?;
    let days = days_from_civil(year, month, day);
    days.checked_mul(86_400_000)?
        .checked_add(hour * 3_600_000)?
        .checked_add(minute * 60_000)?
        .checked_add(second * 1_000)?
        .checked_add(millis)
}

fn days_from_civil(mut year: i64, month: i64, day: i64) -> i64 {
    year -= i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::unix_millis;

    #[test]
    fn calculates_epoch_and_hour_difference() {
        assert_eq!(unix_millis("1970-01-01T00:00:00.000Z"), Some(0));
        let start = unix_millis("2026-08-01T00:00:00.000Z").unwrap();
        let end = unix_millis("2026-08-01T01:00:00.000Z").unwrap();
        assert_eq!(end - start, 3_600_000);
    }
}
