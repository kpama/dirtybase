use base64ct::Encoding;
use dirtybase_encrypt::Encrypter;

fn main() {
    let key = Encrypter::generate_aes256gcm_key();
    let data = "The quick brown fox jumps over the lazy dog";

    let encrypter = Encrypter::new(&key, None);

    let encrypted_data = encrypter.encrypt(data.into());

    if let Ok(e) = encrypted_data {
        println!("encrypted: {}", base64ct::Base64::encode_string(&e));
        if let Ok(e) = encrypter.decrypt(&e) {
            println!("decrypted: {}", String::from_utf8_lossy(&e));
        }
    }
}
