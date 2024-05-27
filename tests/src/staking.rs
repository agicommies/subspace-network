// mod mock;

// use frame_support::{assert_noop, assert_ok};
// use log::info;
// use mock::*;
// use pallet_subspace::{Error, MaxRegistrationsPerBlock};
// use sp_core::U256;
// use substrate_fixed::types::I64F64;
// mod mock;

// use frame_support::assert_ok;
// use log::info;
// use mock::*;
// use pallet_subspace::{Dividends, Emission, Incentive, Tempo};
// use sp_core::U256;
// use substrate_fixed::types::I64F64;

// // /***********************************************************
// // 	staking::add_stake() tests
// // ************************************************************/
// #[test]
// fn test_stake() {
//     new_test_ext().execute_with(|| {
//         let max_uids: u16 = 10;
//         let netuids: [u16; 4] = core::array::from_fn(|i| i as u16);
//         let amount_staked_vector: Vec<u64> = netuids.iter().map(|_| to_nano(10)).collect();
//         let mut total_stake: u64 = 0;
//         let mut subnet_stake: u64 = 0;
//         // make sure that the results won´t get affected by burn
//         zero_min_burn();
//         MaxRegistrationsPerBlock::<Test>::set(1000);

//         for netuid in netuids {
//             info!("NETUID: {}", netuid);
//             let amount_staked = amount_staked_vector[netuid as usize];
//             let key_vector: Vec<U256> =
//                 (0..max_uids).map(|i| U256::from(i + max_uids * netuid)).collect();

//             for key in key_vector.iter() {
//                 info!(
//                     " KEY {} KEY STAKE {} STAKING AMOUNT {} ",
//                     key,
//                     SubspaceModule::get_stake(key),
//                     amount_staked
//                 );

//                 assert_ok!(register_module(netuid, *key, amount_staked));
//                 info!(
//                     " KEY STAKE {} STAKING AMOUNT {} ",
//                     SubspaceModule::get_stake(key),
//                     amount_staked
//                 );

//                 // SubspaceModule::add_stake(get_origin(*key), netuid, amount_staked);
//                 assert_eq!(SubspaceModule::get_stake(key), amount_staked);
//                 assert_eq!(SubspaceModule::get_balance(key), 1);

//                 // REMOVE STAKE
//                 assert_ok!(SubspaceModule::remove_stake(
//                     get_origin(*key),
//                     *key,
//                     amount_staked
//                 ));
//                 assert_eq!(SubspaceModule::get_balance(key), amount_staked + 1);
//                 assert_eq!(SubspaceModule::get_stake(key), 0);

//                 // ADD STAKE AGAIN LOL
//                 assert_ok!(SubspaceModule::add_stake(
//                     get_origin(*key),
//                     *key,
//                     amount_staked
//                 ));
//                 assert_eq!(SubspaceModule::get_stake(key), amount_staked);
//                 assert_eq!(SubspaceModule::get_balance(key), 1);

//                 // AT THE END WE SHOULD HAVE THE SAME TOTAL STAKE
//                 subnet_stake += SubspaceModule::get_stake(key);
//             }
//             assert_eq!(SubspaceModule::get_total_subnet_stake(netuid), subnet_stake);
//             total_stake += subnet_stake;
//             assert_eq!(SubspaceModule::total_stake(), total_stake);
//             subnet_stake = 0;
//             info!("TOTAL STAKE: {}", total_stake);
//             info!(
//                 "TOTAL SUBNET STAKE: {}",
//                 SubspaceModule::get_total_subnet_stake(netuid)
//             );
//         }
//     });
// }

// #[test]
// fn test_multiple_stake() {
//     new_test_ext().execute_with(|| {
//         let n: u16 = 10;
//         let stake_amount: u64 = 10_000_000_000;
//         let _total_stake: u64 = 0;
//         let netuid: u16 = 0;
//         let _subnet_stake: u64 = 0;
//         let _uid: u16 = 0;
//         let num_staked_modules: u16 = 10;
//         let total_stake: u64 = stake_amount * num_staked_modules as u64;
//         // make sure that the results won´t get affected by burn
//         zero_min_burn();

