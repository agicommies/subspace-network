use pallet_subnet_emission::{
    subnet_pricing::root::RootPricing, PendingEmission, SubnetEmission, UnitEmission,
};
use pallet_subnet_emission_api::{SubnetConsensus, SubnetEmissionApi};
use pallet_subspace::{
    Kappa, MaxAllowedUids, MaxRegistrationsPerBlock, MaxRegistrationsPerInterval,
    MinimumAllowedStake, Rho, TargetRegistrationsPerInterval, Tempo,
};
use substrate_fixed::types::I64F64;

pub use crate::mock::*;

const ROOT_NETUID: u16 = 0;

#[test]
fn test_root_pricing() {
    new_test_ext().execute_with(|| {
        zero_min_burn();

        MaxRegistrationsPerBlock::<Test>::set(6);
        MaxRegistrationsPerInterval::<Test>::set(0, 3);

        assert_ok!(register_named_subnet(u32::MAX, 0, "Rootnet"));
        Test::set_subnet_consensus_type(0, Some(SubnetConsensus::Root));

        let net1_id = 1;
        let net2_id = 2;
        let net3_id = 3;

        let val1_id = 101;
        let val2_id = 102;
        let val3_id = 103;

        let val1_stake = to_nano(20_000);
        let val2_stake = to_nano(40_000);
        let val3_stake = to_nano(40_000);

        assert_ok!(register_module(net1_id, val1_id, val1_stake));
        assert_ok!(register_module(net2_id, val2_id, val2_stake));
        assert_ok!(register_module(net3_id, val3_id, val3_stake));

        let _ = assert_ok!(register_root_validator(val1_id, val1_stake));
        let _ = assert_ok!(register_root_validator(val2_id, val2_stake));
        let _ = assert_ok!(register_root_validator(val3_id, val3_stake));

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
        let priced_subnets = assert_ok!(RootPricing::<Test>::new(0, to_nano(1_000)).run());

        let net1_emission = *priced_subnets.get(&net1_id).unwrap();
        let net2_emission = *priced_subnets.get(&net2_id).unwrap();
        let net3_emission = *priced_subnets.get(&net3_id).unwrap();

        let net1_perc = net1_emission as f32 / distributed as f32;
        let net2_perc = net2_emission as f32 / distributed as f32;
        let net3_perc = net3_emission as f32 / distributed as f32;

        assert_in_range!(net1_perc, 0.04f32, 0.03f32);
        assert_in_range!(net2_perc, 0.78f32, 0.03f32);
        assert_in_range!(net3_perc, 0.18f32, 0.04f32);
    });
}

#[test]
fn test_emission() {
    new_test_ext_with_block(1).execute_with(|| {
        zero_min_burn();
        MinimumAllowedStake::<Test>::set(0);

        assert_ok!(register_named_subnet(u32::MAX, 0, "Rootnet"));
        Test::set_subnet_consensus_type(0, Some(SubnetConsensus::Root));

        let n = 10;
        MaxRegistrationsPerBlock::<Test>::set(n * 2);
        TargetRegistrationsPerInterval::<Test>::set(ROOT_NETUID, n);
        MaxAllowedUids::<Test>::set(ROOT_NETUID, n);
        UnitEmission::<Test>::set(1000000000);
        Rho::<Test>::set(30);
        Kappa::<Test>::set(32767);

        for i in 0..n {
            let key_id: u32 = i as u32;
            let key_origin = get_origin(key_id);

            SubspaceMod::add_balance_to_account(&key_id, 1_000_000_000_000_000);
            assert_ok!(SubspaceMod::register(
                key_origin,
                b"Rootnet".to_vec(),
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
            let uids: Vec<u16> = vec![i];
            let values: Vec<u16> = vec![1];
            assert_ok!(SubspaceMod::set_weights(
                key_origin,
                ROOT_NETUID,
                uids,
                values
            ));
        }

        Tempo::<Test>::set(0, 1);

        let _ = SubnetEmissionMod::get_subnet_pricing(1_000_000_000);
        for netuid in 1..n {
            let emission = SubnetEmission::<Test>::get(netuid);
            println!(
                "expected emission for {}: 99_999_999, got {}",
                netuid, &emission
            );

            assert_eq!(emission, 99_999_999);
        }
        step_block(2);
        println!("stepped 2 blocks");

        for netuid in 1..n {
            let pending_emission = PendingEmission::<Test>::get(netuid);
            println!(
                "expected pending emission for {}: 199_999_998, got {}",
                netuid, &pending_emission
            );
            assert_eq!(pending_emission, 199_999_998);
        }

        step_block(1);
        println!("stepped 1 block");
        for netuid in 1..n {
            let pending_emission = PendingEmission::<Test>::get(netuid);
            println!(
                "expected pending emission for {}: 299_999_997, got {}",
                netuid, &pending_emission
            );
            assert_eq!(pending_emission, 299_999_997);
        }

        let step =
            SubspaceMod::blocks_until_next_epoch(10, SubspaceMod::get_current_block_number());
        step_block(step as u16);
        assert_eq!(PendingEmission::<Test>::get(10), 0);
    });
}

#[test]
fn test_weight_transformation() {
    new_test_ext_with_block(1).execute_with(|| {
        let num_root_validators = 3;

        // log::warn!("num_root_validators = {num_root_validators}");

        let subnet_ids = vec![
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37,
        ];
        // log::warn!("subnet_ids = {subnet_ids:?}");
        let num_subnet_ids = 38;
        // log::warn!("num_subnet_ids = {num_subnet_ids}");

        let mut weights: Vec<Vec<I64F64>> =
            vec![vec![I64F64::from_num(0.0); num_subnet_ids]; num_root_validators];

        let o_weights = vec![(
            3,
            vec![
                (0, 11915),
                (1, 10723),
                (13, 9532),
                (14, 8340),
                (15, 7149),
                (17, 5957),
                (18, 4766),
                (20, 3574),
                (26, 2383),
                (12, 1191),
            ],
        )];

        for (uid_i, weights_i) in o_weights {
            println!("validator {uid_i}");
            for (netuid, weight_ij) in &weights_i {
                println!("netuid {netuid} weight {weight_ij}");

                let idx = uid_i as usize;
                if let Some(weight) = weights.get_mut(idx) {
                    if let Some((w, _)) =
                        weight.iter_mut().zip(&subnet_ids).find(|(_, subnet)| *subnet == netuid)
                    {
                        *w = I64F64::from_num(*weight_ij);
                    } else {
                        println!("huh1?");
                    }
                } else {
                    println!("huh2?");
                }
            }
        }
        // log::warn!("weights = {weights:?}");

        dbg!(&weights);

        panic!("hehe")
    });
}
