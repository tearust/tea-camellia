use super::*;

pub trait CommonUtils {
    type AccountId;

    fn generate_random(sender: Self::AccountId, salt: &RandomSalt) -> U256;
}

pub trait LockableOperations {
    type AccountId;
    type BalanceOf;

    /// Locks the specified amount of tokens from the sender.
    /// identifier is generally holed by a pallet, reason can be none.
    fn lock_capital(
        sender: &Self::AccountId,
        identifier: LockIdentifier,
        amount: Self::BalanceOf,
        reason: Option<WithdrawReasons>,
        emit_event: bool,
    );

    /// Extends the lock period.
    /// identifier is generally holed by a pallet, reason can be none.
    fn extend_lock(
        sender: &Self::AccountId,
        identifier: LockIdentifier,
        amount: Self::BalanceOf,
        reason: Option<WithdrawReasons>,
        emit_event: bool,
    );

    /// Releases all locked tokens within specified identifier.
    fn unlock_all(sender: &Self::AccountId, identifier: LockIdentifier, emit_event: bool);
}
