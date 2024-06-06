use super::*;

use sp_arithmetic::per_things::Percent;
use sp_runtime::DispatchError;
use sp_std::{collections::btree_map::BTreeMap, vec::Vec};

impl<T: Config> Pallet<T> {
    /// Adds stake to multiple modules in a single transaction
    pub fn do_add_stake_multiple(
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
        // the reason for this check at staking is that it has no fee,
        // in transfer multiple, this is not needed, as user pays gass
        ensure!(amounts.len() <= 100, Error::<T>::TooManyKeys);

        // --- 3. Check if the caller has enough balance to stake
        let total_amount: u64 = amounts.iter().sum();
        ensure!(
            Self::has_enough_balance(&key, total_amount),
            Error::<T>::NotEnoughStakeToWithdraw
        );

        // --- 4. Add stake to each module
        for (m_key, amount) in module_keys.iter().zip(amounts.iter()) {
            // do not allow zero amounts in add_stake
            Self::do_add_stake(origin.clone(), m_key.clone(), *amount)?;
        }

        // --- 5. Done and ok
        Ok(())
    }

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

        // --- 4. Make sure we can convert to balance
        let stake_to_be_added_as_currency = Self::u64_to_balance(amount);
        ensure!(
            stake_to_be_added_as_currency.is_some(),
            Error::<T>::CouldNotConvertToBalance
        );

        // -- 5. Check before values
        let stake_before_remove: u64 = Self::get_stake_to_module(&key, &module_key.clone());
        let balance_before_remove: u64 = Self::get_balance_u64(&key);
        let module_stake_before_remove: u64 = Stake::<T>::get(&module_key);

        // --- 6. We remove the balance from the key.
        Self::decrease_stake(Some(&key), &module_key, Some(amount), false);

