use entity::{apod, prelude::*};
use sea_orm::{prelude::*, ActiveValue::Set};
use serde::Serialize;

use crate::db::user::user::BasicUserInfo;

#[derive(Serialize)]
pub struct ApodInfo {
  pub title: String,
  pub user: BasicUserInfo,
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
  ) -> Result<Option<(apod::Model, Option<BasicUserInfo>)>, DbErr> {
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
      res[0].1.first().map(|user| BasicUserInfo {
        name: user.name.clone(),
        uuid: user.id,
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
            user: BasicUserInfo {
              name: users[0].name.clone(),
              uuid: users[0].id,
            },
          }
        })
        .collect(),
    )
  }
}
