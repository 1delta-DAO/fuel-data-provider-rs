use async_trait::async_trait;
use sea_orm_migration::prelude::*;
use uuid::Uuid;

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
                    .col(ColumnDef::new(SyncStatus::FirstCalculationPoint).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(SyncStatus::CalculationDataReady).boolean().not_null().default(false))
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
                    .col(ColumnDef::new(Token::Price).decimal().not_null().default(0))
                    .col(ColumnDef::new(Token::Volume24).decimal().not_null().default(0))
                    .col(ColumnDef::new(Token::Decimals).unsigned().not_null())
                    .col(ColumnDef::new(Token::HighRisk).boolean().not_null().default(false))
                    .col(ColumnDef::new(Token::NoLiquidity).boolean().not_null().default(false))
                    .col(ColumnDef::new(Token::Quoting).boolean().not_null().default(false))
                    .col(ColumnDef::new(Token::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(Token::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .to_owned(),
            )
            .await?;

        //Seed data

        let token_id: Uuid = Uuid::new_v4();
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO \"token\" (id, address, symbol, name, decimals) VALUES ('{}', '{}','{}','{}','{}');",
                token_id,
                "f8f8b6283d7fa5b672b530cbb84fcccb4ff8dc40f8176ef4544ddb1f1952ad07",
                "ETH",
                "Ethereum",
                9,
            ))
            .await?;
        let token_id: Uuid = Uuid::new_v4();
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO \"token\" (id, address, symbol, name, decimals,quoting) VALUES ('{}', '{}','{}','{}','{}','{}');",
                token_id,
                "33a6d90877f12c7954cca6d65587c25e9214c7bed2231c188981c7114c1bdb78",
                "USDF",
                "USDF",
                9,
                true,
            ))
            .await?;

        let token_id: Uuid = Uuid::new_v4();
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO \"token\" (id, address, symbol, name, decimals,quoting) VALUES ('{}', '{}','{}','{}','{}','{}');",
                token_id,
                "286c479da40dc953bddc3bb4c453b608bba2e0ac483b077bd475174115395e6b",
                "USDC",
                "USD Coin",
                9,
                true,
            ))
            .await?;

        let token_id: Uuid = Uuid::new_v4();
        manager
            .get_connection()
            .execute_unprepared(&format!(
                "INSERT INTO \"token\" (id, address, symbol, name, decimals,quoting) VALUES ('{}', '{}','{}','{}','{}','{}');",
                token_id,
                "a0265fb5c32f6e8db3197af3c7eb05c48ae373605b8165b6f4a51c5b0ba4812e",
                "USDT",
                "Teher USD",
                6,
                true,
            ))
            .await?;

        // Create table `unknown token`
        manager
            .create_table(
                Table::create()
                    .table(UnknownToken::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(UnknownToken::Id).uuid().not_null().primary_key().default(Expr::cust("uuid_generate_v4()")))
                    .col(ColumnDef::new(UnknownToken::Address).string().not_null().unique_key())
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
                    .col(ColumnDef::new(PairSwaps::BlockTime).timestamp_with_time_zone())
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

        manager
            .create_table(
                Table::create()
                    .table(VolumeData::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(VolumeData::Timestamp).timestamp_with_time_zone().not_null().primary_key().default(Expr::cust("NOW()")))
                    .col(ColumnDef::new(VolumeData::TokenId).uuid().not_null())
                    .col(ColumnDef::new(VolumeData::Volume).decimal().not_null().default(0))
                    .foreign_key(
                        ForeignKey::create()
                            .from(VolumeData::Table, VolumeData::TokenId)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(PriceData::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(PriceData::Id).uuid().not_null().primary_key().default(Expr::cust("uuid_generate_v4()")))
                    .col(ColumnDef::new(PriceData::TokenId).uuid().not_null())
                    .col(ColumnDef::new(PriceData::Price).decimal().not_null().default(0))
                    .col(ColumnDef::new(PriceData::Timestamp).timestamp_with_time_zone().not_null().default(Expr::cust("NOW()")))
                    .foreign_key(
                        ForeignKey::create()
                            .from(PriceData::Table, PriceData::TokenId)
                            .to(Token::Table, Token::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())

    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(SyncStatus::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(PairSwaps::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(MiraPools::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(TokenPairs::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Token::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(UnknownToken::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(VolumeData::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(PriceData::Table).if_exists().to_owned()).await?;
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
    Price,
    Volume24,
    Decimals,
    CreatedAt,
    UpdatedAt,
    HighRisk,
    NoLiquidity,
    Quoting,
}

#[derive(Iden)]
pub enum UnknownToken {
    Table,
    Id,
    Address,
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
    BlockTime,
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
    FirstCalculationPoint,
    CalculationDataReady,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
pub enum VolumeData{
    Table,
    Timestamp,
    TokenId,
    Volume,
}

#[derive(Iden)]
pub enum PriceData{
    Table,
    Id,
    TokenId,
    Price,
    Timestamp,
}