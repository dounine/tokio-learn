use std::time::Duration;
use anyhow::{anyhow, bail};
use tracing::{info, info_span, instrument, Instrument, warn};
use entity::{Country, CountryActiveModel};
use migration::sea_orm::{ActiveModelTrait, DatabaseBackend, DatabaseTransaction, EntityTrait, Set, Statement};
use migration::{ConnectionTrait, Value};

#[instrument(skip(tx))]
pub async fn incrment(tx: &DatabaseTransaction, id: i64) -> Result<(), anyhow::Error> {
    let info = Country::find_by_id(id).one(tx).await?;
    let info = info.ok_or(anyhow!("data not found"))?;
    let rows = tx.execute(Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        r#"update "test_table" set "ref_count" = "test_table"."ref_count" + 1 , "v" = "test_table"."v" + 1 where "id" = $1 and "test_table"."v" = $2"#,
        vec![Value::from(id), Value::from(info.v)],
    )).await?;
    if rows.rows_affected() != 1 {
        bail!("修改失败，影响行数：{}",rows.rows_affected())
    }
    let span = info_span!("async_test",id=id);
    async_test().instrument(span.clone()).await?;
    instrument_test().await?;

    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;

        let s = info_span!("hello",parent=&span.clone(),aaaaaaa=tracing::field::Empty);

        // let child = span.clone();
        let _g = s.enter();
        warn!("警告");
        s.record("aaaaaaa", "你好");
        info!("ok");
    });

    Ok(())
}

async fn async_test() -> Result<(), anyhow::Error> {
    info!("ok");
    return Ok(());
}

#[instrument]
async fn instrument_test() -> Result<(), anyhow::Error> {
    info!("ok");
    return Ok(());
}

#[instrument(skip(tx))]
pub async fn decrment(tx: &DatabaseTransaction, id: i64) -> Result<(), anyhow::Error> {
    let info = Country::find_by_id(id).one(tx).await?;
    let info = info.ok_or(anyhow!("data not found"))?;
    let rows =
        tx.execute(Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            r#"update "test_table" set "ref_count" = "test_table"."ref_count" - 1 , "v" = "test_table"."v" + 1 where "id" = $1 and "test_table"."v" = $2"#,
            vec![Value::from(id), Value::from(info.v)],
        )).await?;
    if rows.rows_affected() != 1 {
        bail!("修改失败，影响行数：{}",rows.rows_affected())
    }
    Ok(())
}

#[instrument(skip(tx))]
pub async fn update_test(tx: &DatabaseTransaction) -> Result<(), anyhow::Error> {
    //如果修改找不到数据会异常
    let now = chrono::Local::now().naive_local();
    let mut model = CountryActiveModel {
        id: Set(1),
        name: Set("hello".to_string()),
        ref_count: Set(0),
        created_at: Set(now),
        ..Default::default()
    };
    let mut model: CountryActiveModel = Country::find_by_id(1).one(tx).await?.unwrap().into();
    model.name = Set("hi".to_string());//只更新name字段，其他字段不更新
    // model.ref_count = Set(10);
    let res = model.update(tx).await?;
    Ok(())
}
