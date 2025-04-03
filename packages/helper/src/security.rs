pub fn random_bytes(length: u32) -> Vec<u8> {
    (0..length).map(|_| rand::random::<u8>()).collect()
}

pub fn random_bytes_hex(length: u32) -> String {
    hex::encode(random_bytes(length))
}
