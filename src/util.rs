use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

pub fn random_str(len: usize) -> String {
    let rng = thread_rng();
    rng.sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}
