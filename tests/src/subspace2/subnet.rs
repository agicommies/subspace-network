use crate::mock::*;
use frame_support::assert_err;
use pallet_governance::{GovernanceConfiguration, SubnetGovernanceConfig, VoteMode};
use pallet_subspace::*;
use sp_runtime::Percent;
use subnet::SubnetChangeset;

#[test]
fn adds_and_removes_subnets() {
    new_test_ext().execute_with(|| {
        zero_min_burn();

        let iterations = 5u16;
        max_subnet_registrations_per_interval(iterations + 1);

        MaxRegistrationsPerBlock::<Test>::set(iterations * iterations);

        for i in 0..iterations {
            assert_ok!(register_module(i, i as u32, 1));
            for j in 0..iterations {
                if i != j {
                    assert_ok!(register_module(i, j as u32, 1));
                }
            }

            assert_eq!(N::<Test>::get(i), iterations);
            assert_eq!(
                TotalSubnets::<Test>::get(),
                i + 1,
                "number of subnets is not equal to expected subnets"
            );
        }

        assert_err!(
            register_module(iterations + 1, 0, 1),
            Error::<Test>::TooManyRegistrationsPerBlock
        );
    });
}

#[test]
fn subnet_update_changes_all_parameter_values() {
    new_test_ext().execute_with(|| {
        assert_ok!(register_module(0, 0, to_nano(10)));

        let params = SubnetParams::<Test> {
            founder: 1,
            founder_share: 65,
            immunity_period: 3,
            incentive_ratio: 4,
            max_allowed_uids: 5,
            max_allowed_weights: 7,
            min_allowed_weights: 6,
            max_weight_age: 600,
            min_stake: 9,
            name: b"test".to_vec().try_into().unwrap(),
            tempo: 300,
            trust_ratio: 11,
            maximum_set_weight_calls_per_epoch: 12,
            bonds_ma: 13,
            target_registrations_interval: 14,
            target_registrations_per_interval: 15,
            max_registrations_per_interval: 16,
            adjustment_alpha: 17,
            governance_config: GovernanceConfiguration {
                proposal_cost: 18,
                proposal_expiration: 19,
                vote_mode: VoteMode::Vote,
                proposal_reward_treasury_allocation: Percent::from_parts(20),
                max_proposal_reward_treasury_allocation: 21,
                proposal_reward_interval: 22,
            },
        };

        let SubnetParams {
            founder,
            founder_share,
            immunity_period,
            incentive_ratio,
            max_allowed_uids,
            max_allowed_weights,
            min_allowed_weights,
            max_weight_age,
            min_stake,
            name,
            tempo,
            trust_ratio,
            maximum_set_weight_calls_per_epoch,
            bonds_ma,
            target_registrations_interval,
            target_registrations_per_interval,
            max_registrations_per_interval,
            adjustment_alpha,
            governance_config,
        } = params.clone();

        SubnetChangeset::<Test>::update(0, params).unwrap().apply(0).unwrap();

        assert_eq!(Founder::<Test>::get(0), founder);
        assert_eq!(FounderShare::<Test>::get(0), founder_share);
        assert_eq!(ImmunityPeriod::<Test>::get(0), immunity_period);
        assert_eq!(IncentiveRatio::<Test>::get(0), incentive_ratio);
        assert_eq!(MaxAllowedUids::<Test>::get(0), max_allowed_uids);
        assert_eq!(MaxAllowedWeights::<Test>::get(0), max_allowed_weights);
        assert_eq!(MinAllowedWeights::<Test>::get(0), min_allowed_weights);
        assert_eq!(MaxWeightAge::<Test>::get(0), max_weight_age);
        assert_eq!(MinStake::<Test>::get(0), min_stake);
        assert_eq!(SubnetNames::<Test>::get(0), name.into_inner());
        assert_eq!(Tempo::<Test>::get(0), tempo);
        assert_eq!(TrustRatio::<Test>::get(0), trust_ratio);
        assert_eq!(
            MaximumSetWeightCallsPerEpoch::<Test>::get(0),
            Some(maximum_set_weight_calls_per_epoch)
        );
        assert_eq!(BondsMovingAverage::<Test>::get(0), bonds_ma);
        assert_eq!(
            TargetRegistrationsInterval::<Test>::get(0),
            target_registrations_interval
        );
        assert_eq!(
            TargetRegistrationsPerInterval::<Test>::get(0),
            target_registrations_per_interval
        );
        assert_eq!(
            MaxRegistrationsPerInterval::<Test>::get(0),
            max_registrations_per_interval
        );
        assert_eq!(AdjustmentAlpha::<Test>::get(0), adjustment_alpha);

        assert_eq!(SubnetGovernanceConfig::<Test>::get(0), governance_config);

        assert_eq!(TotalSubnets::<Test>::get(), 1);
        assert_eq!(N::<Test>::get(0), 1);
    });
}

