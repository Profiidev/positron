use anyhow::Error;
use surrealdb::{
  engine::remote::ws::{Client, Wss},
  opt::auth::Namespace,
  Surreal,
};
use tables::Tables;

pub mod tables;

pub struct DB {
  surrealdb: Surreal<Client>,
}

impl DB {
  pub async fn init_db_from_env() -> Result<Self, Error> {
    let address = std::env::var("DB_ADDRESS").expect("No DB address found");
    let port = std::env::var("DB_PORT")
      .expect("No DB address found")
      .parse()
      .expect("Failed to parse Port");
    let username = std::env::var("DB_USERNAME").expect("No DB address found");
    let password = std::env::var("DB_PASSWORD").expect("No DB address found");

    Self::init_db(&address, port, &username, &password).await
  }

  pub async fn init_db(
    address: &str,
    port: u16,
    username: &str,
    password: &str,
  ) -> Result<Self, Error> {
    let db = Surreal::new::<Wss>(format!("{}:{}", address, port)).await?;

    db.signin(Namespace {
      namespace: "positron",
      username,
      password,
    })
    .await?;

    db.use_ns("positron").use_db("positron").await?;

    Tables::new(&db).create_tables().await?;

    Ok(DB { surrealdb: db })
  }

  pub fn tables(&self) -> Tables<'_> {
    Tables::new(&self.surrealdb)
  }
}
