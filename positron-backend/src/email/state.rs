use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use lettre::{
  address::AddressError,
  error::Error,
  message::{header::ContentType, Mailbox},
  transport::smtp::{self, authentication::Credentials},
  Message, SmtpTransport, Transport,
};
use rand::{distributions::Uniform, Rng};
use rocket::tokio::sync::Mutex;
use thiserror::Error;
use uuid::Uuid;

pub struct Mailer {
  transport: SmtpTransport,
  sender: Mailbox,
  pub site_link: String,
}

pub struct EmailState {
  pub change_req: Mutex<HashMap<Uuid, ChangeInfo>>,
  pub exp: i64,
}

#[derive(Clone)]
pub struct ChangeInfo {
  exp: DateTime<Utc>,
  pub old_code: String,
  pub new_code: String,
  pub new_email: String,
}

#[derive(Error, Debug)]
pub enum MailError {
  #[error("AddressParseError {source:?}")]
  AddressParse {
    #[from]
    source: AddressError,
  },
  #[error("MailCreationError {source:?}")]
  MailCreation {
    #[from]
    source: Error,
  },
  #[error("MailSendError {source:?}")]
  MailSend {
    #[from]
    source: smtp::Error,
  },
}

impl ChangeInfo {
  pub fn check_old(&self, code: String) -> bool {
    self.exp >= Utc::now() && code == *self.old_code
  }

  pub fn check_new(&self, code: String) -> bool {
    self.exp >= Utc::now() && code == *self.new_code
  }
}

impl Default for EmailState {
  fn default() -> Self {
    let exp = std::env::var("AUTH_JWT_EXPIRATION_SHORT")
      .expect("Failed to load JwtExpiration")
      .parse()
      .expect("Failed to parse JwtExpiration");

    Self {
      change_req: Default::default(),
      exp,
    }
  }
}

impl EmailState {
  pub fn gen_info(&self, new_email: String) -> Option<ChangeInfo> {
    let old_code = gen_code();
    let new_code = gen_code();

    let exp = Utc::now().checked_add_signed(Duration::seconds(self.exp))?;

    Some(ChangeInfo {
      old_code,
      new_code,
      exp,
      new_email,
    })
  }
}

fn gen_code() -> String {
  rand::thread_rng()
    .sample_iter(Uniform::new(48, 58))
    .take(6)
    .map(char::from)
    .collect()
}

impl Default for Mailer {
  fn default() -> Self {
    let username = std::env::var("SMTP_USERNAME").expect("Failed to load SMTP_USERNAME");
    let password = std::env::var("SMTP_PASSWORD").expect("Failed to load SMTP_PASSWORD");
    let domain = std::env::var("SMTP_DOMAIN").expect("Failed to load SMTP_DOMAIN");

    let sender_name = std::env::var("SMTP_SENDER_NAME").expect("Failed to load SMTP_SENDER_NAME");
    let sender_email = std::env::var("SMTP_SENDER_EMAIL")
      .expect("Failed to load SMTP_SENDER_EMAIL")
      .parse()
      .expect("Failed to parse SMTP_SENDER_EMAIL");

    let site_link = std::env::var("SMTP_SITE_LINK").expect("Failed to load SMTP_SITE_LINK");

    let credentials = Credentials::new(username, password);
    let transport = SmtpTransport::relay(&domain)
      .expect("Failed to initialize Smtp Transport")
      .credentials(credentials)
      .build();

    let sender = Mailbox::new(Some(sender_name), sender_email);

    Self {
      transport,
      sender,
      site_link,
    }
  }
}

impl Mailer {
  pub fn send_mail(
    &self,
    username: String,
    email: String,
    subject: String,
    body: String,
  ) -> Result<(), MailError> {
    let receiver = Mailbox::new(Some(username), email.parse()?);

    let mail = Message::builder()
      .from(self.sender.clone())
      .to(receiver)
      .subject(subject)
      .header(ContentType::TEXT_HTML)
      .body(body)?;

    self.transport.send(&mail)?;

    Ok(())
  }
}
