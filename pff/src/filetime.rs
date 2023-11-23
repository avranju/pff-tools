use chrono::NaiveDateTime;

const HECTONANOSECS_IN_SEC: i64 = 10_000_000;
const HECTONANOSEC_TO_UNIX_EPOCH: i64 = 11_644_473_600 * HECTONANOSECS_IN_SEC;

pub(crate) struct FileTime(pub(crate) i64);

impl FileTime {
    fn file_time_to_unix_seconds(&self) -> i64 {
        ((self.0 - HECTONANOSEC_TO_UNIX_EPOCH) / HECTONANOSECS_IN_SEC) as i64
    }

    fn filetime_to_naive_dt(&self) -> NaiveDateTime {
        NaiveDateTime::from_timestamp_opt(self.file_time_to_unix_seconds(), 0)
            .expect("from_timestamp_opt cannot fail since nanosecond is set to zero")
    }
}

impl From<FileTime> for NaiveDateTime {
    fn from(ft: FileTime) -> Self {
        ft.filetime_to_naive_dt()
    }
}
