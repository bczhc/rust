use lettre::transport::smtp::authentication::Credentials;

pub mod cli;

pub struct Config {
    pub smtp: String,
    pub from: String,
    pub username: String,
    pub password: String,
}

pub struct Message {
    pub from: String,
    pub to: String,
    pub subject: Option<String>,
    pub body: String,
}

pub struct Account {
    pub smtp: String,
    pub credentials: Credentials,
}
