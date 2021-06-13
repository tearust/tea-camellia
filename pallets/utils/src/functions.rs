use super::*;

impl<T: utils::Config> utils::Pallet<T> {
    pub fn generate_random(sender: T::AccountId, salt: &RandomSalt) -> U256 {
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

#[cfg(test)]
mod tests {
    use crate::mock::*;

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
            let random2 = Utils::generate_random(1, &vec![1]);
            assert_ne!(random1, random2);
        })
    }
}
