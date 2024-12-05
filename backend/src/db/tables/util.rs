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
      relations
        .iter()
        .find(|&r| **mapped_value == relation_to_id(r))
        .is_none()
    })
    .map(|mapped_value| uuids_to_active_model(*mapped_value, id))
    .collect();

  let to_remove: Vec<Uuid> = relations
    .into_iter()
    .filter_map(|r| {
      mapped_values
        .iter()
        .find(|mapped_value| **mapped_value == relation_to_id(&r))?;

      Some(relation_to_id(&r))
    })
    .collect();

  RT::insert_many(to_add).exec(db).await?;
  RT::delete_many()
    .filter(column.eq(id))
    .filter(map_column.is_in(to_remove))
    .exec(db)
    .await?;

  Ok(())
}
