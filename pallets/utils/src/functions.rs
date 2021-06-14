use super::*;

impl<T: utils::Config> CommonUtils for utils::Pallet<T> {
    type AccountId = T::AccountId;

    fn generate_random(sender: Self::AccountId, salt: &RandomSalt) -> U256 {
        let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
        let payload = (
            random_seed,
            sender.clone(),
            salt,
            frame_system::Pallet::<T>::block_number(),
        );
        payload.using_encoded(blake2_256).into()
    }
}

impl<T: utils::Config> LockableOperations for utils::Pallet<T> {
    type AccountId = T::AccountId;
    type BalanceOf = <<T as utils::Config>::Currency as Currency<
        <T as frame_system::Config>::AccountId,
    >>::Balance;

    fn lock_capital(
        sender: &Self::AccountId,
        identifier: LockIdentifier,
        amount: Self::BalanceOf,
        reason: Option<WithdrawReasons>,
        emit_event: bool,
    ) {
        T::Currency::set_lock(
            identifier,
            sender,
            amount,
            reason.unwrap_or(WithdrawReasons::all()),
        );
        if emit_event {
            Self::deposit_event(Event::Locked(sender.clone(), identifier, amount));
        }
    }

    fn extend_lock(
        sender: &Self::AccountId,
        identifier: LockIdentifier,
        amount: Self::BalanceOf,
        reason: Option<WithdrawReasons>,
        emit_event: bool,
    ) {
        T::Currency::extend_lock(
            identifier,
            sender,
            amount,
            reason.unwrap_or(WithdrawReasons::all()),
        );
        if emit_event {
            Self::deposit_event(Event::ExtendedLock(sender.clone(), identifier, amount));
        }
    }

    fn unlock_all(sender: &Self::AccountId, identifier: LockIdentifier, emit_event: bool) {
        T::Currency::remove_lock(identifier, sender);
        if emit_event {
            Self::deposit_event(Event::Unlocked(sender.clone(), identifier));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{mock::*, CommonUtils, LockableOperations};
    use frame_support::{
        assert_noop, assert_ok,
        traits::{Currency, ExistenceRequirement::AllowDeath, LockIdentifier},
    };
    use pallet_balances::Error as BalanceError;

    const ID_1: LockIdentifier = *b"1       ";
    const ID_2: LockIdentifier = *b"2       ";

    #[test]
    fn generate_random_works() {
        new_test_ext().execute_with(|| {
            frame_system::Pallet::<Test>::set_block_number(100);

            // same (account + block_number + salt)
            let random1 = Utils::generate_random(1, &vec![1]);
            let random2 = Utils::generate_random(1, &vec![1]);
            assert_eq!(random1, random2);

            // different salt
            let random1 = Utils::generate_random(1, &vec![1]);
            let random2 = Utils::generate_random(1, &vec![2]);
            assert_ne!(random1, random2);

            // different account
            let random1 = Utils::generate_random(1, &vec![1]);
            let random2 = Utils::generate_random(2, &vec![1]);
            assert_ne!(random1, random2);

            // different block height
            frame_system::Pallet::<Test>::set_block_number(100);
            let random1 = Utils::generate_random(1, &vec![1]);
            frame_system::Pallet::<Test>::set_block_number(101);
            let random2 = <Utils as CommonUtils>::generate_random(1, &vec![1]);
            assert_ne!(random1, random2);
        })
    }

    #[test]
    fn lock_capital_works() {
        // lock without event
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            assert_eq!(Balances::free_balance(&1), 10);
            Utils::lock_capital(&1, ID_1, 5, None, false);

            assert_eq!(Balances::free_balance(&1), 10);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
        });

        // same identifier lock with multiple times
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            assert_eq!(Balances::free_balance(&1), 10);
            Utils::lock_capital(&1, ID_1, 5, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::lock_capital(&1, ID_1, 3, None, false);
            assert_ok!(<Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath));
        });

        // different identifiers lock twice
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            assert_eq!(Balances::free_balance(&1), 10);
            Utils::lock_capital(&1, ID_1, 5, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::lock_capital(&1, ID_2, 3, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
            // lock with different identifier will take the largest
            assert_ok!(<Balances as Currency<_>>::transfer(&1, &2, 5, AllowDeath));
        });

        // lock with event
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            assert_eq!(Balances::free_balance(&1), 10);
            Utils::lock_capital(&1, ID_1, 5, None, true);

            assert_eq!(Balances::free_balance(&1), 10);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
            // todo should work
            // System::assert_last_event(Event::pallet_utils(crate::Event::Locked(1, ID_1, 5)));
        });
    }

    #[test]
    fn unlock_all_works() {
        // lock without event
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            Utils::lock_capital(&1, ID_1, 5, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::unlock_all(&1, ID_1, false);
            assert_ok!(<Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath));
        });

        // unlock from not exist identifier will do nothing
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            Utils::lock_capital(&1, ID_1, 5, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::unlock_all(&1, ID_2, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
        });

        // lock with multiple identifiers
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            Utils::lock_capital(&1, ID_1, 5, None, false);
            Utils::lock_capital(&1, ID_2, 3, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::unlock_all(&1, ID_1, false);
            assert_ok!(<Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath));
            // there is still 3 lock by ID_2
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 3, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::unlock_all(&1, ID_2, false);
            assert_ok!(<Balances as Currency<_>>::transfer(&1, &2, 3, AllowDeath));
        });

        // lock with event
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            Utils::lock_capital(&1, ID_1, 5, None, true);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::unlock_all(&1, ID_1, true);
            assert_ok!(<Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath));
            // todo should work
            // System::assert_last_event(Event::pallet_utils(crate::Event::Unlocked(1, ID_1)));
        });
    }

    #[test]
    fn extend_lock_works() {
        // lock without event
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            Utils::lock_capital(&1, ID_1, 6, None, false);
            Utils::extend_lock(&1, ID_2, 4, None, false);

            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
        });

        // extend from void works
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            Utils::extend_lock(&1, ID_2, 5, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
        });

        // same identifier lock with multiple times
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            assert_eq!(Balances::free_balance(&1), 10);
            Utils::extend_lock(&1, ID_1, 5, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::extend_lock(&1, ID_1, 3, None, false); // locked 5 after extend
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
        });

        // different identifiers lock twice
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            assert_eq!(Balances::free_balance(&1), 10);
            Utils::extend_lock(&1, ID_1, 5, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );

            Utils::extend_lock(&1, ID_2, 3, None, false);
            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
            // lock with different identifier will take the largest
            assert_ok!(<Balances as Currency<_>>::transfer(&1, &2, 5, AllowDeath));
        });

        // lock with event
        new_test_ext().execute_with(|| {
            let _ = Balances::deposit_creating(&1, 10);

            Utils::lock_capital(&1, ID_1, 6, None, true);
            Utils::extend_lock(&1, ID_2, 4, None, true);

            assert_noop!(
                <Balances as Currency<_>>::transfer(&1, &2, 6, AllowDeath),
                BalanceError::<Test>::LiquidityRestrictions
            );
            // todo should work
            // System::assert_last_event(Event::pallet_utils(crate::Event::ExtendedLock(1, ID_2, 4)));
        })
    }
}
