use super::*;

impl<T: cml::Config> cml::Pallet<T> {
	pub fn next_id() -> CmlId {
		LastCmlId::<T>::mutate(|id| {
			if *id < u64::MAX {
				*id += 1;
			} else {
				*id = 1;
			}

			*id
		})
	}
}

pub fn init_from_genesis_seeds<T>(genesis_seeds: &GenesisSeeds, account: T::AccountId)
where
	T: Config,
{
	let a_cml_list = convert_genesis_seeds_to_cmls::<T::AccountId, T::BlockNumber>(
		&genesis_seeds.a_seeds,
		account.clone(),
	);
	let b_cml_list = convert_genesis_seeds_to_cmls::<T::AccountId, T::BlockNumber>(
		&genesis_seeds.b_seeds,
		account.clone(),
	);
	let c_cml_list = convert_genesis_seeds_to_cmls::<T::AccountId, T::BlockNumber>(
		&genesis_seeds.c_seeds,
		account,
	);

	a_cml_list
		.into_iter()
		.chain(b_cml_list.into_iter())
		.chain(c_cml_list.into_iter())
		.for_each(|cml| {
			UserCmlStore::<T>::insert(NPCAccount::<T>::get().unwrap(), cml.id(), ());
			CmlStore::<T>::insert(cml.id(), cml);
		});

	LastCmlId::<T>::mutate(|old_last| {
		*old_last = old_last.saturating_add(
			(genesis_seeds.a_seeds.len()
				+ genesis_seeds.b_seeds.len()
				+ genesis_seeds.c_seeds.len()) as CmlId,
		);
	});
}

pub fn convert_genesis_seeds_to_cmls<AccountId, BlockNumber>(
	seeds: &Vec<Seed>,
	account: AccountId,
) -> Vec<CML<AccountId, BlockNumber>>
where
	AccountId: PartialEq + Clone + MaxEncodedLen,
	BlockNumber: Default + AtLeast32BitUnsigned + Clone + MaxEncodedLen,
{
	let mut cml_list = Vec::new();

	for seed in seeds {
		let cml = CML::from_seed(seed.clone(), account.clone());
		cml_list.push(cml);
	}
	cml_list
}
