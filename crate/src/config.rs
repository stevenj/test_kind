//! Configuration control for the `test_kind` maro.
//!
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::env;

use crate::unit_age::UnitAge;

lazy_static! {
    static ref TEST_KIND_EXCLUDE: Vec<String> = read_env_var_list("TEST_KIND_EXCLUDE");
    pub(crate) static ref TEST_KIND_UNIT_AGE: UnitAge = UnitAge::from_env();
    static ref TEST_KIND_KNOWN_RESOURCES: Vec<String> =
        read_env_var_list("TEST_KIND_KNOWN_RESOURCES");
    static ref TEST_KIND_RESOURCES: Vec<String> = read_env_var_list("TEST_KIND_RESOURCES");
    static ref TEST_KIND_DEFINED: Vec<String> = read_env_var_list("TEST_KIND_DEFINED");
}

/// Read an env var which contains a comma separated list of items.
///
/// spaces are stripped from the items, such that `foo, foo bar` becomes `["foo", "foobar"]`.
fn read_env_var_list(env_var: &str) -> Vec<String> {
    env::var(env_var)
        .unwrap_or_else(|_| String::new())
        .split(',')
        .map(|s| s.replace(' ', ""))
        .filter(|s| !s.is_empty())
        .collect()
}

/// Check if a test kind is excluded or not.
pub(crate) fn is_test_kind_excluded(kind: &str) -> bool {
    let excluded = TEST_KIND_EXCLUDE
        .iter()
        .any(|s| s.eq_ignore_ascii_case(kind));
    eprintln!("Check test of kind: {kind} are excluded: {excluded}");
    excluded
}

/// Check if a list of resources is found in the available resources.
/// Returns a list of missing resources.
pub(crate) fn has_resources_available(resources: &[String]) -> Vec<String> {
    let set1: HashSet<_> = resources.iter().cloned().collect();
    let set2: HashSet<_> = TEST_KIND_RESOURCES.iter().cloned().collect();

    set1.difference(&set2).cloned().collect()
}

/// Check if a test kind is defined or not.
pub(crate) fn is_test_kind_defined(kind: &str) -> bool {
    // If the env var is not defined, everything is defined.
    if TEST_KIND_DEFINED.is_empty() {
        return true;
    }
    // Otherwise only the listed kinds of tests are defined.
    TEST_KIND_DEFINED
        .iter()
        .any(|s| s.eq_ignore_ascii_case(kind))
}

/// Check if a test resource defined or not.
pub(crate) fn is_test_resource_defined(resource: &str) -> bool {
    // If the env var is not defined, everything is defined.
    if TEST_KIND_KNOWN_RESOURCES.is_empty() {
        return true;
    }
    // Otherwise only the listed kinds of test resources are defined.
    TEST_KIND_KNOWN_RESOURCES
        .iter()
        .any(|s| s.eq_ignore_ascii_case(resource))
}
