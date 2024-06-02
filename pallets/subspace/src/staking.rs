// TODO:
// - Move this file to a seperate Staking pallet
use super::*;

use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

/// Notes:
/// - Due to safety reasons, the function `do_add_stake_multiple` has been removed.
/// - The stake operation doesn't have a fee, hence there were a large number of possible
///   vulnerabilities in batch misuse of the function.
impl<T: Config> Pallet<T> {
    // Moving stake
    // ============

    pub fn do_remove_stake_multiple(
        origin: T::RuntimeOrigin,
        module_keys: Vec<T::AccountId>,
        amounts: Vec<u64>,
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the
        let key = ensure_signed(origin.clone())?;

        // --- 2. Ensure that the lengths of the module_keys and amounts are the same
        ensure!(
            amounts.len() == module_keys.len(),
            Error::<T>::DifferentLengths
        );

        // --- 2.1 make sure that the lengths are not zero
        ensure!(!amounts.is_empty(), Error::<T>::EmptyKeys);

        // -- 2.2 Make sure they are not above 100
        ensure!(amounts.len() <= 100, Error::<T>::TooManyKeys);

        // --- 3. Remove stake from each module
        for (m_key, amount) in module_keys.iter().zip(amounts.iter()) {
            ensure!(
                Self::has_enough_stake(&key, m_key, *amount),
                Error::<T>::NotEnoughStakeToWithdraw
            );
            Self::do_remove_stake(origin.clone(), m_key.clone(), *amount)?;
        }

        // --- 4. Done and ok
        Ok(())
    }