        // --- 7. We add the balancer to the key. If the above fails we will not credit this key.
        Self::add_balance_to_account(&key, Self::u64_to_balance(amount).unwrap());

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
            balance_after_remove == balance_before_remove.saturating_add(amount),
            Error::<T>::BalanceNotAdded
        );
        ensure!(
            module_stake_after_remove == module_stake_before_remove.saturating_sub(amount),
            Error::<T>::StakeNotRemoved
        );

        Self::deposit_event(Event::StakeRemoved(key, module_key, amount));

        // --- 10. Done and ok.
        Ok(())
    }

    /// Returns the total amount of stake in the staking table.

    /// Returns the total amount of stake in the staking table.
    pub fn total_stake() -> u64 {
        TotalStake::<T>::get()
    }

    pub fn get_total_subnet_stake(netuid: u16) -> u64 {
        let mut total_stake = 0;

        for (_, account_id) in Keys::<T>::iter_prefix(netuid) {
            let stake = Stake::<T>::get(account_id);
            total_stake += stake;
        }

        total_stake
    }

    // Returns the delegation fee of a module
    pub fn get_delegation_fee(netuid: u16, module_key: &T::AccountId) -> Percent {
        let min_deleg_fee_global = FloorDelegationFee::<T>::get();
        let delegation_fee = DelegationFee::<T>::get(netuid, module_key);

        delegation_fee.max(min_deleg_fee_global)
    }

    pub fn has_enough_stake(key: &T::AccountId, module_key: &T::AccountId, amount: u64) -> bool {
        amount > 0 && Self::get_stake_to_module(key, module_key) >= amount
    }

    pub fn get_stake_to_module(key: &T::AccountId, module_key: &T::AccountId) -> u64 {
        StakeTo::<T>::get(key, module_key)
    }

    pub fn get_stake_to_vector(key: &T::AccountId) -> BTreeMap<T::AccountId, u64> {
        StakeTo::<T>::iter_prefix(key).collect()
    }

    pub fn set_stake_to_vector(key: &T::AccountId, stake_to_vector: BTreeMap<T::AccountId, u64>) {
        StakeTo::<T>::remove_prefix(key, None);
        for (k, v) in stake_to_vector.iter() {
            StakeTo::<T>::insert(key, k, v);
        }
    }

    pub fn set_stake_from_vector(
        module_key: &T::AccountId,
        stake_from_vector: BTreeMap<T::AccountId, u64>,
    ) {
        StakeFrom::<T>::remove_prefix(module_key, None);
        for (k, v) in stake_from_vector.iter() {
            StakeFrom::<T>::insert(module_key, k, v);
        }
    }

    pub fn get_stake_from_vector(module_key: &T::AccountId) -> BTreeMap<T::AccountId, u64> {
        StakeFrom::<T>::iter_prefix(module_key).collect::<BTreeMap<_, _>>()
    }

    pub fn get_total_stake_to(key: &T::AccountId) -> u64 {
        Self::get_stake_to_vector(key).into_values().sum()
    }

    pub fn increase_stake(key: &T::AccountId, module_key: &T::AccountId, amount: u64) -> bool {
        let mut stake_from_vector = Self::get_stake_from_vector(module_key);
        let found_key_in_vector = stake_from_vector.iter_mut().find(|(k, _)| *k == key);
        if let Some((_, v)) = found_key_in_vector {
            *v = v.saturating_add(amount);
        } else {
            stake_from_vector.insert(key.clone(), amount);
        }

        // reset the stake to vector, as we have updated the stake_to_vector
        let mut stake_to_vector = Self::get_stake_to_vector(key);
        let found_key_in_vector = stake_to_vector.iter_mut().find(|(k, _)| *k == module_key);
        if let Some((_, v)) = found_key_in_vector {
            *v = v.saturating_add(amount);
        } else {
            stake_to_vector.insert(module_key.clone(), amount);
        }

        Self::set_stake_to_vector(key, stake_to_vector);
        Self::set_stake_from_vector(module_key, stake_from_vector);

        Stake::<T>::mutate(module_key, |stake| *stake = stake.saturating_add(amount));
        TotalStake::<T>::mutate(|total_stake| *total_stake = total_stake.saturating_add(amount));

        true
    }

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
        let stake_from_vector = Self::get_stake_from_vector(module_key);

        for (&ref delegate_key, delegate_stake_amount) in &stake_from_vector {
            Self::reduce_stake_internal(&delegate_key, module_key, *delegate_stake_amount);
            if add_balance {
                Self::reward_balance_to_account(&delegate_key, *delegate_stake_amount)?;
            }
            total_unstaked_amount += delegate_stake_amount;
        }

        StakeFrom::<T>::remove_prefix(module_key, None);
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
            &mut |module_key| Self::get_stake_from_vector(module_key),
            &mut Self::set_stake_from_vector,
        );

        // Update stake_to_vector
        Self::update_stake(
            module_key,
            key,
            amount,
            &mut |key| Self::get_stake_to_vector(key),
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

    // Decreases the stake by the amount while decreasing other counters.
    pub fn remove_stake_from_storage(module_key: &T::AccountId) {
        let stake_from_vector = Self::get_stake_from_vector(module_key);
        for (delegate_key, delegate_stake_amount) in stake_from_vector.iter() {
            Self::decrease_stake(
                Some(&delegate_key),
                module_key,
                Some(*delegate_stake_amount),
                false,
            );
            Self::add_balance_to_account(
                delegate_key,
                Self::u64_to_balance(*delegate_stake_amount).unwrap(),
            );
        }

        StakeFrom::<T>::remove_prefix(module_key, None);
        Stake::<T>::remove(module_key);
    }

    pub fn get_balance(key: &T::AccountId) -> BalanceOf<T> {
        T::Currency::free_balance(key)
    }

    // gets the overall stake value for a given account_id,
    pub fn get_account_stake(account_id: &T::AccountId) -> u64 {
        StakeTo::<T>::iter_prefix_values(account_id).sum()
    }
}
