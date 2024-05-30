pub struct UserLoginFailed {
    timestamp: i64,
    id: String,
    attempts: u8,
}

impl UserLoginFailed {
    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn attempts(&self) -> u8 {
        self.attempts
    }
}
