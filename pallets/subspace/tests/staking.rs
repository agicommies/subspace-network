mod mock;

use frame_support::assert_ok;
use mock::*;
use sp_core::U256;

// /***********************************************************
// 	staking::add_stake() tests
// ************************************************************/
// #[test]
// fn test_stake_overflow() {
// 	new_test_ext().execute_with(|| {

//         let token_amount : u64 = 1_000_000_000;
//         let balance : u64 = 10 * token_amount;
//         let netuid : u16 = 0;

//         for i in [0,1].iter() {
//             let delta : u64 = 1 * token_amount;
//             let stake : u64 = balance + delta*(*i);
//             let key : U256 = U256::from(*i);
//             add_balance(key, balance);
//             let result =register_module(netuid, key, stake);
//             println!("RESULT: {:?}", result);

//             println!("STAKE {}", SubspaceModule::get_stake(netuid, &key));
//             assert_eq!(SubspaceModule::get_stake(netuid, &key), balance);
//             assert_eq!(SubspaceModule::get_balance(&key), 0);
//         }

// 	});
// }

#[test]
fn test_stake() {
    new_test_ext().execute_with(|| {
        let max_uids: u16 = 10;
        let token_amount: u64 = 1_000_000_000;
        let netuids: [u16; 4] = core::array::from_fn(|i| i as u16);
        let stake_amounts: Vec<u64> = netuids.iter().map(|_| 10 * token_amount).collect();
        let mut total_stake: u64 = 0;
        let mut subnet_stake: u64 = 0;

        for netuid in netuids {
            let staked_amount = stake_amounts[netuid as usize];
            let key_vector =
                (0..max_uids).map(|i| U256::from(i + max_uids * netuid)).collect::<Vec<U256>>();

            for key in key_vector.iter() {
                assert_ok!(register_module(netuid, *key, staked_amount));

                // SubspaceModule::add_stake(get_origin(*key), netuid, amount_staked);
                assert_eq!(SubspaceModule::get_stake(netuid, key), staked_amount);
                assert_eq!(SubspaceModule::get_balance(key), 1);

                // REMOVE STAKE
                assert_ok!(SubspaceModule::remove_stake(
                    get_origin(*key),
                    netuid,
                    *key,
                    staked_amount
                ));
                assert_eq!(SubspaceModule::get_balance(key), staked_amount + 1);
                assert_eq!(SubspaceModule::get_stake(netuid, key), 0);

                // ADD STAKE AGAIN LOL
                assert_ok!(SubspaceModule::add_stake(
                    get_origin(*key),
                    netuid,
                    *key,
                    staked_amount
                ));
                assert_eq!(SubspaceModule::get_stake(netuid, key), staked_amount);
                assert_eq!(SubspaceModule::get_balance(key), 1);

                // AT THE END WE SHOULD HAVE THE SAME TOTAL STAKE
                subnet_stake += SubspaceModule::get_stake(netuid, key);
            }
            assert_eq!(SubspaceModule::get_total_subnet_stake(netuid), subnet_stake);
            total_stake += subnet_stake;
            assert_eq!(SubspaceModule::total_stake(), total_stake);
            subnet_stake = 0;
        }
    });
}

