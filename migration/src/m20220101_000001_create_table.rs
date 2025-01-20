use async_trait::async_trait;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {

        manager.get_connection()
            .execute_unprepared("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\";")
            .await?;

        // Create table `sync_status`
        manager
            .create_table(
                Table::create()
                    .table(SyncStatus::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(SyncStatus::Id).uuid().not_null().primary_key().default(Expr::cust("uuid_generate_v4()")))
                    .col(ColumnDef::new(SyncStatus::BlockNumber).unsigned().not_null().default(0))
                    .col(ColumnDef::new(SyncStatus::BlockTime).timestamp_with_time_zone())
                    .col(ColumnDef::new(SyncStatus::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(SyncStatus::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .to_owned(),
            )
            .await?;

        // Create table `token`
        manager
            .create_table(
                Table::create()
                    .table(Token::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Token::Id).uuid().not_null().primary_key().default(Expr::cust("uuid_generate_v4()")))
                    .col(ColumnDef::new(Token::Address).string().not_null().unique_key())
                    .col(ColumnDef::new(Token::Symbol).string().not_null().unique_key())
                    .col(ColumnDef::new(Token::Name).string().not_null())
                    .col(ColumnDef::new(Token::Decimals).unsigned().not_null())
                    .col(ColumnDef::new(Token::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(Token::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(Token::HighRisk).boolean().not_null().default(false))
                    .col(ColumnDef::new(Token::NoLiquidity).boolean().not_null().default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(TokenPairs::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TokenPairs::Id).uuid().not_null().primary_key().default(Expr::cust("uuid_generate_v4()")))
                    .col(ColumnDef::new(TokenPairs::BaseTokenDetailsId).uuid().not_null())
                    .col(ColumnDef::new(TokenPairs::QuoteTokenDetailsId).uuid().not_null())
                    .col(ColumnDef::new(TokenPairs::BaseAddress).string().not_null())
                    .col(ColumnDef::new(TokenPairs::QuoteAddress).string().not_null())
                    .col(ColumnDef::new(TokenPairs::PairAddress).string().not_null())
                    .col(ColumnDef::new(TokenPairs::BaseSymbol).string().not_null())
                    .col(ColumnDef::new(TokenPairs::QuoteSymbol).string().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(TokenPairs::Table, TokenPairs::BaseTokenDetailsId)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(TokenPairs::Table, TokenPairs::QuoteTokenDetailsId)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(MiraPools::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(MiraPools::Id).uuid().not_null().primary_key().default(Expr::cust("uuid_generate_v4()")))
                    .col(ColumnDef::new(MiraPools::PairId).uuid().not_null())
                    .col(ColumnDef::new(MiraPools::Swaps).big_integer().not_null().default(0))
                    .col(ColumnDef::new(MiraPools::ReserveBase).decimal().not_null().default(0))
                    .col(ColumnDef::new(MiraPools::ReserveQuote).decimal().not_null().default(0))
                    .col(ColumnDef::new(MiraPools::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(MiraPools::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(MiraPools::Table, MiraPools::PairId)
                            .to(TokenPairs::Table, TokenPairs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PairSwaps::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PairSwaps::Id).uuid().not_null().primary_key().default(Expr::cust("uuid_generate_v4()")))
                    .col(ColumnDef::new(PairSwaps::BlockNumber).string().not_null())
                    .col(ColumnDef::new(PairSwaps::TxId).string().not_null())
                    .col(ColumnDef::new(PairSwaps::UtxoId).string().not_null())
                    .col(ColumnDef::new(PairSwaps::PairId).uuid().not_null())
                    .col(ColumnDef::new(PairSwaps::BaseAmount).decimal().not_null().default(0))
                    .col(ColumnDef::new(PairSwaps::QuoteAmount).decimal().not_null().default(0))
                    .col(ColumnDef::new(PairSwaps::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(PairSwaps::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(PairSwaps::Table, PairSwaps::PairId)
                            .to(TokenPairs::Table, TokenPairs::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SyncStatus::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(PairSwaps::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(MiraPools::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(TokenPairs::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Token::Table).to_owned()).await?;
        Ok(())
    }
}

// Define enums for table and column names
#[derive(Iden)]
pub enum Token {
    Table,
    Id,
    Address,
    Symbol,
    Name,
    Decimals,
    CreatedAt,
    UpdatedAt,
    HighRisk,
    NoLiquidity,
}

#[derive(Iden)]
pub enum TokenPairs {
    Table,
    Id,
    PairAddress,
    BaseTokenDetailsId,
    QuoteTokenDetailsId,
    BaseAddress,
    QuoteAddress,
    BaseSymbol,
    QuoteSymbol,
}

#[derive(Iden)]
pub enum PairSwaps {
    Table,
    Id,
    BlockNumber,
    TxId,
    UtxoId,
    PairId,
    BaseAmount,
    QuoteAmount,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum MiraPools {
    Table,
    Id,
    PairId,
    Swaps,
    ReserveBase,
    ReserveQuote,
    CreatedAt,
    UpdatedAt,
}


#[derive(Iden)]
pub enum SyncStatus{
    Table,
    Id,
    BlockNumber,
    BlockTime,
    CreatedAt,
    UpdatedAt,
}