#[test]
fn removes_subnet_from_storage() {
    new_test_ext().execute_with(|| {
        zero_min_burn();

        macro_rules! params {
            ($m:ident) => {
                let SubnetParams {
                    founder,
                    founder_share,
                    immunity_period,
                    incentive_ratio,
                    max_allowed_uids,
                    max_allowed_weights,
                    min_allowed_weights,
                    max_weight_age,
                    min_stake,
                    name,
                    tempo,
                    trust_ratio,
                    maximum_set_weight_calls_per_epoch: _,
                    bonds_ma,
                    target_registrations_interval,
                    target_registrations_per_interval,
                    max_registrations_per_interval,
                    adjustment_alpha,
                    governance_config,
                } = DefaultSubnetParams::<Test>::get();

                $m!(Founder, founder);
                $m!(FounderShare, founder_share);
                $m!(ImmunityPeriod, immunity_period);
                $m!(IncentiveRatio, incentive_ratio);
                $m!(MaxAllowedUids, max_allowed_uids);
                $m!(MaxAllowedWeights, max_allowed_weights);
                $m!(MinAllowedWeights, min_allowed_weights);
                $m!(MaxWeightAge, max_weight_age);
                $m!(MinStake, min_stake);
                $m!(SubnetNames, name);
                $m!(Tempo, tempo);
                $m!(TrustRatio, trust_ratio);
                $m!(BondsMovingAverage, bonds_ma);
                $m!(TargetRegistrationsInterval, target_registrations_interval);
                $m!(
                    TargetRegistrationsPerInterval,
                    target_registrations_per_interval
                );
                $m!(MaxRegistrationsPerInterval, max_registrations_per_interval);
                $m!(AdjustmentAlpha, adjustment_alpha);
                $m!(SubnetGovernanceConfig, governance_config);
                $m!(N);
            };
        }

        macro_rules! exists {
            ($v:ident, $f:ident) => {
                let _ = $f;
                assert!($v::<Test>::contains_key(0));
            };
            ($v:ident) => {
                assert!($v::<Test>::contains_key(0));
            };
        }
        macro_rules! not_exists {
            ($v:ident, $f:ident) => {
                let _ = $f;
                assert!(!$v::<Test>::contains_key(0));
            };
            ($v:ident) => {
                assert!(!$v::<Test>::contains_key(0));
            };
        }

        assert_ok!(register_module(0, 0, to_nano(10)));
        params!(exists);
        assert_eq!(TotalSubnets::<Test>::get(), 1);

        SubspaceMod::remove_subnet(0);
        params!(not_exists);
        assert_eq!(TotalSubnets::<Test>::get(), 0);
        assert!(SubnetGaps::<Test>::get().contains(&0));
    });
}

#[test]
fn update_subnet_verifies_names_uniquiness_integrity() {
    new_test_ext().execute_with(|| {
        zero_min_burn();
        max_subnet_registrations_per_interval(2);

        let update_params = |key, netuid, params: SubnetParams<Test>| {
            SubspaceMod::update_subnet(
                get_origin(key),
                netuid,
                params.founder,
                params.founder_share,
                params.immunity_period,
                params.incentive_ratio,
                params.max_allowed_uids,
                params.max_allowed_weights,
                params.min_allowed_weights,
                params.max_weight_age,
                params.min_stake,
                params.name,
                params.tempo,
                params.trust_ratio,
                params.maximum_set_weight_calls_per_epoch,
                params.governance_config.vote_mode,
                params.bonds_ma,
                params.target_registrations_interval,
                params.target_registrations_per_interval,
                params.max_registrations_per_interval,
                params.adjustment_alpha,
            )
        };

        assert_ok!(register_module(0, 0, 1));
        assert_ok!(register_module(1, 1, 1));

        assert_ok!(update_params(0, 0, SubspaceMod::subnet_params(0)));
        assert_err!(
            update_params(0, 0, SubspaceMod::subnet_params(1)),
            Error::<Test>::SubnetNameAlreadyExists
        );
    });
}

#[test]
fn subnet_is_replaced_on_reaching_max_allowed_subnets() {
    new_test_ext().execute_with(|| {
        zero_min_burn();
        max_subnet_registrations_per_interval(6);

        // Defines the maximum number of modules, that can be registered,
        // on all subnets at once.
        let expected_subnet_amount: u16 = 3;
        MaxAllowedModules::<Test>::put(expected_subnet_amount);

        let subnets = [
            (0, to_nano(100_000)),
            (1, to_nano(5_000)),
            (2, to_nano(4_000)),
            (3, to_nano(1_100)),
        ];

        let random_keys = [4, 5];

        // Register all subnets
        for (i, (subnet_key, subnet_stake)) in subnets.iter().enumerate() {
            assert_ok!(register_module(i as u16, *subnet_key, *subnet_stake));
        }

        let subnet_amount = TotalSubnets::<Test>::get();
        assert_eq!(subnet_amount, expected_subnet_amount);

        // Register module on the subnet one (netuid 0), this means that subnet
        // subnet two (netuid 1) will be deregistered, as we reached global module limit.
        assert_ok!(register_module(0, random_keys[0], to_nano(1_000)));
        assert_ok!(register_module(4, random_keys[1], to_nano(150_000)));

        let subnet_amount = TotalSubnets::<Test>::get();
        assert_eq!(subnet_amount, expected_subnet_amount - 1);

        // netuid 1 replaced by subnet four
        assert_ok!(register_module(3, subnets[3].0, subnets[3].1));

        let subnet_amount = TotalSubnets::<Test>::get();
        let total_module_amount = SubspaceMod::global_n_modules();
        assert_eq!(subnet_amount, expected_subnet_amount);
        assert_eq!(total_module_amount, expected_subnet_amount);

        let netuids = SubspaceMod::netuids();
        let max_netuid = netuids.iter().max().unwrap();
        assert_eq!(*max_netuid, 2);
    });
}