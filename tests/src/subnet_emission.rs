use crate::mock::*;
use frame_support::assert_ok;
use pallet_subnet_emission::PendingEmission;
use sp_core::U256;

// Subnet Pricing
// =============

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
use pallet_subnet_emission::UnitEmission;

// Halving
// =======

// Tests halving logic of the blockchain
#[test]
fn test_halving() {
    new_test_ext().execute_with(|| {
        // Set the emission configuration
        let decimals = 9;
        let multiplier = 10_u64.pow(decimals as u32);
        set_emission_config(decimals, 250_000_000, 1_000_000_000);

        // Set the initial unit emission to a large value
        let initial_unit_emission = 1_000_000_000_000_000;
        UnitEmission::<Test>::put(initial_unit_emission);

        // Test emission at different total issuance levels
        set_total_issuance(0);
        assert_eq!(
            SubnetEmissionMod::get_total_emission_per_block(),
            initial_unit_emission
        );

        set_total_issuance(250_000_000 * multiplier);
        assert_eq!(
            SubnetEmissionMod::get_total_emission_per_block(),
            initial_unit_emission / 2
        );

        set_total_issuance(500_000_000 * multiplier);
        assert_eq!(
            SubnetEmissionMod::get_total_emission_per_block(),
            initial_unit_emission / 4
        );

        set_total_issuance(750_000_000 * multiplier);
        assert_eq!(
            SubnetEmissionMod::get_total_emission_per_block(),
            initial_unit_emission / 8
        );

        set_total_issuance(1_000_000_000 * multiplier);
        assert_eq!(SubnetEmissionMod::get_total_emission_per_block(), 0);

        // mission beyond the maximum supply
        set_total_issuance(1_250_000_000 * multiplier);
        assert_eq!(SubnetEmissionMod::get_total_emission_per_block(), 0);
    });
}
