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
fn test_ownership_ratio() {
    new_test_ext().execute_with(|| {
        let netuid: u16 = 0;
        let num_modules: u16 = 10;
        let tempo = 1;
        let stake_per_module: u64 = 1_000_000_000;

        register_n_modules(netuid, num_modules, stake_per_module);
        SubspaceModule::set_tempo(netuid, tempo);

        let keys = SubspaceModule::get_keys(netuid);
        let voter_key = keys[0];
        let miner_keys = keys[1..].to_vec();
        let delegate_keys = (0..num_modules)
            .map(|key| U256::from(key + num_modules + 1))
            .collect::<Vec<U256>>();
        let miner_uids = miner_keys
            .iter()
            .map(|key| SubspaceModule::get_uid_for_key(netuid, key))
            .collect::<Vec<u16>>();
        let miner_weights = vec![1; miner_uids.len()];

        for key in delegate_keys.iter() {
            add_balance(*key, stake_per_module + 1);
        }

        let pre_delegate_stake_from_vector =
            SubspaceModule::get_stake_from_vector(netuid, &voter_key);
        assert_eq!(pre_delegate_stake_from_vector.len(), 1); // +1 for the module itself, +1 for the delegate key on

        for (i, d) in delegate_keys.iter().enumerate() {
            assert_ok!(SubspaceModule::add_stake(
                get_origin(*d),
                netuid,
                voter_key,
                stake_per_module
            ));

            let stake_from_vector = SubspaceModule::get_stake_from_vector(netuid, &voter_key);
            assert_eq!(
                stake_from_vector.len(),
                pre_delegate_stake_from_vector.len() + i + 1
            );
        }
        let ownership_ratios = SubspaceModule::get_ownership_ratios(netuid, &voter_key);
        assert_eq!(ownership_ratios.len(), delegate_keys.len() + 1);

        let total_balance = keys
            .iter()
            .map(SubspaceModule::get_balance)
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_stake = keys
            .iter()
            .map(|k| SubspaceModule::get_stake_to_module(netuid, k, k))
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_delegate_stake = delegate_keys
            .iter()
            .map(|k| SubspaceModule::get_stake_to_module(netuid, k, &voter_key))
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_delegate_balance = delegate_keys
            .iter()
            .map(SubspaceModule::get_balance)
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_tokens_before =
            total_balance + total_stake + total_delegate_stake + total_delegate_balance;

        assert_ok!(SubspaceModule::set_weights(
            get_origin(voter_key),
            netuid,
            miner_uids.clone(),
            miner_weights.clone(),
        ));

        step_epoch(netuid);

        let emissions = SubspaceModule::get_emissions(netuid);
        let total_emissions = emissions.iter().sum::<u64>();

        let total_balance = keys
            .iter()
            .map(SubspaceModule::get_balance)
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_stake = keys
            .iter()
            .map(|k| SubspaceModule::get_stake_to_module(netuid, k, k))
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_delegate_stake = delegate_keys
            .iter()
            .map(|k| SubspaceModule::get_stake_to_module(netuid, k, &voter_key))
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_delegate_balance = delegate_keys
            .iter()
            .map(SubspaceModule::get_balance)
            .collect::<Vec<u64>>()
            .iter()
            .sum::<u64>();
        let total_tokens_after =
            total_balance + total_stake + total_delegate_stake + total_delegate_balance;
        let total_new_tokens = total_tokens_after - total_tokens_before;
        assert_eq!(total_new_tokens, total_emissions);
    });
}