    /// Transfers stake from one module to another
    pub fn do_transfer_stake(
        origin: T::RuntimeOrigin,
        module_key: T::AccountId,
        new_module_key: T::AccountId,
        amount: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the
        let key = ensure_signed(origin.clone())?;

        // --- 2. Check if both modules are registered
        // --- 2.1 old module check
        ensure!(
            Self::is_registered(None, &module_key),
            Error::<T>::NotRegistered
        );
        // --- 2.2 new module check
        ensure!(
            Self::is_registered(None, &new_module_key),
            Error::<T>::NotRegistered
        );

        // --- 3. Check if the caller has enough stake in the old module
        ensure!(
            Self::has_enough_stake(&key, &module_key, amount),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // --- 4. Remove stake from the source module and add it to the destination module
        Self::do_remove_stake(origin.clone(), module_key.clone(), amount)?;
        // don't allow zero stakes
        Self::do_add_stake(origin.clone(), new_module_key, amount)?;

        // --- 5. Done and ok
        Ok(())
    }

    pub fn do_add_stake(
        origin: T::RuntimeOrigin,
        module_key: T::AccountId,
        amount: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the
        // T::AccountId key information.
        let key = ensure_signed(origin)?;

        // --- 2. We check that the module is registered.
        ensure!(
            Self::is_registered(None, &module_key.clone()),
            Error::<T>::NotRegistered
        );

        // --- 3. We check that the caller has enough balance to stake.
        ensure!(
            Self::has_enough_balance(&key, amount),
            Error::<T>::NotEnoughBalanceToStake
        );

        // --- 4. Make sure we can convert to balance
        let removed_balance_as_currency = Self::u64_to_balance(amount);
        ensure!(
            removed_balance_as_currency.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );

        // -- 5. Check before values
        let stake_before_add: u64 = Self::get_stake_to_module(&key, &module_key.clone());
        let balance_before_add: u64 = Self::get_balance_u64(&key);
        let module_stake_before_add: u64 = Stake::<T>::get(&module_key);

        // --- 6. We remove the balance from the key.
        Self::remove_balance_from_account(&key, removed_balance_as_currency.unwrap())?;

        // --- 7. We add the stake to the module.
        Self::increase_stake(&key, &module_key, amount);

        // -- 8. Check after values
        let stake_after_add: u64 = Self::get_stake_to_module(&key, &module_key.clone());
        let balance_after_add: u64 = Self::get_balance_u64(&key);
        let module_stake_after_add = Stake::<T>::get(&module_key);

        // -- 9. Make sure everything went as expected.
        // Otherwise these ensurers will revert the storage changes.
        ensure!(
            stake_after_add == stake_before_add.saturating_add(amount),
            Error::<T>::StakeNotAdded
        );
        ensure!(
            balance_after_add == balance_before_add.saturating_sub(amount),
            Error::<T>::BalanceNotRemoved
        );
        ensure!(
            module_stake_after_add == module_stake_before_add.saturating_add(amount),
            Error::<T>::StakeNotAdded
        );

        Self::deposit_event(Event::StakeAdded(key, module_key, amount));

        // --- 10. Done and ok.
        Ok(())
    }

    pub fn do_remove_stake(
        origin: T::RuntimeOrigin,
        module_key: T::AccountId,
        amount: u64,
    ) -> dispatch::DispatchResult {
        // --- 1. We check the transaction is signed by the caller and retrieve the T::AccountId key
        // information.
        let key = ensure_signed(origin)?;

        // --- 2. We check that the module is registered.
        ensure!(
            Self::is_registered(None, &module_key.clone()),
            Error::<T>::NotRegistered
        );

        // --- 3. We check that the caller has enough stake in the module.
        ensure!(
            Self::has_enough_stake(&key, &module_key, amount),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // -- 5. Check before values
        let stake_before_remove: u64 = Self::get_stake_to_module(&key, &module_key.clone());
        let balance_before_remove: u64 = Self::get_balance_u64(&key);
        let module_stake_before_remove: u64 = Stake::<T>::get(&module_key);

        // --- 6. We remove the balance from the key.
        Self::decrease_stake(Some(&key), &module_key, Some(amount), true)?;

        // --- 8. Check after values
        let stake_after_remove: u64 = Self::get_stake_to_module(&key, &module_key.clone());
        let balance_after_remove: u64 = Self::get_balance_u64(&key);
        let module_stake_after_remove = Stake::<T>::get(&module_key);

        // -- 9. Make sure everything went as expected.
        // Otherwise these ensurers will revert the storage changes.
        ensure!(
            stake_after_remove == stake_before_remove.saturating_sub(amount),
            Error::<T>::StakeNotRemoved
        );
        ensure!(
            module_stake_after_remove == module_stake_before_remove.saturating_sub(amount),
            Error::<T>::StakeNotRemoved
        );
        ensure!(
            balance_after_remove == balance_before_remove.saturating_add(amount),
            Error::<T>::BalanceNotAdded
        );

        Self::deposit_event(Event::StakeRemoved(key, module_key, amount));

        // --- 10. Done and ok.
        Ok(())
    }

    pub fn increase_stake(staker: &T::AccountId, staked: &T::AccountId, amount: u64) -> bool {
        StakeFrom::<T>::mutate(staked, |stake_from| {
            stake_from
                .entry(staker.clone())
                .and_modify(|v| *v = v.saturating_add(amount))
                .or_insert(amount);
        });

        StakeTo::<T>::mutate(staker, |stake_to| {
            stake_to
                .entry(staked.clone())
                .and_modify(|v| *v = v.saturating_add(amount))
                .or_insert(amount);
        });

        Stake::<T>::mutate(staked, |stake| *stake = stake.saturating_add(amount));
        TotalStake::<T>::mutate(|total_stake| *total_stake = total_stake.saturating_add(amount));

        true
    }

    // Stake reducing operations
    // =========================

    // TODO:
    // Luiz refactor
    // Write unit tests on desired behaivor
    pub fn decrease_stake(
        key: Option<&T::AccountId>,
        module_key: &T::AccountId,
        amount: Option<u64>,
        add_balance: bool,
    ) -> Result<(), &'static str> {
        let total_unstaked_amount = match amount {
            None => Self::decrease_stake_all(module_key, add_balance)?,
            Some(amount) => Self::decrease_stake_partial(key, module_key, amount, add_balance)?,
        };

        TotalStake::<T>::mutate(|total_stake| {
            *total_stake = total_stake.saturating_sub(total_unstaked_amount)
        });

        Ok(())
    }

    fn decrease_stake_all(
        module_key: &T::AccountId,
        add_balance: bool,
    ) -> Result<u64, &'static str> {
        let mut total_unstaked_amount = 0;
        let stake_from_vector = StakeFrom::<T>::get(module_key);

        for (&ref delegate_key, delegate_stake_amount) in &stake_from_vector {
            Self::reduce_stake_internal(&delegate_key, module_key, *delegate_stake_amount);
            if add_balance {
                Self::reward_balance_to_account(&delegate_key, *delegate_stake_amount)?;
            }
            total_unstaked_amount += delegate_stake_amount;
        }

        StakeFrom::<T>::remove(module_key);
        Stake::<T>::remove(module_key);

        Ok(total_unstaked_amount)
    }

