extern crate chrono;

use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};

pub fn days_elapsed_between_timestamps(timestamp_a: i64, timestamp_b: i64, offset: FixedOffset) -> (bool, i64) {
    let naive_a = NaiveDateTime::from_timestamp_opt(timestamp_a, 0).unwrap();
    let timestamp_a_datetime: DateTime<FixedOffset> = offset.from_utc_datetime(&naive_a);
    let naive_b = NaiveDateTime::from_timestamp_opt(timestamp_b, 0).unwrap();
    let timestamp_b_datetime: DateTime<FixedOffset> = offset.from_utc_datetime(&naive_b);
    let duration = timestamp_b_datetime.signed_duration_since(timestamp_a_datetime);
    let days_elapsed = duration.num_days();

    (timestamp_b_datetime.time() <  timestamp_a_datetime.time(), days_elapsed)
}

#[cfg(test)]
mod tests {
    use chrono::FixedOffset;
    use crate::days_elapsed_between_timestamps;

    #[test]
    fn it_works() {
        let hour = 3600;

        assert_eq!((true, 8), days_elapsed_between_timestamps(60 * 60 * 15, 60 * 60 * 24 * 10 + 3600 * -11, FixedOffset::west_opt(8 * hour).unwrap()));
        assert_eq!((false, 9), days_elapsed_between_timestamps(60 * 60 * 15, 60 * 60 * 24 * 10 + 3600 * -4, FixedOffset::west_opt(8 * hour).unwrap()));
        assert_eq!((true, 9), days_elapsed_between_timestamps(60 * 60 * 15, 60 * 60 * 24 * 10 + 3600 * 10, FixedOffset::west_opt(8 * hour).unwrap()));
    }

}