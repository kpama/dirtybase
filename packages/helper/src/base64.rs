use base64ct::Encoding;

pub fn encode(input: &[u8]) -> String {
    base64ct::Base64::encode_string(input)
}

pub fn decode(input: &str) -> Result<Vec<u8>, base64ct::Error> {
    let mut result = base64ct::Base64::decode_vec(input);
    if result.is_ok() {
        return result;
    }

    result = url_decode(input);
    if result.is_ok() {
        return result;
    }

    result = url_decode_unpadded(input);

    return result;
}

pub fn url_encode(input: &[u8]) -> String {
    base64ct::Base64Url::encode_string(input)
}

pub fn url_decode(input: &str) -> Result<Vec<u8>, base64ct::Error> {
    base64ct::Base64Url::decode_vec(input)
}

pub fn url_encode_unpadded(input: &[u8]) -> String {
    base64ct::Base64UrlUnpadded::encode_string(input)
}

pub fn url_decode_unpadded(input: &str) -> Result<Vec<u8>, base64ct::Error> {
    base64ct::Base64UrlUnpadded::decode_vec(input)
}
