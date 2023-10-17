use test_kind::test_kind;

#[test_kind(e2e, resources=db)]
fn e2e_test() {
    // Test code
}

#[test_kind(api, resources=db,net)]
fn api_test() {
    // Test code
}