//         register_n_modules(netuid, n, 10);
//         let controler_key = U256::from(n + 1);
//         let og_staker_balance: u64 = total_stake + 1;
//         add_balance(controler_key, og_staker_balance);

//         let keys: Vec<U256> = SubspaceModule::get_keys(netuid);

//         // stake to all modules

//         let stake_amounts: Vec<u64> = vec![stake_amount; num_staked_modules as usize];

//         info!("STAKE AMOUNTS: {stake_amounts:?}");
//         let total_actual_stake: u64 =
//             keys.clone().into_iter().map(|k| SubspaceModule::get_stake(&k)).sum();
//         let staker_balance = SubspaceModule::get_balance(&controler_key);
//         info!("TOTAL ACTUAL STAKE: {total_actual_stake}");
//         info!("TOTAL STAKE: {total_stake}");
//         info!("STAKER BALANCE: {staker_balance}");
//         assert_ok!(SubspaceModule::add_stake_multiple(
//             get_origin(controler_key),
//             keys.clone(),
//             stake_amounts.clone(),
//         ));

//         let total_actual_stake: u64 =
//             keys.clone().into_iter().map(|k| SubspaceModule::get_stake(&k)).sum();
//         let staker_balance = SubspaceModule::get_balance(&controler_key);

//         assert_eq!(
//             total_actual_stake,
//             total_stake + (n as u64 * 10),
//             "total stake should be equal to the sum of all stakes"
//         );
//         assert_eq!(
//             staker_balance,
//             og_staker_balance - total_stake,
//             "staker balance should be 0"
//         );

//         // unstake from all modules
//         assert_ok!(SubspaceModule::remove_stake_multiple(
//             get_origin(controler_key),
//             keys.clone(),
//             stake_amounts.clone(),
//         ));

//         let total_actual_stake: u64 =
//             keys.clone().into_iter().map(|k| SubspaceModule::get_stake(&k)).sum();
//         let staker_balance = SubspaceModule::get_balance(&controler_key);
//         assert_eq!(
//             total_actual_stake,
//             n as u64 * 10,
//             "total stake should be equal to the sum of all stakes"
//         );
//         assert_eq!(
//             staker_balance, og_staker_balance,
//             "staker balance should be 0"
//         );
//     });
// }

// #[test]
// fn test_transfer_stake() {
//     new_test_ext().execute_with(|| {
//         let n: u16 = 10;
//         let stake_amount: u64 = 10_000_000_000;
//         let netuid: u16 = 0;
//         zero_min_burn();

//         register_n_modules(netuid, n, stake_amount);

//         let keys: Vec<U256> = SubspaceModule::get_keys(netuid);

//         assert_ok!(SubspaceModule::transfer_stake(
//             get_origin(keys[0]),
//             keys[0],
//             keys[1],
//             stake_amount
//         ));

//         let key0_stake = SubspaceModule::get_stake(&keys[0]);
//         let key1_stake = SubspaceModule::get_stake(&keys[1]);
//         assert_eq!(key0_stake, 0);
//         assert_eq!(key1_stake, stake_amount * 2);

//         assert_ok!(SubspaceModule::transfer_stake(
//             get_origin(keys[0]),
//             keys[1],
//             keys[0],
//             stake_amount
//         ));

//         let key0_stake = SubspaceModule::get_stake(&keys[0]);
//         let key1_stake = SubspaceModule::get_stake(&keys[1]);
//         assert_eq!(key0_stake, stake_amount);
//         assert_eq!(key1_stake, stake_amount);
//     });
// }