    fn decrease_stake_partial(
        key: Option<&T::AccountId>,
        module_key: &T::AccountId,
        amount: u64,
        add_balance: bool,
    ) -> Result<u64, &'static str> {
        let key = key.ok_or("Key is required")?;
        Self::reduce_stake_internal(key, module_key, amount);
        if add_balance {
            Self::reward_balance_to_account(key, amount)?;
        }
        Stake::<T>::mutate(module_key, |stake| *stake = stake.saturating_sub(amount));

        Ok(amount)
    }

    fn reduce_stake_internal(key: &T::AccountId, module_key: &T::AccountId, amount: u64) {
        // Update stake_from_vector
        Self::update_stake(
            key,
            module_key,
            amount,
            &mut |module_key| StakeFrom::<T>::get(module_key),
            &mut Self::set_stake_from_vector,
        );

        // Update stake_to_vector
        Self::update_stake(
            module_key,
            key,
            amount,
            &mut |key| StakeTo::<T>::get(key),
            &mut Self::set_stake_to_vector,
        );
    }

    fn update_stake<F, G>(
        key: &T::AccountId,
        module_key: &T::AccountId,
        amount: u64,
        getter: &mut F,
        setter: &mut G,
    ) where
        F: FnMut(&T::AccountId) -> BTreeMap<T::AccountId, u64>,
        G: FnMut(&T::AccountId, BTreeMap<T::AccountId, u64>),
    {
        let mut stake_map = getter(module_key);
        if let Some(v) = stake_map.get_mut(key) {
            let remaining_stake = v.saturating_sub(amount);
            *v = remaining_stake;
        }
        stake_map.retain(|_, v| *v != 0);
        setter(module_key, stake_map);
    }

    // Utils
    // =====

    pub fn has_enough_stake(key: &T::AccountId, module_key: &T::AccountId, amount: u64) -> bool {
        amount > 0 && Self::get_stake_to_module(key, module_key) >= amount
    }

    // Getters
    // =======

    pub fn get_balance(key: &T::AccountId) -> BalanceOf<T> {
        T::Currency::free_balance(key)
    }

    pub fn get_total_stake_to(key: &T::AccountId) -> u64 {
        StakeTo::<T>::get(key).into_values().sum()
    }

    pub fn get_total_subnet_stake(netuid: u16) -> u64 {
        Keys::<T>::iter_prefix(netuid)
            .map(|(_, account_id)| Stake::<T>::get(account_id))
            .sum()
    }

    pub fn get_stake_to_module(key: &T::AccountId, module_key: &T::AccountId) -> u64 {
        StakeTo::<T>::get(key)
            .into_iter()
            .find(|(k, _)| k == module_key)
            .map(|(_, v)| v)
            .unwrap_or(0)
    }

    // Setters
    // =======

    pub fn set_stake_to_vector(key: &T::AccountId, stake_to_vector: BTreeMap<T::AccountId, u64>) {
        if stake_to_vector.is_empty() {
            StakeTo::<T>::remove(key);
        } else {
            StakeTo::<T>::insert(key, stake_to_vector);
        }
    }

    pub fn set_stake_from_vector(
        module_key: &T::AccountId,
        stake_from_vector: BTreeMap<T::AccountId, u64>,
    ) {
        StakeFrom::<T>::insert(module_key, stake_from_vector);
    }
}
