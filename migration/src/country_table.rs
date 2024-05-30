use sea_orm_migration::prelude::*;

use entity::{Country, CountryActiveModel};

use crate::sea_orm::{ActiveModelTrait, EntityName, Set, TransactionTrait};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Fields {
    Id,
    Name,
    RefCount,
    V,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Country.table_ref())
                    .comment("测试表")
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Fields::Id)
                            .big_integer()
                            .not_null()
                            .primary_key()
                            .auto_increment()
                            .comment("地区ID"),
                    )
                    .col(
                        ColumnDef::new(Fields::Name)
                            .string_len(200)
                            .not_null()
                            .comment("地区名称"),
                    )
                    .col(
                        ColumnDef::new(Fields::V)
                            .big_integer()
                            .default(0)
                            .comment("版本"),
                    )
                    .col(
                        ColumnDef::new(Fields::RefCount)
                            .big_integer()
                            .null()
                            .comment("引用"),
                    )
                    .col(
                        ColumnDef::new(Fields::CreatedAt)
                            .timestamp()
                            .not_null()
                            .comment("创建时间"),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .table(Country.table_ref())
                    .name("idx-country-created_at")
                    .col(Fields::CreatedAt)
                    .to_owned(),
            )
            .await?;
        let conn = manager.get_connection();
        let tx = conn.begin().await?;
        let now = chrono::Local::now().naive_local();
        CountryActiveModel {
            name: Set("中国".to_owned()),
            ref_count: Set(0),
            created_at: Set(now),
            ..Default::default()
        }
            .insert(conn)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .if_exists()
                    .table(Country.table_ref())
                    .name("idx-country-created_at")
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(Country.table_ref())
                    .to_owned(),
            )
            .await
    }
}