// #[test]
// fn test_delegate_stake() {
//     new_test_ext().execute_with(|| {
//         let max_uids: u16 = 10;
//         let netuids: Vec<u16> = [0, 1, 2, 3].to_vec();
//         let amount_staked_vector: Vec<u64> = netuids.iter().map(|_i| to_nano(10)).collect();
//         let mut total_stake: u64 = 0;
//         let mut subnet_stake: u64 = 0;
//         // make sure that the results won´t get affected by burn
//         zero_min_burn();
//         MaxRegistrationsPerBlock::<Test>::set(1000);

//         for i in netuids.iter() {
//             let netuid = *i;
//             info!("NETUID: {}", netuid);
//             let amount_staked = amount_staked_vector[netuid as usize];
//             let key_vector: Vec<U256> =
//                 (0..max_uids).map(|i| U256::from(i + max_uids * netuid)).collect();
//             let delegate_key_vector: Vec<U256> = key_vector.iter().map(|i| (*i + 1)).collect();

//             for (i, key) in key_vector.iter().enumerate() {
//                 info!(
//                     " KEY {} KEY STAKE {} STAKING AMOUNT {} ",
//                     key,
//                     SubspaceModule::get_stake(key),
//                     amount_staked
//                 );

//                 let delegate_key: U256 = delegate_key_vector[i];
//                 add_balance(delegate_key, amount_staked + 1);

//                 assert_ok!(register_module(netuid, *key, 10));
//                 info!(
//                     " DELEGATE KEY STAKE {} STAKING AMOUNT {} ",
//                     SubspaceModule::get_stake(&delegate_key),
//                     amount_staked
//                 );

//                 assert_ok!(SubspaceModule::add_stake(
//                     get_origin(delegate_key),
//                     *key,
//                     amount_staked
//                 ));
//                 let uid = SubspaceModule::get_uid_for_key(netuid, key);
//                 // SubspaceModule::add_stake(get_origin(*key), netuid, amount_staked);
//                 assert_eq!(get_stake_for_uid(netuid, uid), amount_staked + 10);
//                 assert_eq!(SubspaceModule::get_balance(&delegate_key), 1);
//                 assert_eq!(SubspaceModule::get_stake_to_vector(&delegate_key).len(), 1);
//                 // REMOVE STAKE
//                 assert_ok!(SubspaceModule::remove_stake(
//                     get_origin(delegate_key),
//                     *key,
//                     amount_staked
//                 ));
//                 assert_eq!(
//                     SubspaceModule::get_balance(&delegate_key),
//                     amount_staked + 1
//                 );
//                 assert_eq!(get_stake_for_uid(netuid, uid), 10);
//                 assert_eq!(SubspaceModule::get_stake_to_vector(&delegate_key).len(), 0);

//                 // ADD STAKE AGAIN
//                 assert_ok!(SubspaceModule::add_stake(
//                     get_origin(delegate_key),
//                     *key,
//                     amount_staked
//                 ));
//                 assert_eq!(get_stake_for_uid(netuid, uid), amount_staked + 10);
//                 assert_eq!(SubspaceModule::get_balance(&delegate_key), 1);
//                 assert_eq!(SubspaceModule::get_stake_to_vector(&delegate_key).len(), 1);

//                 // AT THE END WE SHOULD HAVE THE SAME TOTAL STAKE
//                 subnet_stake += get_stake_for_uid(netuid, uid);
//             }
//             assert_eq!(SubspaceModule::get_total_subnet_stake(netuid), subnet_stake);
//             total_stake += subnet_stake;
//             assert_eq!(SubspaceModule::total_stake(), total_stake);
//             subnet_stake = 0;
//             info!("TOTAL STAKE: {}", total_stake);
//             info!(
//                 "TOTAL SUBNET STAKE: {}",
//                 SubspaceModule::get_total_subnet_stake(netuid)
//             );
//         }
//     });
// }

