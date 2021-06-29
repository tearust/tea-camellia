use super::*;

pub trait CommonUtils {
	type AccountId;

	fn generate_random(sender: Self::AccountId, salt: &RandomSalt) -> U256;
}

pub trait CurrencyOperations {
	type AccountId;
	/// The balance of an account.
	type Balance;

	/// The total amount of issuance in the system.
	fn total_issuance() -> Self::Balance;

	/// The minimum balance any single account may have. This is equivalent to the `Balances` module's
	/// `ExistentialDeposit`.
	fn minimum_balance() -> Self::Balance;

	/// The combined (reserved and free) balance of `who`.
	fn total_balance(who: &Self::AccountId) -> Self::Balance;

	/// The 'free' balance of a given account.
	///
	/// This is the only balance that matters in terms of most operations on tokens. It alone
	/// is used to determine the balance when in the contract execution environment. When this
	/// balance falls below the value of `ExistentialDeposit`, then the 'current account' is
	/// deleted: specifically `FreeBalance`.
	///
	/// `system::AccountNonce` is also deleted if `ReservedBalance` is also zero (it also gets
	/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.
	fn free_balance(who: &Self::AccountId) -> Self::Balance;

	/// Transfer some liquid free balance to another staker.
	///
	/// This is a very high-level function. It will ensure all appropriate fees are paid
	/// and no imbalance in the system remains.
	fn transfer(
		source: &Self::AccountId,
		dest: &Self::AccountId,
		value: Self::Balance,
		existence_requirement: ExistenceRequirement,
	) -> DispatchResult;

	/// The amount of the balance of a given account that is externally reserved; this can still get
	/// slashed, but gets slashed last of all.
	///
	/// This balance is a 'reserve' balance that other subsystems use in order to set aside tokens
	/// that are still 'owned' by the account holder, but which are suspendable.
	///
	/// When this balance falls below the value of `ExistentialDeposit`, then this 'reserve account'
	/// is deleted: specifically, `ReservedBalance`.
	///
	/// `system::AccountNonce` is also deleted if `FreeBalance` is also zero (it also gets
	/// collapsed to zero if it ever becomes less than `ExistentialDeposit`.
	fn reserved_balance(who: &Self::AccountId) -> Self::Balance;

	/// Moves `value` from balance to reserved balance.
	///
	/// If the free balance is lower than `value`, then no funds will be moved and an `Err` will
	/// be returned to notify of this.
	fn reserve(who: &Self::AccountId, amount: Self::Balance) -> DispatchResult;

	/// Moves up to `value` from reserved balance to free balance.
	///
	/// As much funds up to `value` will be moved as possible. If the reserve balance of `who`
	/// is less than `value`, an `Err` will be returned to notify of this.
	fn unreserve(who: &Self::AccountId, amount: Self::Balance) -> DispatchResult;

	/// Deducts up to `value` from reserved balance of `who`. This function cannot fail.
	///
	/// As much funds up to `value` will be deducted as possible. If the reserve balance of `who`
	/// is less than `value`, then a non-zero value will be returned.
	fn slash_reserved(who: &Self::AccountId, value: Self::Balance) -> Self::Balance;

	/// Moves up to `value` from reserved balance of account `slashed` to balance of account
	/// `beneficiary`. `beneficiary` must exist for this to succeed. If it does not, `Err` will be
	/// returned. Funds will be placed in either the `free` balance or the `reserved` balance,
	/// depending on the `status`.
	///
	/// As much funds up to `value` will be deducted as possible. If this is less than `value`,
	/// then `Ok(non_zero)` will be returned.
	///
	/// # NOTES
	///
	/// - moved reserved balance will become free balance in `beneficiary` account
	fn repatriate_reserved(
		slashed: &Self::AccountId,
		beneficiary: &Self::AccountId,
		value: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;

	/// Batch operation of `repatriate_reserved`, reserved balance should larger than the sum of
	/// `value_list`, otherwise `Err` will be reserved.
	fn repatriate_reserved_batch(
		slashed: &Self::AccountId,
		beneficiary_list: &Vec<Self::AccountId>,
		value_list: &Vec<Self::Balance>,
	) -> DispatchResult;

	/// Adds up to `value` to the free balance of `who`. If `who` doesn't exist, it is created.
	///
	/// Infallible.
	fn deposit_creating(who: &Self::AccountId, value: Self::Balance);
}
