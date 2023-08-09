use std::fs::File;
use std::io::{stdin, Read};
use std::process::abort;
use std::thread::{sleep, spawn};
use std::time::Duration;

use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{SmtpTransport, Transport};
use send_email::cli::build_cli;
use send_email::{Account, Config, Message};

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    File::open(config_file)
        .unwrap()
        .read_to_string(&mut config)
        .unwrap();
    let config: toml::Value = toml::from_str(&config).unwrap();

    let email_cfg = &config["email"];
    let config = Config {
        smtp: email_cfg["smtp"].as_str().unwrap().into(),
        from: email_cfg["from"].as_str().unwrap().into(),
        username: email_cfg["username"].as_str().unwrap().into(),
        password: email_cfg["password"].as_str().unwrap().into(),
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

    let send_thread = spawn(move || send_email(message, account));
    if let Some(&timeout) = timeout {
        spawn(move || {
            sleep(Duration::from_millis(timeout as u64));
            eprintln!("Time out!");
            // force to abort
            abort();
        });
    }

    let result = send_thread.join().unwrap();
    match result {
        Ok(_) => {
            println!("Done");
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}

fn send_email(
    message: Message,
    account: Account,
) -> Result<lettre::transport::smtp::response::Response, lettre::transport::smtp::Error> {
    let mut email = lettre::Message::builder()
        .from(message.from.parse().unwrap())
        .to(message.to.parse().unwrap());

    if let Some(ref s) = message.subject {
        email = email.subject(s);
    }
    let email = email
        .header(ContentType::TEXT_PLAIN)
        .body(message.body)
        .unwrap();

    let creds = account.credentials.clone();

    let mailer = SmtpTransport::relay(&account.smtp)
        .unwrap()
        .credentials(creds)
        .build();

    mailer.send(&email)
}
