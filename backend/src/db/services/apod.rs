use centaurus::db::tables::group::SimpleUserInfo;
use entity::{apod, prelude::*};
use sea_orm::{ActiveValue::Set, prelude::*};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApodInfo {
  pub title: String,
  pub user: SimpleUserInfo,
  pub date: Date,
}

pub struct ApodTable<'db> {
  db: &'db DatabaseConnection,
}

impl<'db> ApodTable<'db> {
  pub fn new(db: &'db DatabaseConnection) -> Self {
    Self { db }
  }

  pub async fn create(&self, apod: apod::Model) -> Result<(), DbErr> {
    let apod: apod::ActiveModel = apod.into();
    apod.insert(self.db).await?;
    Ok(())
  }

  pub async fn get_for_date(
    &self,
    date: Date,
  ) -> Result<Option<(apod::Model, Option<SimpleUserInfo>)>, DbErr> {
    let res = Apod::find()
      .filter(apod::Column::Date.eq(date))
      .find_with_related(User)
      .all(self.db)
      .await?;

    if res.is_empty() {
      return Ok(None);
    }

    Ok(Some((
      res[0].0.clone(),
      res[0].1.first().map(|user| SimpleUserInfo {
        name: user.name.clone(),
        id: user.id,
      }),
    )))
  }

  pub async fn set_good(&self, date: Date, user: Uuid, good: bool) -> Result<(), DbErr> {
    let mut apod: apod::ActiveModel = self
      .get_for_date(date)
      .await?
      .ok_or(DbErr::RecordNotFound("Not Found".into()))?
      .0
      .into();

    apod.selector = if good { Set(Some(user)) } else { Set(None) };

    apod.update(self.db).await?;

    Ok(())
  }

  pub async fn list_all(&self) -> Result<Vec<apod::Model>, DbErr> {
    Apod::find().all(self.db).await
  }

  pub async fn list(&self) -> Result<Vec<ApodInfo>, DbErr> {
    let res = Apod::find()
      .filter(apod::Column::Selector.is_not_null())
      .find_with_related(User)
      .all(self.db)
      .await?;

    Ok(
      res
        .into_iter()
        .map(|(apod, users)| {
          assert!(users.len() == 1);

          ApodInfo {
            title: apod.title,
            date: apod.date,
            user: SimpleUserInfo {
              name: users[0].name.clone(),
              id: users[0].id,
            },
          }
        })
        .collect(),
    )
  }
}

#[cfg(test)]
mod test {
  use crate::db::{
    DBTrait,
    test::{insert_user, test_db},
  };
  use chrono::NaiveDate;
  use entity::apod;
  use sea_orm::DbErr;
  use uuid::Uuid;

  fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2024, 1, day).unwrap()
  }

  async fn create_apod(db: &centaurus::db::init::Connection, d: NaiveDate, selector: Option<Uuid>) {
    db.apod()
      .create(apod::Model {
        id: Uuid::new_v4(),
        title: format!("APOD {d}"),
        date: d,
        selector,
      })
      .await
      .unwrap();
  }

  #[tokio::test]
  async fn get_for_date_none_when_absent() {
    let db = test_db().await;
    assert!(db.apod().get_for_date(date(1)).await.unwrap().is_none());
  }

  #[tokio::test]
  async fn get_for_date_without_and_with_selector() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    create_apod(&db, date(1), None).await;
    let (apod, info) = db.apod().get_for_date(date(1)).await.unwrap().unwrap();
    assert_eq!(apod.date, date(1));
    assert!(info.is_none());

    create_apod(&db, date(2), Some(user)).await;
    let (_, info) = db.apod().get_for_date(date(2)).await.unwrap().unwrap();
    let info = info.unwrap();
    assert_eq!(info.id, user);
  }

  #[tokio::test]
  async fn set_good_toggles_selector() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    create_apod(&db, date(1), None).await;

    db.apod().set_good(date(1), user, true).await.unwrap();
    let (apod, _) = db.apod().get_for_date(date(1)).await.unwrap().unwrap();
    assert_eq!(apod.selector, Some(user));

    db.apod().set_good(date(1), user, false).await.unwrap();
    let (apod, _) = db.apod().get_for_date(date(1)).await.unwrap().unwrap();
    assert_eq!(apod.selector, None);
  }

  #[tokio::test]
  async fn set_good_errors_for_missing_date() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;
    assert!(matches!(
      db.apod().set_good(date(9), user, true).await,
      Err(DbErr::RecordNotFound(_))
    ));
  }

  #[tokio::test]
  async fn list_all_returns_everything() {
    let db = test_db().await;
    create_apod(&db, date(1), None).await;
    create_apod(&db, date(2), None).await;
    assert_eq!(db.apod().list_all().await.unwrap().len(), 2);
  }

  #[tokio::test]
  async fn list_only_includes_selected() {
    let db = test_db().await;
    let user = insert_user(&db, "u", "u@x.com").await;

    create_apod(&db, date(1), None).await; // not selected -> excluded
    create_apod(&db, date(2), Some(user)).await; // selected -> included

    let list = db.apod().list().await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].date, date(2));
    assert_eq!(list[0].user.id, user);
  }
}