// #[test]
// fn test_ownership_ratio() {
//     new_test_ext().execute_with(|| {
//         let netuid: u16 = 0;
//         let num_modules: u16 = 10;
//         let stake_per_module: u64 = 1_000_000_000;
//         // make sure that the results won´t get affected by burn
//         zero_min_burn();

//         register_n_modules(netuid, num_modules, 10);

//         let keys = SubspaceModule::get_keys(netuid);

//         for k in &keys {
//             let delegate_keys: Vec<U256> =
//                 (0..num_modules).map(|i| U256::from(i + num_modules + 1)).collect();
//             for d in delegate_keys.iter() {
//                 add_balance(*d, stake_per_module + 1);
//             }

//             let pre_delegate_stake_from_vector = SubspaceModule::get_stake_from_vector(k);
//             assert_eq!(pre_delegate_stake_from_vector.len(), 1); // +1 for the module itself, +1
// for the delegate key on

//             info!("KEY: {}", k);
//             for (i, d) in delegate_keys.iter().enumerate() {
//                 info!("DELEGATE KEY: {d}");
//                 assert_ok!(SubspaceModule::add_stake(
//                     get_origin(*d),
//                     *k,
//                     stake_per_module
//                 ));
//                 let stake_from_vector = SubspaceModule::get_stake_from_vector(k);
//                 assert_eq!(
//                     stake_from_vector.len(),
//                     pre_delegate_stake_from_vector.len() + i + 1
//                 );
//             }
//             let ownership_ratios: Vec<(U256, I64F64)> =
//                 SubspaceModule::get_ownership_ratios(netuid, k);

//             assert_eq!(ownership_ratios.len(), delegate_keys.len() + 1);
//             info!("OWNERSHIP RATIOS: {ownership_ratios:?}");

//             step_epoch(netuid);

//             let stake_from_vector = SubspaceModule::get_stake_from_vector(k);
//             let stake: u64 = SubspaceModule::get_stake(k);
//             let sumed_stake: u64 = stake_from_vector.iter().fold(0, |acc, (_a, x)| acc + x);
//             let total_stake: u64 = SubspaceModule::get_total_subnet_stake(netuid);

//             info!("STAKE: {}", stake);
//             info!("SUMED STAKE: {sumed_stake}");
//             info!("TOTAL STAKE: {total_stake}");

//             assert_eq!(stake, sumed_stake);

//             // for (d_a, o) in ownership_ratios.iter() {
//             //     info!("OWNERSHIP RATIO: {}", o);

//             // }
//         }
//     });
// }

// #[test]
// fn test_min_stake() {
//     new_test_ext().execute_with(|| {
//         let netuid: u16 = 0;
//         let num_modules: u16 = 10;
//         let min_stake: u64 = 10_000_000_000;
//         // make sure that the results won´t get affected by burn
//         zero_min_burn();

//         register_n_modules(netuid, num_modules, min_stake);
//         let keys = SubspaceModule::get_keys(netuid);

//         update_params!(netuid => { min_stake: min_stake - 100 });

//         assert_ok!(SubspaceModule::remove_stake(
//             get_origin(keys[0]),
//             keys[0],
//             10_000_000_000
//         ));
//     });
// }

// #[test]
// fn test_stake_zero() {
//     new_test_ext().execute_with(|| {
//         // Register the general subnet.
//         let netuid: u16 = 0;
//         let key = U256::from(0);
//         let stake_amount: u64 = to_nano(1_000);

//         // Make sure registration cost is not affected
//         zero_min_burn();

//         assert_ok!(register_module(netuid, key, stake_amount));

//         // try to stake zero
//         let key_two = U256::from(1);

//         assert_noop!(
//             SubspaceModule::do_add_stake(get_origin(key_two), key, 0),
//             Error::<Test>::NotEnoughBalanceToStake
//         );
//     });
// }

// #[test]
// fn test_unstake_zero() {
//     new_test_ext().execute_with(|| {
//         // Register the general subnet.
//         let netuid: u16 = 0;
//         let key = U256::from(0);
//         let stake_amount: u64 = to_nano(1_000);

