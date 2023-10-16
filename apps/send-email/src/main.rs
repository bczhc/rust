#![feature(try_blocks)]

use std::fs::File;
use std::io::{stdin, Read};
use std::process::abort;
use std::thread::{sleep, spawn};
use std::time::Duration;

use anyhow::anyhow;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::response::Response;
use lettre::{SmtpTransport, Transport};

use send_email::cli::build_cli;
use send_email::{Account, Config, Message};

fn main() -> anyhow::Result<()> {
    let matches = build_cli().get_matches();

    let config_file = matches.get_one::<String>("config").unwrap();
    let to = matches.get_one::<String>("to").unwrap();
    let subject = matches.get_one::<String>("subject");
    let timeout = matches.get_one::<u32>("timeout");

    let body = matches.get_one::<String>("message");
    let body = match body {
        None => {
            eprintln!("Read message from stdin...");
            let mut message = String::new();
            stdin().read_to_string(&mut message)?;
            message
        }
        Some(b) => b.clone(),
    };

    let mut config = String::new();
    File::open(config_file)?.read_to_string(&mut config)?;
    let config: toml::Value = toml::from_str(&config)?;

    let email_cfg = &config["email"];
    let config: Option<Config> = try {
        Config {
            smtp: email_cfg["smtp"].as_str()?.into(),
            from: email_cfg["from"].as_str()?.into(),
            username: email_cfg["username"].as_str()?.into(),
            password: email_cfg["password"].as_str()?.into(),
        }
    };
    let Some(config) = config else {
        return Err(anyhow!("Wrong TOML format"));
    };

    let credentials = Credentials::new(config.username.clone(), config.password.clone());
    let account = Account {
        credentials,
        smtp: config.smtp.clone(),
    };
    let message = Message {
        from: config.from,
        to: to.into(),
        subject: subject.cloned(),
        body,
    };

    if let Some(&timeout) = timeout {
        spawn(move || {
            sleep(Duration::from_millis(timeout as u64));
            eprintln!("Time out!");
            // force to abort
            abort();
        });
    }
    let _response = send_email(message, account)?;
    println!("Done");
    Ok(())
}

fn send_email(message: Message, account: Account) -> anyhow::Result<Response> {
    let mut email = lettre::Message::builder()
        .from(message.from.parse()?)
        .to(message.to.parse()?);

    if let Some(ref s) = message.subject {
        email = email.subject(s);
    }
    let email = email.header(ContentType::TEXT_PLAIN).body(message.body)?;

    let creds = account.credentials.clone();

    let mailer = SmtpTransport::relay(&account.smtp)?
        .credentials(creds)
        .build();

    Ok(mailer.send(&email)?)
}
