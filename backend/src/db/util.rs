use sea_orm::prelude::*;
use sea_orm::EntityTrait;

pub async fn update_relations<RT: EntityTrait>(
  db: &DatabaseConnection,
  mapped_values: Vec<Uuid>,
  id: Uuid,
  relation_to_id: impl Fn(&<RT as EntityTrait>::Model) -> Uuid,
  uuids_to_active_model: impl Fn(Uuid, Uuid) -> RT::ActiveModel,
  column: impl ColumnTrait,
  map_column: impl ColumnTrait,
) -> Result<(), DbErr> {
  let relations = RT::find().filter(column.eq(id)).all(db).await?;

  let to_add: Vec<RT::ActiveModel> = mapped_values
    .iter()
    .filter(|mapped_value| {
      !relations
        .iter()
        .any(|r| **mapped_value == relation_to_id(r))
    })
    .map(|mapped_value| uuids_to_active_model(*mapped_value, id))
    .collect();

  let to_remove: Vec<Uuid> = relations
    .into_iter()
    .filter(|r| {
      !mapped_values
        .iter()
        .any(|mapped_value| *mapped_value == relation_to_id(r))
    })
    .map(|r| relation_to_id(&r))
    .collect();

  if !to_add.is_empty() {
    RT::insert_many(to_add).exec(db).await?;
  }
  if !to_remove.is_empty() {
    RT::delete_many()
      .filter(column.eq(id))
      .filter(map_column.is_in(to_remove))
      .exec(db)
      .await?;
  }

  Ok(())
}