//         // Make sure registration cost is not affected
//         zero_min_burn();

//         assert_ok!(register_module(netuid, key, stake_amount));

//         // try to unstake zero
//         let key_two = U256::from(1);

//         assert_noop!(
//             SubspaceModule::do_remove_stake(get_origin(key_two), key, 0),
//             Error::<Test>::NotEnoughStakeToWithdraw
//         );
//     });
// }

// #[test]
// fn test_ownership_ratio() {
//     new_test_ext().execute_with(|| {
//         let netuid: u16 = 0;
//         let num_modules: u16 = 10;
//         let tempo = 1;
//         let stake_per_module: u64 = 1_000_000_000;
//         // make sure that the results won´t get affected by burn
//         zero_min_burn();

//         register_n_modules(netuid, num_modules, stake_per_module);
//         Tempo::<Test>::insert(netuid, tempo);

//         let keys = SubspaceModule::get_keys(netuid);
//         let voter_key = keys[0];
//         let miner_keys = keys[1..].to_vec();
//         let miner_uids: Vec<u16> =
//             miner_keys.iter().map(|k| SubspaceModule::get_uid_for_key(netuid, k)).collect();
//         let miner_weights = vec![1; miner_uids.len()];

//         let delegate_keys: Vec<U256> =
//             (0..num_modules).map(|i| U256::from(i + num_modules + 1)).collect();
//         for d in delegate_keys.iter() {
//             add_balance(*d, stake_per_module + 1);
//         }

//         let pre_delegate_stake_from_vector = SubspaceModule::get_stake_from_vector(&voter_key);
//         assert_eq!(pre_delegate_stake_from_vector.len(), 1); // +1 for the module itself, +1 for
// the delegate key on

//         for (i, d) in delegate_keys.iter().enumerate() {
//             info!("DELEGATE KEY: {d}");
//             assert_ok!(SubspaceModule::add_stake(
//                 get_origin(*d),
//                 voter_key,
//                 stake_per_module
//             ));
//             let stake_from_vector = SubspaceModule::get_stake_from_vector(&voter_key);
//             assert_eq!(
//                 stake_from_vector.len(),
//                 pre_delegate_stake_from_vector.len() + i + 1
//             );
//         }
//         let ownership_ratios: Vec<(U256, I64F64)> =
//             SubspaceModule::get_ownership_ratios(netuid, &voter_key);
//         assert_eq!(ownership_ratios.len(), delegate_keys.len() + 1);

//         let founder_tokens_before = SubspaceModule::get_balance(&voter_key)
//             + SubspaceModule::get_stake_to_module(&voter_key, &voter_key);

//         let delegate_balances_before =
//             delegate_keys.iter().map(SubspaceModule::get_balance).collect::<Vec<u64>>();
//         let delegate_stakes_before = delegate_keys
//             .iter()
//             .map(|k| SubspaceModule::get_stake_to_module(k, &voter_key))
//             .collect::<Vec<u64>>();
//         let delegate_total_tokens_before = delegate_balances_before
//             .iter()
//             .zip(delegate_stakes_before.clone())
//             .map(|(a, x)| a + x)
//             .sum::<u64>();

//         let total_balance = keys
//             .iter()
//             .map(SubspaceModule::get_balance)
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_stake = keys
//             .iter()
//             .map(|k| SubspaceModule::get_stake_to_module(k, k))
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_delegate_stake = delegate_keys
//             .iter()
//             .map(|k| SubspaceModule::get_stake_to_module(k, &voter_key))
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_delegate_balance = delegate_keys
//             .iter()
//             .map(SubspaceModule::get_balance)
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_tokens_before =
//             total_balance + total_stake + total_delegate_stake + total_delegate_balance;
//         info!("total_tokens_before: {total_tokens_before:?}");