#[test]
fn test_multiple_stake() {
    new_test_ext().execute_with(|| {
        let n: u16 = 10;
        let stake_amount: u64 = 10_000_000_000;
        let netuid: u16 = 0;
        let staked_modules: u16 = 10;
        let total_stake: u64 = stake_amount * staked_modules as u64;

        register_n_modules(netuid, n, 0);
        let controler_key = U256::from(n + 1);
        let og_staker_balance: u64 = total_stake + 1;
        add_balance(controler_key, og_staker_balance);

        let keys: Vec<U256> = SubspaceModule::get_keys(netuid);

        // stake to all modules
        let stake_amounts: Vec<u64> = vec![stake_amount; staked_modules as usize];

        assert_ok!(SubspaceModule::add_stake_multiple(
            get_origin(controler_key),
            netuid,
            keys.clone(),
            stake_amounts.clone(),
        ));

        let total_actual_stake = keys
            .clone()
            .into_iter()
            .map(|k| SubspaceModule::get_stake(netuid, &k))
            .sum::<u64>();
        let staker_balance = SubspaceModule::get_balance(&controler_key);

        assert_eq!(
            total_actual_stake, total_stake,
            "total stake should be equal to the sum of all stakes"
        );
        assert_eq!(
            staker_balance,
            og_staker_balance - total_stake,
            "staker balance should be 0"
        );

        // unstake from all modules
        assert_ok!(SubspaceModule::remove_stake_multiple(
            get_origin(controler_key),
            netuid,
            keys.clone(),
            stake_amounts.clone(),
        ));

        let total_actual_stake = keys
            .clone()
            .into_iter()
            .map(|k| SubspaceModule::get_stake(netuid, &k))
            .sum::<u64>();
        let staker_balance = SubspaceModule::get_balance(&controler_key);
        assert_eq!(
            total_actual_stake, 0,
            "total stake should be equal to the sum of all stakes"
        );
        assert_eq!(
            staker_balance, og_staker_balance,
            "staker balance should be 0"
        );
    });
}

#[test]
fn test_transfer_stake() {
    new_test_ext().execute_with(|| {
        let n: u16 = 10;
        let stake_amount: u64 = 10_000_000_000;
        let netuid: u16 = 0;

        register_n_modules(netuid, n, stake_amount);

        let keys: Vec<U256> = SubspaceModule::get_keys(netuid);
        assert_ok!(SubspaceModule::transfer_stake(
            get_origin(keys[0]),
            netuid,
            keys[0],
            keys[1],
            stake_amount
        ));

        let key0_stake = SubspaceModule::get_stake(netuid, &keys[0]);
        let key1_stake = SubspaceModule::get_stake(netuid, &keys[1]);
        assert_eq!(key0_stake, 0);
        assert_eq!(key1_stake, stake_amount * 2);
        assert_ok!(SubspaceModule::transfer_stake(
            get_origin(keys[0]),
            netuid,
            keys[1],
            keys[0],
            stake_amount
        ));

        let key0_stake = SubspaceModule::get_stake(netuid, &keys[0]);
        let key1_stake = SubspaceModule::get_stake(netuid, &keys[1]);
        assert_eq!(key0_stake, stake_amount);
        assert_eq!(key1_stake, stake_amount);
    });
}

