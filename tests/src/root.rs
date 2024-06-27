pub use crate::mock::*;

const ROOT_NETUID: u16 = 0;

#[test]
fn test_register_root_validator() {
    new_test_ext().execute_with(|| {});
}
