use super::*;
use sp_runtime::DispatchError;

impl<T: Config> Pallet<T> {
    // Moving balance
    // ==============

    pub fn do_transfer_multiple(
        origin: T::RuntimeOrigin,
        destinations: Vec<T::AccountId>,
        amounts: Vec<u64>,
    ) -> dispatch::DispatchResult {
        // --- 1. We check that the transaction is signed by the caller and retrieve the
        let key = ensure_signed(origin.clone())?;

        // --- 2. Ensure that the lengths of the module_keys and amounts are the same
        ensure!(
            amounts.len() == destinations.len(),
            Error::<T>::DifferentLengths
        );

        // --- 3. Check if the caller has enough balance to transfer
        let total_amount: u64 = amounts.iter().sum();
        ensure!(
            Self::has_enough_balance(&key, total_amount), // do not allow zero stakes.
            Error::<T>::NotEnoughBalanceToTransfer
        );

        // --- 4. Transfer balance to each destination
        for (m_key, amount) in destinations.iter().zip(amounts.iter()) {
            ensure!(
                Self::has_enough_balance(&key, *amount), // do not allow zero stakes.
                Error::<T>::NotEnoughBalanceToTransfer
            );
            Self::transfer_balance_to_account(&key, m_key, *amount)?;
        }

        // --- 5. Done and ok
        Ok(())
    }

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

    pub fn add_balance_to_account(key: &T::AccountId, amount: BalanceOf<T>) {
        let _ = T::Currency::deposit_creating(key, amount); // Infallibe
    }

    pub fn remove_balance_from_account(
        key: &T::AccountId,
        amount: BalanceOf<T>,
    ) -> Result<(), DispatchError> {
        let _ = T::Currency::withdraw(
            key,
            amount,
            WithdrawReasons::except(WithdrawReasons::TIP),
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| Error::<T>::BalanceCouldNotBeRemoved)?;

        Ok(())
    }

    pub fn transfer_balance_to_account(
        from: &T::AccountId,
        to: &T::AccountId,
        amount: u64,
    ) -> Result<(), DispatchError> {
        T::Currency::transfer(
            from,
            to,
            Self::u64_to_balance(amount).unwrap(),
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| Error::<T>::NotEnoughBalanceToTransfer)?;

        Ok(())
    }

    pub fn reward_balance_to_account(key: &T::AccountId, amount: u64) -> Result<(), &'static str> {
        let balance = Self::u64_to_balance(amount).ok_or("Failed to convert amount to balance")?;
        Self::add_balance_to_account(key, balance);
        Ok(())
    }

    // Util
    // ====

    pub fn has_enough_balance(key: &T::AccountId, amount: u64) -> bool {
        if amount == 0 {
            false
        } else {
            Self::get_balance_u64(key) >= amount
        }
    }

    pub fn balance_to_u64(x: BalanceOf<T>) -> u64 {
        x.try_into().ok().unwrap()
    }

    pub fn u64_to_balance(x: u64) -> Option<BalanceOf<T>> {
        x.try_into().ok()
    }

    pub fn get_balance_u64(key: &T::AccountId) -> u64 {
        Self::balance_to_u64(Self::get_balance(key))
    }
}
