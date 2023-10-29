use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();
        // Create Users table
        manager.create_table(
            sea_query::Table::create()
            .table(User::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(User::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
            )
            .col(ColumnDef::new(User::Email).string().not_null())
            .col(ColumnDef::new(User::Name).string().not_null())
            .col(ColumnDef::new(User::Picture).string().not_null())
            .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        // todo!();
        manager.drop_table(sea_query::Table::drop().table(User::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum User {
    Table,
    Id,
    Email,
    Name,
    Picture
}
