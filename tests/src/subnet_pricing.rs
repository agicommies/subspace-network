#[allow(unused_imports)]

use crate::mock::*;
use frame_support::assert_ok;
use pallet_subnet_emission::PendingEmission;
use sp_core::U256;

#[test]
fn test_demo_pricing() {
    new_test_ext().execute_with(|| {
        // This test is very naive, it just registers two subnets,
        // and checks if the emission distribution is even. (Proof of concept subnet pricing)
        let key = U256::from(0);
        let key2 = U256::from(1);
        let stake = to_nano(1000);
        assert_ok!(register_module(0, key, stake));
        assert_ok!(register_module(1, key2, stake));
        step_block(10);
        let pending_emission_zero = PendingEmission::<Test>::get(0);
        let pending_emission_one = PendingEmission::<Test>::get(1);
        assert_eq!(pending_emission_zero, pending_emission_one);
    });
}
