use super::*;

pub trait CommonUtils {
    type AccountId;
    fn generate_random(sender: Self::AccountId, salt: &RandomSalt) -> U256;
}
