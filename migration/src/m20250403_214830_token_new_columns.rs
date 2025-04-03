use async_trait::async_trait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Token::Table)
                    .add_column(ColumnDef::new(Token::Volume24Usd).decimal().not_null().default(0))
                    .add_column(ColumnDef::new(Token::Liquidity).decimal().not_null().default(0))
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Token::Table)
                    .drop_column(Token::Volume24Usd)
                    .drop_column(Token::Liquidity)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Token {
    Table,
    Volume24Usd,
    Liquidity,
}