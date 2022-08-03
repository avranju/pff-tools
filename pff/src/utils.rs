use chrono::NaiveDateTime;

const HECTONANOSECS_IN_SEC: i64 = 10_000_000;
const HECTONANOSEC_TO_UNIX_EPOCH: i64 = 11_644_473_600 * HECTONANOSECS_IN_SEC;

fn file_time_to_unix_seconds(t: i64) -> i64 {
    ((t - HECTONANOSEC_TO_UNIX_EPOCH) / HECTONANOSECS_IN_SEC) as i64
}

pub(crate) fn filetime_to_naive_dt(inp: i64) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(file_time_to_unix_seconds(inp), 0)
}
