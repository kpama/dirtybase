use base64ct::Encoding;

pub fn encode(input: &[u8]) -> String {
    base64ct::Base64::encode_string(input)
}

pub fn decode(input: &str) -> Result<Vec<u8>, base64ct::Error> {
    base64ct::Base64::decode_vec(input)
}
