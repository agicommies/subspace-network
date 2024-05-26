mod mock;
use mock::*;

use pallet_subnet_emission::UnitEmission;

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
            SubnetEmission::get_total_emission_per_block(),
            initial_unit_emission
        );

        set_total_issuance(250_000_000 * multiplier);
        assert_eq!(
            SubnetEmission::get_total_emission_per_block(),
            initial_unit_emission / 2
        );

        set_total_issuance(500_000_000 * multiplier);
        assert_eq!(
            SubnetEmission::get_total_emission_per_block(),
            initial_unit_emission / 4
        );

        set_total_issuance(750_000_000 * multiplier);
        assert_eq!(
            SubnetEmission::get_total_emission_per_block(),
            initial_unit_emission / 8 
        );

        set_total_issuance(1_000_000_000 * multiplier);
        assert_eq!(SubnetEmission::get_total_emission_per_block(), 0);

        // mission beyond the maximum supply
        set_total_issuance(1_250_000_000 * multiplier);
        assert_eq!(SubnetEmission::get_total_emission_per_block(), 0);
    });
}
