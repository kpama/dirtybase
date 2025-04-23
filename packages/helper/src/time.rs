use chrono::{DateTime, Duration, Utc};

#[derive(Debug, Clone, Copy)]
pub struct Time {
    now: DateTime<Utc>,
}

impl Time {
    pub fn now() -> Self {
        Self { now: Utc::now() }
    }

    pub fn new(now: DateTime<Utc>) -> Self {
        Self { now }
    }

    pub fn timestamp(&self) -> i64 {
        self.now.timestamp()
    }

    // weeks
    pub fn add_weeks(self, weeks: i64) -> Self {
        let mut now = self.now;
        now += Duration::weeks(weeks);

        Self { now }
    }

    // days
    pub fn add_days(self, days: i64) -> Self {
        let mut now = self.now;
        now += Duration::days(days);

        Self { now }
    }

    // hours
    pub fn add_hours(self, hours: i64) -> Self {
        let mut now = self.now;
        now += Duration::hours(hours);

        Self { now }
    }

    // minutes
    pub fn add_minutes(self, minutes: i64) -> Self {
        let mut now = self.now;
        now += Duration::minutes(minutes);

        Self { now }
    }

    pub fn subtract_minutes(self, minutes: i64) -> Self {
        let mut now = self.now;
        now -= Duration::minutes(minutes);

        Self { now }
    }

    // seconds
    pub fn add_seconds(self, seconds: i64) -> Self {
        let mut now = self.now;
        now += Duration::seconds(seconds);

        Self { now }
    }

    // milliseconds
    pub fn add_milliseconds(self, milliseconds: i64) -> Self {
        let mut now = self.now;
        now += Duration::milliseconds(milliseconds);

        Self { now }
    }

    // microseconds
    pub fn add_microseconds(self, microseconds: i64) -> Self {
        let mut now = self.now;
        now += Duration::microseconds(microseconds);

        Self { now }
    }

    pub fn add_nanoseconds(self, nanoseconds: i64) -> Self {
        let mut now = self.now;
        now += Duration::nanoseconds(nanoseconds);

        Self { now }
    }

    pub fn as_datetime(self) -> DateTime<Utc> {
        self.now
    }
}

pub fn now() -> Time {
    Time::now()
}

pub fn current_datetime() -> DateTime<Utc> {
    Utc::now()
}

pub fn now_ts() -> i64 {
    now().timestamp()
}

impl From<Time> for i64 {
    fn from(value: Time) -> Self {
        value.timestamp()
    }
}

impl From<Time> for Option<i64> {
    fn from(value: Time) -> Self {
        Some(value.timestamp())
    }
}