//         info!("delegate_balances before: {delegate_balances_before:?}");
//         info!("delegate_stakes before: {delegate_stakes_before:?}");
//         info!("delegate_total_tokens before: {delegate_total_tokens_before:?}");

//         let result = SubspaceModule::set_weights(
//             get_origin(voter_key),
//             netuid,
//             miner_uids.clone(),
//             miner_weights.clone(),
//         );

//         assert_ok!(result);

//         step_epoch(netuid);

//         let dividends = Dividends::<Test>::get(netuid);
//         let incentives = Incentive::<Test>::get(netuid);
//         let emissions = Emission::<Test>::get(netuid);

//         info!("dividends: {dividends:?}");
//         info!("incentives: {incentives:?}");
//         info!("emissions: {emissions:?}");
//         let total_emissions = emissions.iter().sum::<u64>();

//         info!("total_emissions: {total_emissions:?}");

//         let delegate_balances =
//             delegate_keys.iter().map(SubspaceModule::get_balance).collect::<Vec<u64>>();
//         let delegate_stakes = delegate_keys
//             .iter()
//             .map(|k| SubspaceModule::get_stake_to_module(k, &voter_key))
//             .collect::<Vec<u64>>();
//         let delegate_total_tokens = delegate_balances
//             .iter()
//             .zip(delegate_stakes.clone())
//             .map(|(a, x)| a + x)
//             .sum::<u64>();
//         let founder_tokens = SubspaceModule::get_balance(&voter_key)
//             + SubspaceModule::get_stake_to_module(&voter_key, &voter_key);
//         let founder_new_tokens = founder_tokens - founder_tokens_before;
//         let delegate_new_tokens: Vec<u64> = delegate_stakes
//             .iter()
//             .zip(delegate_stakes_before.clone())
//             .map(|(a, x)| a - x)
//             .collect::<Vec<u64>>();

//         let total_new_tokens = founder_new_tokens + delegate_new_tokens.iter().sum::<u64>();

//         info!("owner_ratios: {ownership_ratios:?}");
//         info!("total_new_tokens: {total_new_tokens:?}");
//         info!("founder_tokens: {founder_tokens:?}");
//         info!("delegate_balances: {delegate_balances:?}");
//         info!("delegate_stakes: {delegate_stakes:?}");
//         info!("delegate_total_tokens: {delegate_total_tokens:?}");
//         info!("founder_new_tokens: {founder_new_tokens:?}");
//         info!("delegate_new_tokens: {delegate_new_tokens:?}");

//         let total_balance = keys
//             .iter()
//             .map(SubspaceModule::get_balance)
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_stake = keys
//             .iter()
//             .map(|k| SubspaceModule::get_stake_to_module(k, k))
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_delegate_stake = delegate_keys
//             .iter()
//             .map(|k| SubspaceModule::get_stake_to_module(k, &voter_key))
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_delegate_balance = delegate_keys
//             .iter()
//             .map(SubspaceModule::get_balance)
//             .collect::<Vec<u64>>()
//             .iter()
//             .sum::<u64>();
//         let total_tokens_after =
//             total_balance + total_stake + total_delegate_stake + total_delegate_balance;
//         let total_new_tokens = total_tokens_after - total_tokens_before;
//         info!("total_tokens_after: {total_tokens_before:?}");
//         info!("total_new_tokens: {total_new_tokens:?}");
//         assert_eq!(total_new_tokens, total_emissions);

//         let stake_from_vector = SubspaceModule::get_stake_from_vector(&voter_key);
//         let _stake: u64 = SubspaceModule::get_stake(&voter_key);
//         let _sumed_stake: u64 = stake_from_vector.iter().fold(0, |acc, (_a, x)| acc + x);
//         let _total_stake: u64 = SubspaceModule::get_total_subnet_stake(netuid);
//         info!("stake_from_vector: {stake_from_vector:?}");
//     });
// }
