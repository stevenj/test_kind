//! What kind of test is this and what are its attributes
use chrono::{Duration, Local, NaiveDate};
use indoc::indoc;
use std::collections::HashSet;
use syn::{Error, Result};

use crate::config::{
    has_resources_available, is_test_kind_defined, is_test_kind_excluded, is_test_resource_defined,
    TEST_KIND_UNIT_AGE,
};
use crate::unit_age::UnitAgeResult;

#[derive(Debug)]
/// What kind of Test is this and its attributes.
pub(crate) enum AttributeKind {
    /// Unit tests.
    Unit {
        /// Last date it was updated.
        updated: NaiveDate,
    },
    /// Stand alone integration tests.
    Integration,
    /// Any other tests that have resources.
    Other {
        /// Kind of test
        kind: String,
        /// Resources it requires.
        resources: Vec<String>,
    },
}

/// What to do with a test based on its kind and attributes.
pub(crate) enum TestSettings {
    /// Run the test.
    Run,
    /// Silently ignore the test.
    Ignore,
    /// Skip the test - with a reason.
    Skip {
        /// Reason for skipping.
        reason: String,
    },
}

impl AttributeKind {
    /// Is this attribute kind excluded?
    fn is_excluded(&self) -> bool {
        match *self {
            AttributeKind::Unit { .. } => is_test_kind_excluded("unit"),
            AttributeKind::Integration => is_test_kind_excluded("integration"),
            AttributeKind::Other { ref kind, .. } => is_test_kind_excluded(kind.as_str()),
        }
    }

    /// Parse the updated date for the unit test kind.
    ///
    /// Date has the format `updated=YYYY-MM-DD`
    ///
    /// Returns an error if the date is invalid.
    /// Date must be:
    /// * after October 10, 2023;
    /// * and no more than 2 days into the future.
    #[allow(clippy::unwrap_in_result)]
    fn parse_updated(attributes: &String, options: &&str) -> Result<NaiveDate> {
        if let Some(date_str) = options.strip_prefix("updated = ") {
            let date = match NaiveDate::parse_from_str(date_str, "%Y - %m - %d") {
                Ok(date) => date,
                Err(err) => {
                    return Err(Error::new_spanned(
                        attributes,
                        format!("Invalid date format: {err:?}"),
                    ))
                }
            };

            // Validate the date
            #[allow(clippy::unwrap_used)]
            let min_date = NaiveDate::from_ymd_opt(2023, 10, 10).unwrap(); // Can't panic
            let max_date = Local::now().date_naive() + Duration::days(2);

            if date < min_date {
                return Err(Error::new_spanned(
                    attributes,
                    format!("`updated={date}` must not be before 10 October 2023."),
                ));
            }

            if date > max_date {
                return Err(Error::new_spanned(
                    attributes,
                    format!("`updated={date}` must not be more than 2 days after the current date. Max date = {max_date}."),
                ));
            }

            Ok(date)
        } else {
            Err(Error::new_spanned(
                attributes,
                format!("Invalid options for test kind 'unit': {attributes}:{options}"),
            ))
        }
    }

    /// Parse the list of resources for the given kind
    ///
    /// Returns an error if the list of resources is invalid, or not unique
    ///
    fn parse_resources(kind: &str, attributes: &String, options: &&str) -> Result<Vec<String>> {
        if !is_test_kind_defined(kind) {
            return Err(Error::new_spanned(
                attributes,
                format!("Undefined Test Kind: {kind}"),
            ));
        }
        if let Some(resources_str) = options.strip_prefix("resources = ") {
            let resources: Vec<String> = resources_str
                .split(',')
                .map(|s| s.trim().to_owned())
                .collect();

            if resources.is_empty() {
                return Err(Error::new_spanned(
                    attributes,
                    "At least one resource must be specified",
                ));
            }

            let unknown_resources: Vec<String> = resources
                .iter()
                .cloned()
                .filter(|r| !is_test_resource_defined(r))
                .collect();
            if !unknown_resources.is_empty() {
                return Err(Error::new_spanned(
                    attributes,
                    format!("Unknown Resources: {unknown_resources:?}"),
                ));
            }

            let unique_set: HashSet<_> = resources.iter().cloned().collect();
            if resources.len() != unique_set.len() {
                return Err(Error::new_spanned(
                    attributes,
                    "Resources may not be specified multiple times",
                ));
            }

            Ok(resources)
        } else {
            Err(Error::new_spanned(
                attributes,
                format!("Invalid list of resources for for test kind {kind} : {options}"),
            ))
        }
    }

    /// Convert the literal string parameters of the macro into a `AttributeKind`.
    ///
    /// * `lit_str`: The literal string
    ///
    /// Returns an error if the parameters are invalid.
    pub(crate) fn from_str(attributes: &String) -> Result<Self> {
        let parts: Vec<&str> = attributes.splitn(2, ',').map(str::trim).collect();

        match *parts.as_slice() {
            ["unit", options] => Ok(Self::Unit {
                updated: AttributeKind::parse_updated(attributes, &options)?,
            }),
            ["integration"] => Ok(Self::Integration),
            [kind, options] => Ok(Self::Other {
                kind: (*kind).to_owned(),
                resources: AttributeKind::parse_resources(kind, attributes, &options)?,
            }),
            _ => {
                let msg = indoc! {"
                    Invalid attribute format.
                    Must be one of: 
                     * unit, updated=YYYY-MM-DD
                     * integration
                     * <something>, resources=<comma separated list of resources>
                "};
                Err(Error::new_spanned(attributes, msg))
            }
        }
    }

    /// What to do with this particular test case?
    pub(crate) fn what_to_do(self) -> TestSettings {
        match self {
            AttributeKind::Unit { updated } => {
                match TEST_KIND_UNIT_AGE.unit_aged_out(updated) {
                    // We only run Young unit tests.
                    UnitAgeResult::Young => {
                        if self.is_excluded() {
                            TestSettings::Skip {
                                reason: "Unit tests are excluded".to_owned(),
                            }
                        } else {
                            TestSettings::Run
                        }
                    }
                    // Recently Aged tests are skipped with a message.
                    UnitAgeResult::Aged(reason) => TestSettings::Skip { reason },
                    // Older than that we just inhibit them.
                    UnitAgeResult::Old => TestSettings::Ignore,
                }
            }

            // Integration tests are only excluded when requested.
            AttributeKind::Integration => {
                if self.is_excluded() {
                    TestSettings::Skip {
                        reason: "Integration tests are excluded".to_owned(),
                    }
                } else {
                    TestSettings::Run
                }
            }

            AttributeKind::Other { kind, resources } => {
                if is_test_kind_excluded(kind.as_str()) {
                    TestSettings::Skip {
                        reason: format!("Test of kind: {kind} are excluded"),
                    }
                } else {
                    let missing_resources = has_resources_available(&resources);
                    if missing_resources.is_empty() {
                        TestSettings::Run
                    } else {
                        TestSettings::Skip {
                            reason: format!("Test of kind: {kind} requires {missing_resources:?}"),
                        }
                    }
                }
            }
        }
    }
}
