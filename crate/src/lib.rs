//! Test Kind control.
//!
//! Provide the capability to define what KIND of test is being run, and then to
//! control the execution or skipping of tests, based on their kind and various parameters.
//!
//! There are three basic "kinds" of tests.
//!
//! * `unit`
//! * `integration`
//! * everything else.
//!
//! All test kinds are controlled by the `TEST_KIND_*` environment variables.
//!
//! The `TEST_KIND_*` environment variables are:
//!
//! * `TEST_KIND_EXCLUDE` - A list of Test Kinds NOT to run.  
//!     for example: `TEST_KIND_EXCLUDE=unit,integration` would exclude unit and integration tests.
//!
//! ## Unit Tests
//!
//! Unit tests will only run for 365 days after they were last updated.
//! Then for 30 days after that time they will show as skipped and then fall silent.
//! This is to reduce noise in the CI pipeline, and show important tests.
//! This is the default, the timing can be controlled by env vars:
//!
//! * `TEST_KIND_UNIT_AGE` - Maximum number of days a unit test runs for in CI.
//! * `TEST_KIND_UNIT_SKIP` - Number of days the unit test will show as skipped when it ages out.
//!
//! Setting `TEST_KIND_UNIT_AGE` to 0 will disable unit test age-out.
//!
//! These are specified as:
//!
//! ```rust
//! #[test_kind(unit, updated="YYYY-MM-DD")]
//! fn my_test() {
//!    // Test code
//! }
//! ```
//!
//! ## Integration Tests
//!
//! These tests do not require any external resources, and do not age out.
//! They should be tests on larger interconnected pieces of the code.
//!
//! There are no specific env vars associated with these, other than `TEST_KIND_EXCLUDE`.
//!
//! These are specified as:
//!
//! ```rust
//! #[test_kind(integration)]
//! fn my_test() {
//!    // Test code
//! }
//! ```
//!
//! ## Everything Else
//!
//! All other kinds of tests are expected to have at least 1 external resource dependency.
//!
//! The resources the tests can assume are present are defined as a list in the `TEST_KIND_RESOURCES` env var.
//! When any other kind of test is defined its list of necessary external resources must be supplied.
//!
//! The name of the test is arbitrary but should match what kind of test it is.
//! Examples of these kinds of tests:
//!
//! * `end2end` - End to end tests
//! * `ext-integration` - External Integration tests. (Like integrating to a DB)
//! * `api` - API level tests.
//!
//! There is no limit to the kinds of tests, but they should be constrained by reasonableness.
//! Projects should define a known set of tests, and what they mean to maintain consistency.
//! These can be enforced with the `TEST_KIND_DEFINED` env var, which lists the known list of
//! kinds of tests, `unit` and `integration` do not need to be listed.
//! If this env var is not defined, any unit test name is allowed.
//!
//! These are specified as:
//! ```rust
//! #[test_kind(end2end, resources=foo, bar)]
//! fn my_test() {
//!    // Test code
//! }
//! ```
mod attribute_kind;
mod config;
mod unit_age;

use attribute_kind::{AttributeKind, TestSettings};

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

#[proc_macro_attribute]
pub fn test_kind(attr: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let test_fn = parse_macro_input!(input as syn::ItemFn);

    // Parse the attribute arguments
    let attr_str = parse_macro_input!(attr as LitStr);
    let kind = match AttributeKind::from_lit_str(&attr_str) {
        Ok(kind) => kind,
        Err(err) => return err.to_compile_error().into(),
    };

    match kind.what_to_do() {
        TestSettings::Run => {
            // Return the test function, and allow it to run.
            quote! {
                #[test]
                #test_fn
            }
        }
        TestSettings::Ignore => {
            // Return an empty TokenStream to exclude the function from the code
            quote!()
        }
        TestSettings::Skip { reason } => {
            quote! {
               #[cfg(test)]
               #[ignore = #reason]
               #test_fn
            }
        }
    }
    .into()
}
