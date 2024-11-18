use surrealdb::{
  engine::remote::ws::{Client, Ws, Wss},
  opt::auth::Namespace,
  Error, Surreal,
};
use tables::Tables;

pub mod tables;

pub struct DB {
  surrealdb: Surreal<Client>,
}

impl DB {
  pub async fn init_db_from_env() -> Result<Self, Error> {
    let secure = std::env::var("DB_SECURE")
      .expect("No DB_SECURE found")
      .parse()
      .expect("Failed to parse Port");
    let address = std::env::var("DB_ADDRESS").expect("No DB address found");
    let port = std::env::var("DB_PORT")
      .expect("No DB address found")
      .parse()
      .expect("Failed to parse Port");
    let username = std::env::var("DB_USERNAME").expect("No DB address found");
    let password = std::env::var("DB_PASSWORD").expect("No DB address found");
    let database = std::env::var("DB_DATABASE").expect("No DB address found");

    Self::init_db(secure, &address, port, &username, &password, &database).await
  }

  pub async fn init_db(
    secure: bool,
    address: &str,
    port: u16,
    username: &str,
    password: &str,
    database: &str,
  ) -> Result<Self, Error> {
    let address = format!("{}:{}", address, port);
    let db = if secure {
      Surreal::new::<Wss>(address).await?
    } else {
      Surreal::new::<Ws>(address).await?
    };

    db.signin(Namespace {
      namespace: "positron",
      username,
      password,
    })
    .await?;

    db.use_ns("positron").await?;

    db.query(format!("DEFINE DATABASE IF NOT EXISTS {}", database))
      .await?;

    db.use_db(database).await?;

    Tables::new(&db).create_tables().await?;

    Ok(DB { surrealdb: db })
  }

  pub fn tables(&self) -> Tables<'_> {
    Tables::new(&self.surrealdb)
  }
}