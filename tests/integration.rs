mod deposit;
#[allow(dead_code)]
mod helper;
mod swap;
mod swap_with_parameters;

#[test]
fn test_svm_setup() {
    let _svm = helper::setup_svm();
}
