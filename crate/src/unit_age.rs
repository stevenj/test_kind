//! Unit Test Aging control

use chrono::{Duration, NaiveDate, Utc};
use std::env;

/// Defines the aging parameters for unit tests.
pub(crate) struct UnitAge {
    /// Number of days a unit test runs for in CI
    max: u32,
    /// Number of days a unit test is skipped
    skip: u32,
}

/// What to do with a unit test based on its age.
pub(crate) enum UnitAgeResult {
    /// Unit test is young enough to run.
    Young,
    /// Unit test is aged out, but skips with a msg.
    Aged(String),
    /// Unit test is too old. Ignored silently.
    Old,
}

impl UnitAge {
    /// Read the `UnitAge` settings from env vars.
    ///
    /// * `TEST_KIND_UNIT_AGE` - Maximum number of days a unit test runs for in CI.
    /// * `TEST_KIND_UNIT_SKIP` - Number of days the unit test will show as skipped when it ages out.
    ///
    /// Returns the `UnitAge` structure.
    pub(crate) fn from_env() -> UnitAge {
        let max = env::var("TEST_KIND_UNIT_AGE")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(365);

        let skip = env::var("TEST_KIND_UNIT_SKIP")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(30);

        UnitAge { max, skip }
    }

    /// Is the unit test too old?
    ///
    /// Given the `since` date, returns a `UnitAgeResult`.
    pub(crate) fn unit_aged_out(&self, since: NaiveDate) -> UnitAgeResult {
        // Always young if Unit Max age is 0.
        if self.max == 0 {
            return UnitAgeResult::Young;
        }
        let now = Utc::now().naive_utc().date();
        let age = now.signed_duration_since(since);
        let silent_age: i64 = self.max.saturating_add(self.skip).into();
        if age < Duration::days(self.max.into()) {
            UnitAgeResult::Young
        } else {
            let skip_left = silent_age.saturating_sub(age.num_days());
            if skip_left > 0 {
                UnitAgeResult::Aged(format!("Silenced in {skip_left} days"))
            } else {
                UnitAgeResult::Old
            }
        }
    }
}
