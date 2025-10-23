use std::{collections::HashMap, sync::Arc};

use centaurus::{impl_from_error, FromReqExtension};
use chrono::{DateTime, Duration, Utc};
use http::StatusCode;
use lettre::{
  address::AddressError,
  error::Error,
  message::{header::ContentType, Mailbox},
  transport::smtp::{self, authentication::Credentials},
  Message, SmtpTransport, Transport,
};
use rand::{distr::Uniform, Rng};
use thiserror::Error;
use tokio::sync::Mutex;
use tracing::instrument;
use uuid::Uuid;

use crate::config::Config;

#[derive(Clone, FromReqExtension)]
pub struct Mailer {
  transport: SmtpTransport,
  sender: Mailbox,
  pub site_link: String,
}

#[derive(Clone, FromReqExtension)]
pub struct EmailState {
  pub change_req: Arc<Mutex<HashMap<Uuid, ChangeInfo>>>,
  pub exp: i64,
}

#[derive(Clone)]
pub struct ChangeInfo {
  exp: DateTime<Utc>,
  pub old_code: String,
  pub new_code: String,
  pub new_email: String,
}

impl_from_error!(MailError, StatusCode::INTERNAL_SERVER_ERROR);

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

impl EmailState {
  pub fn init(config: &Config) -> Self {
    Self {
      change_req: Default::default(),
      exp: config.auth_jwt_expiration_short,
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
  rand::rng()
    .sample_iter(Uniform::new(48, 58).unwrap())
    .take(6)
    .map(char::from)
    .collect()
}

impl Mailer {
  pub fn init(config: &Config) -> Self {
    let credentials = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());
    let transport = SmtpTransport::relay(&config.smtp_domain)
      .expect("Failed to initialize Smtp Transport")
      .credentials(credentials)
      .build();

    let sender = Mailbox::new(
      Some(config.smtp_sender_name.clone()),
      config
        .smtp_sender_email
        .parse()
        .expect("Invalid sender email address"),
    );

    Self {
      transport,
      sender,
      site_link: config.smtp_site_link.clone(),
    }
  }
}

impl Mailer {
  #[instrument(skip(self, body))]
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
