pub struct LoginStep1Request {
    pub login: String,
    pub password: String,
}

pub struct LoginStep2Request {
    pub code: String,
}
