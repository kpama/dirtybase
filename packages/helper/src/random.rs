use rand::{distr::SampleString, rng};

pub fn random_bytes(length: usize) -> Vec<u8> {
    (0..length).map(|_| rand::random::<u8>()).collect()
}

pub fn random_bytes_hex(length: usize) -> String {
    hex::encode(random_bytes(length))
}

pub fn random_string(length: usize) -> String {
    rand::distr::Alphanumeric.sample_string(&mut rng(), length)
}
