use pallet_subnet_emission::{
    subnet_pricing::root::RootPricing, PendingEmission, SubnetEmission, UnitEmission,
};
use pallet_subspace::{
    BurnConfig, Emission, MaxAllowedUids, MaxRegistrationsPerBlock, MaxRegistrationsPerInterval,
    Stake, SubnetBurnConfig, TargetRegistrationsPerInterval, Tempo, ValidatorPermits,
};

pub use crate::mock::*;

const ROOT_NETUID: u16 = 0;

#[test]
fn test_root_pricing() {
    new_test_ext().execute_with(|| {
        zero_min_burn();

        MaxRegistrationsPerBlock::<Test>::set(6);
        MaxRegistrationsPerInterval::<Test>::set(0, 3);

        let net1_id = 1;
        let net2_id = 2;
        let net3_id = 3;

        let val1_id = 101;
        let val2_id = 102;
        let val3_id = 103;

        let val1_stake = to_nano(20_000);
        let val2_stake = to_nano(40_000);
        let val3_stake = to_nano(40_000);

        let _ = assert_ok!(register_root_validator(val1_id, val1_stake));
        let _ = assert_ok!(register_root_validator(val2_id, val2_stake));
        let _ = assert_ok!(register_root_validator(val3_id, val3_stake));

        assert_ok!(register_module(net1_id, val1_id, val1_stake));
        assert_ok!(register_module(net2_id, val2_id, val2_stake));
        assert_ok!(register_module(net3_id, val3_id, val3_stake));

        set_weights(
            0,
            val1_id,
            vec![1, 2, 3],
            vec![u16::MAX, u16::MIN, u16::MIN],
        );
        set_weights(
            0,
            val2_id,
            vec![1, 2, 3],
            vec![u16::MIN, 655 /* ~1% */, 64879u16 /* ~99% */],
        );
        set_weights(
            0,
            val3_id,
            vec![1, 2, 3],
            vec![u16::MIN, u16::MAX, u16::MIN],
        );

        let distributed = to_nano(1_000);
        let priced_subnets = assert_ok!(RootPricing::<Test>::new(to_nano(1_000)).run());

        let net1_emission = *priced_subnets.get(&net1_id).unwrap();
        let net2_emission = *priced_subnets.get(&net2_id).unwrap();
        let net3_emission = *priced_subnets.get(&net3_id).unwrap();

        let net1_perc = net1_emission as f32 / distributed as f32;
        let net2_perc = net2_emission as f32 / distributed as f32;
        let net3_perc = net3_emission as f32 / distributed as f32;

        dbg!(&net1_perc);
        dbg!(&net2_perc);
        dbg!(&net3_perc);

        assert_eq!(net1_perc, 0.04f32);
        assert_eq!(net2_perc, 0.78f32);
        assert_eq!(net3_perc, 0.18f32);
    });
}

#[test]
fn test_emission() {
    new_test_ext().execute_with(|| {
        zero_min_burn();

        let n = 10;
        MaxRegistrationsPerBlock::<Test>::set(n * 2 as u16);
        TargetRegistrationsPerInterval::<Test>::set(ROOT_NETUID, n * 2 as u16);
        MaxAllowedUids::<Test>::set(ROOT_NETUID, n as u16);
        UnitEmission::<Test>::set(500000001);

        for i in 0..n {
            let key_id: u32 = i as u32;
            let key_origin = get_origin(key_id);

            SubspaceMod::add_balance_to_account(&key_id, 1_000_000_000_000_000);
            assert_ok!(SubspaceMod::register(
                key_origin,
                b"rootnet".to_vec(),
                format!("test{}", i).as_bytes().to_vec(),
                b"0.0.0.0:30333".to_vec(),
                1000,
                key_id,
                None
            ));
        }

        for i in 1..n {
            let key_id: u32 = i as u32 + 100;
            let key_origin = get_origin(key_id);
            SubspaceMod::add_balance_to_account(&key_id, 1_000_000_000_000_000);
            assert_ok!(SubspaceMod::register(
                key_origin,
                format!("net{}", i).as_bytes().to_vec(),
                format!("test{}", i).as_bytes().to_vec(),
                b"0.0.0.0:30333".to_vec(),
                1000,
                key_id,
                None
            ));
        }

        for i in 0..n {
            let key_id: u32 = i as u32;
            let key_origin = get_origin(key_id);
            let uids: Vec<u16> = vec![i as u16];
            let values: Vec<u16> = vec![1];
            assert_ok!(SubspaceMod::set_weights(
                key_origin,
                ROOT_NETUID,
                uids,
                values
            ));
        }

        Tempo::<Test>::set(0, 1);
        // let priced_subnets = assert_ok!(RootPricing::<Test>::new(1_000_000_000).run());

        SubnetEmissionMod::process_emission_distribution(1_000_000_000, 1_000_000_000);
        for netuid in 1..n {
            let emission = SubnetEmission::<Test>::get(netuid);
            println!(
                "expected emission for {}: 99_999_999, got {}",
                netuid, &emission
            );

            // assert_eq!(emission, 99_999_999);
        }
        step_block(2);
        println!("stepped 2 blocks");

        for netuid in 1..n {
            let pending_emission = PendingEmission::<Test>::get(netuid);
            println!(
                "expected pending emission for {}: 199_999_998, got {}",
                netuid, &pending_emission
            );
            // assert_eq!(pending_emission, 199_999_998);
        }

        step_block(1);
        println!("stepped 1 block");
        for netuid in 1..n {
            let pending_emission = PendingEmission::<Test>::get(netuid);
            println!(
                "expected pending emission for {}: 299_999_997, got {}",
                netuid, &pending_emission
            );
            // assert_eq!(pending_emission, 299_999_997);
        }

        panic!();

        // let step = SubnetEmissionMod::blocks_until_next_epoch(
        //     10,
        //     1000,
        //     SubspaceMod::get_current_block_number(),
        // );
        // step_block(step as u16);
        // assert_eq!(PendingEmission::<Test>::get(10), 0);
    });
}