#[test]
fn test_delegate_stake() {
    new_test_ext().execute_with(|| {
        let max_uids: u16 = 10;
        let token_amount: u64 = 1_000_000_000;
        let netuids: Vec<u16> = [0, 1, 2, 3].to_vec();
        let staked_amounts: Vec<u64> = netuids.iter().map(|_i| 10 * token_amount).collect();
        let mut total_stake: u64 = 0;
        let mut subnet_stake: u64 = 0;

        for netuid in netuids.into_iter() {
            let staked_amount = staked_amounts[netuid as usize];
            let keys =
                (0..max_uids).map(|i| U256::from(i + max_uids * netuid)).collect::<Vec<U256>>();
            let delegate_keys: Vec<U256> = keys.iter().map(|i| (*i + 1)).collect();

            for (i, key) in keys.iter().enumerate() {
                let delegate_key: U256 = delegate_keys[i];
                add_balance(delegate_key, staked_amount + 1);

                assert_ok!(register_module(netuid, *key, 0));
                assert_ok!(SubspaceModule::add_stake(
                    get_origin(delegate_key),
                    netuid,
                    *key,
                    staked_amount
                ));

                let uid = SubspaceModule::get_uid_for_key(netuid, key);
                assert_eq!(
                    SubspaceModule::get_stake_for_uid(netuid, uid),
                    staked_amount
                );
                assert_eq!(SubspaceModule::get_balance(&delegate_key), 1);
                assert_eq!(
                    SubspaceModule::get_stake_to_vector(netuid, &delegate_key).len(),
                    1
                );
                // REMOVE STAKE
                assert_ok!(SubspaceModule::remove_stake(
                    get_origin(delegate_key),
                    netuid,
                    *key,
                    staked_amount
                ));
                assert_eq!(
                    SubspaceModule::get_balance(&delegate_key),
                    staked_amount + 1
                );
                assert_eq!(SubspaceModule::get_stake_for_uid(netuid, uid), 0);
                assert_eq!(
                    SubspaceModule::get_stake_to_vector(netuid, &delegate_key).len(),
                    0
                );

                // ADD STAKE AGAIN LOL
                assert_ok!(SubspaceModule::add_stake(
                    get_origin(delegate_key),
                    netuid,
                    *key,
                    staked_amount
                ));
                assert_eq!(
                    SubspaceModule::get_stake_for_uid(netuid, uid),
                    staked_amount
                );
                assert_eq!(SubspaceModule::get_balance(&delegate_key), 1);
                assert_eq!(
                    SubspaceModule::get_stake_to_vector(netuid, &delegate_key).len(),
                    1
                );

                // AT THE END WE SHOULD HAVE THE SAME TOTAL STAKE
                subnet_stake += SubspaceModule::get_stake_for_uid(netuid, uid);
            }
            assert_eq!(SubspaceModule::get_total_subnet_stake(netuid), subnet_stake);
            total_stake += subnet_stake;
            assert_eq!(SubspaceModule::total_stake(), total_stake);
            subnet_stake = 0;
        }
    });
}

#[test]
fn test_ownership_ratio() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 0;
        let num_modules: u16 = 10;
        let stake_per_module: u64 = 1_000_000_000;
        register_n_modules(netuid, num_modules, 0);

        let keys = SubspaceModule::get_keys(netuid);

        for k in &keys {
            let delegate_keys: Vec<U256> =
                (0..num_modules).map(|i| U256::from(i + num_modules + 1)).collect();
            for delegate_key in delegate_keys.iter() {
                add_balance(*delegate_key, stake_per_module + 1);
            }

            let pre_delegate_stake = SubspaceModule::get_stake_from_vector(netuid, k);
            assert_eq!(pre_delegate_stake.len(), 1); // +1 for the module itself, +1 for the delegate key on

            for (i, delegate_key) in delegate_keys.iter().enumerate() {
                assert_ok!(SubspaceModule::add_stake(
                    get_origin(*delegate_key),
                    netuid,
                    *k,
                    stake_per_module
                ));
                let stake_from_vector = SubspaceModule::get_stake_from_vector(netuid, k);
                assert_eq!(stake_from_vector.len(), pre_delegate_stake.len() + i + 1);
            }

            let ownership_ratios = SubspaceModule::get_ownership_ratios(netuid, k);

            assert_eq!(ownership_ratios.len(), delegate_keys.len() + 1);
            step_epoch(netuid);

            let stake_from_vector = SubspaceModule::get_stake_from_vector(netuid, k);
            let stake: u64 = SubspaceModule::get_stake(netuid, k);
            let sum: u64 = stake_from_vector.iter().fold(0, |acc, (_, x)| acc + x);

            assert_eq!(stake, sum);
        }
    });
}

#[test]
fn test_min_stake() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 0;
        let num_modules: u16 = 10;
        let min_stake: u64 = 10_000_000_000;

        register_n_modules(netuid, num_modules, min_stake);
        let keys = SubspaceModule::get_keys(netuid);

        SubspaceModule::set_min_stake(netuid, min_stake - 100);

        assert_ok!(SubspaceModule::remove_stake(
            get_origin(keys[0]),
            netuid,
            keys[0],
            10_000_000_000
        ));
    });
}
