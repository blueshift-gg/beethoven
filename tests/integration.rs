mod deposit;
#[allow(dead_code)]
mod helper;
mod swap;

#[test]
fn test_svm_setup() {
    let _svm = helper::setup_svm();
}
