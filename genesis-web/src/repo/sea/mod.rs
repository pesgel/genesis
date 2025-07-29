mod instruct;

use chrono::{DateTime, Local};
pub use instruct::*;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, ConnectionTrait, DbConn, DbErr,
    EntityTrait, FromQueryResult, IdenStatic, IntoActiveModel, Order, PaginatorTrait,
    PrimaryKeyTrait, QueryOrder, Select, SelectModel, Statement, Value,
};
use sea_orm::{Iterable, QueryFilter};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::option::Option;
use tracing::debug;

mod asset;
mod builder;
mod credential;
mod execute;
mod node;
mod protocol;
mod user;

pub use asset::*;
pub use builder::*;
pub use credential::*;
pub use execute::*;
pub use node::*;
pub use protocol::*;
pub use user::*;

pub(crate) struct SeaRepo;
impl SeaRepo {
    #[allow(dead_code)]
    pub async fn delete_by_id<E>(db: &DbConn, id: &str) -> anyhow::Result<E::Model>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        <E as EntityTrait>::ActiveModel: Send,
    {
        let mut model = E::ActiveModel::default();

        E::Column::iter().for_each(|e| {
            if e.as_str() == FIELD_UPDATED_AT {
                Self::set_now_time::<E>(&mut model, e)
            }
            if e.as_str() == FIELD_ID {
                model.set(e, Value::String(Some(Box::new(id.to_string()))))
            }
            if e.as_str() == FIELD_DELETED {
                model.set(e, Value::TinyInt(Some(1)))
            }
        });

        anyhow::Ok(model.update(db).await?)
    }

    #[allow(dead_code)]
    pub async fn remove_by_id<E, T>(db: &DbConn, id: T) -> anyhow::Result<u64>
    where
        E: EntityTrait,
        T: Into<<E::PrimaryKey as PrimaryKeyTrait>::ValueType>,
    {
        let res = E::delete_by_id(id).exec(db).await?;
        anyhow::Ok(res.rows_affected)
    }
    #[allow(dead_code)]
    pub async fn page_with_default<E>(
        db: &DbConn,
        pg: (u64, u64),
        ces: Option<Vec<ConditionExpression>>,
    ) -> anyhow::Result<(u64, Vec<E::Model>)>
    where
        E: EntityTrait,
        Select<E>: for<'a> PaginatorTrait<'a, DbConn, Selector = SelectModel<E::Model>>,
    {
        Self::page_with_order(db, pg, ces, None).await
    }

    pub async fn page_with_order<E>(
        db: &DbConn,
        pg: (u64, u64),
        ces: Option<Vec<ConditionExpression>>,
        order_by: Option<Vec<(E::Column, Order)>>, // 新增排序参数
    ) -> anyhow::Result<(u64, Vec<E::Model>)>
    where
        E: EntityTrait,
        Select<E>: for<'a> PaginatorTrait<'a, DbConn, Selector = SelectModel<E::Model>>,
    {
        let mut ens = E::find();

        // 处理过滤条件
        if let Some(ft) = ces {
            for exp in ft {
                match exp {
                    ConditionExpression::Condition(cond) => ens = ens.filter(cond),
                    ConditionExpression::SimpleExpr(sim) => ens = ens.filter(sim),
                }
            }
        }

        // 处理默认的 deleted 过滤
        for e in E::Column::iter() {
            if e.as_str() == FIELD_DELETED {
                ens = ens.filter(Condition::all().add(e.eq(0)));
            }
            if e.as_str() == FIELD_CREATED_AT && order_by.is_none() {
                ens = ens.order_by(e, Order::Desc);
            }
        }

        // 新增：处理排序条件
        if let Some(orders) = order_by {
            for (col, order) in orders {
                ens = ens.order_by(col, order);
            }
        }

        let ens = ens.paginate(db, pg.1);
        let count = ens.num_items().await?;
        let res = ens.fetch_page(std::cmp::max(pg.0 - 1, 0)).await?;
        Ok((count, res))
    }
    pub async fn page_with_sql<T>(
        db: &DbConn,
        pg: (u64, u64),
        sql: String,
        values: Vec<Value>,
        order_by: Option<String>,
    ) -> anyhow::Result<(u64, Vec<T>)>
    where
        T: FromQueryResult + Send + Unpin + Debug,
    {
        let backend = db.get_database_backend();

        // COUNT 查询
        let count_sql = format!(
            "SELECT COUNT(*) AS count FROM ({}) AS subquery",
            sql.clone()
        );
        let count_stmt = Statement::from_sql_and_values(backend, &count_sql, values.clone());
        let count_row = db.query_one(count_stmt).await?;
        let count = count_row
            .map(|r| r.try_get::<i64>("", "count").unwrap_or(0))
            .unwrap_or(0);

        if count == 0 {
            return Ok((0, vec![]));
        }
        // 拼接分页 SQL
        let full_sql = if let Some(order) = order_by {
            if sql.to_uppercase().contains("ORDER BY") {
                sql
            } else {
                format!("{sql} ORDER BY {order}")
            }
        } else {
            sql
        };
        let offset = (pg.0.saturating_sub(1)) * pg.1;
        let paginated_sql = format!("{} LIMIT {} OFFSET {}", full_sql, pg.1, offset);
        let query_stmt = Statement::from_sql_and_values(backend, &paginated_sql, values);
        let rows = db.query_all(query_stmt).await?;
        let items = rows
            .into_iter()
            .filter_map(|row| match T::from_query_result(&row, "") {
                Ok(v) => Some(v),
                Err(er) => {
                    debug!("page with sql err: {:?}", er);
                    None
                }
            })
            .collect();

        Ok((count as u64, items))
    }

    #[allow(dead_code)]
    pub async fn update_with_default<E>(
        db: &DbConn,
        mut model: E::ActiveModel,
    ) -> anyhow::Result<E::Model>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        <E as EntityTrait>::ActiveModel: Send,
    {
        E::Column::iter().for_each(|e| {
            if e.as_str() == FIELD_UPDATED_AT {
                Self::set_now_time::<E>(&mut model, e)
            }
        });
        anyhow::Ok(model.update(db).await?)
    }
    /// Inserts an ActiveModel instance into the database.
    #[allow(dead_code)]
    pub async fn insert_with_default<E, D>(db: &DbConn, data: D) -> anyhow::Result<String>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        D: Serialize,
        for<'de> <E as EntityTrait>::Model: Deserialize<'de>,
        <E as EntityTrait>::ActiveModel: Send,
    {
        let mut id = String::new();
        let mut model = E::ActiveModel::from_json(serde_json::to_value(data)?)?;
        E::Column::iter().for_each(|e| match e.as_str() {
            FIELD_ID => match model.get(e) {
                ActiveValue::Set(value) => {
                    if let Value::String(Some(now_id)) = value {
                        if now_id.is_empty() {
                            id = default_id();
                            model.set(e, Value::String(Some(Box::new(id.clone()))))
                        } else {
                            id = *now_id;
                        }
                    }
                }
                ActiveValue::Unchanged(value) => {
                    id = value.to_string();
                }
                ActiveValue::NotSet => {
                    id = default_id();
                    model.set(e, Value::String(Some(Box::new(id.clone()))))
                }
            },
            FIELD_CREATED_AT | FIELD_UPDATED_AT => Self::set_now_time::<E>(&mut model, e),
            _ => {}
        });
        match model.insert(db).await {
            Ok(_) => Ok(id),
            // Optional: handle specific case gracefully
            Err(DbErr::RecordNotInserted) => Ok(id),
            Err(e) => anyhow::bail!(e),
        }
    }

    pub async fn insert_many_with_default<E, D>(
        db: &DbConn,
        data_list: Vec<D>,
    ) -> anyhow::Result<Vec<String>>
    where
        E: EntityTrait,
        E::Model: IntoActiveModel<E::ActiveModel>,
        D: Serialize,
        for<'de> E::Model: Deserialize<'de>,
        E::ActiveModel: Send,
    {
        let mut id_list = Vec::with_capacity(data_list.len());
        let mut active_models = Vec::with_capacity(data_list.len());
        for data in data_list {
            let mut id = String::new();
            let mut model = E::ActiveModel::from_json(serde_json::to_value(data)?)?;
            // 遍历所有字段，处理 ID 和时间
            E::Column::iter().for_each(|e| match e.as_str() {
                FIELD_ID => match model.get(e) {
                    ActiveValue::Set(value) => {
                        if let Value::String(Some(now_id)) = value {
                            if now_id.is_empty() {
                                id = default_id();
                                model.set(e, Value::String(Some(Box::new(id.clone()))));
                            } else {
                                id = *now_id;
                            }
                        }
                    }
                    ActiveValue::Unchanged(value) => {
                        id = value.to_string();
                    }
                    ActiveValue::NotSet => {
                        id = default_id();
                        model.set(e, Value::String(Some(Box::new(id.clone()))));
                    }
                },
                FIELD_CREATED_AT | FIELD_UPDATED_AT => {
                    Self::set_now_time::<E>(&mut model, e);
                }
                _ => {}
            });

            id_list.push(id);
            active_models.push(model);
        }
        // 执行批量插入
        E::insert_many(active_models)
            .exec_without_returning(db)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(id_list)
    }

    /// Convert to sea_orm model
    #[allow(dead_code)]
    pub fn convert_to_model<E, D>(data: D) -> anyhow::Result<E::Model>
    where
        E: EntityTrait,
        D: Serialize,
        for<'de> <E as EntityTrait>::Model: Deserialize<'de>,
    {
        let vl = serde_json::to_value(data)?;
        let data: E::Model = serde_json::from_value(vl)?;
        anyhow::Ok(data)
    }

    fn set_now_time<A>(model: &mut <A as EntityTrait>::ActiveModel, e: <A as EntityTrait>::Column)
    where
        A: EntityTrait,
    {
        match model.get(e) {
            ActiveValue::Set(v) => {
                if let Some(vv) = v.as_ref_chrono_date_time_local() {
                    if vv.eq(&chrono::DateTime::<Local>::default()) {
                        model.set(
                            e,
                            Value::ChronoDateTimeLocal(Some(Box::new(default_time()))),
                        )
                    }
                }
            }
            ActiveValue::Unchanged(_) => {}
            ActiveValue::NotSet => model.set(
                e,
                Value::ChronoDateTimeLocal(Some(Box::new(default_time()))),
            ),
        }
    }
    pub async fn update_with_map<E>(
        db: &DbConn,
        updates: serde_json::Value,
        filters: serde_json::Value,
    ) -> anyhow::Result<u64>
    where
        E: EntityTrait,
        E::Model: Default + IntoActiveModel<E::ActiveModel>,
        for<'de> <E as EntityTrait>::Model: Deserialize<'de>,
    {
        // 解析为 ActiveModel
        let mut active_model = E::ActiveModel::from_json(updates)?;
        // 手动设置 updated_at 字段
        for col in E::Column::iter() {
            if col.as_str() == FIELD_UPDATED_AT {
                Self::set_now_time::<E>(&mut active_model, col);
            }
        }
        // 构造过滤条件
        let mut cond = ::sea_orm::Condition::all();

        if let ::serde_json::Value::Object(filter_map) = filters {
            for (k, v) in filter_map.into_iter() {
                if let Some(col) = <E as EntityTrait>::Column::iter().find(|c| c.as_str() == k) {
                    let expr = match v {
                        serde_json::Value::String(s) => col.eq(s),
                        serde_json::Value::Number(n) => {
                            if let Some(i) = n.as_i64() {
                                col.eq(i)
                            } else if let Some(f) = n.as_f64() {
                                col.eq(f)
                            } else {
                                continue;
                            }
                        }
                        serde_json::Value::Bool(b) => col.eq(b),
                        serde_json::Value::Null => col.is_null(),
                        serde_json::Value::Array(arr) => {
                            if arr.iter().all(|v| v.is_string()) {
                                let in_str: Vec<String> = arr
                                    .into_iter()
                                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                    .collect();
                                col.is_in(in_str)
                            } else if arr.iter().all(|v| v.is_i64()) {
                                let nums: Vec<i64> =
                                    arr.into_iter().filter_map(|v| v.as_i64()).collect();
                                col.is_in(nums)
                            } else {
                                continue;
                            }
                        }
                        _ => continue, // 复杂结构暂不支持
                    };
                    cond = cond.add(expr);
                }
            }
        }

        let res = E::update_many()
            .set(active_model)
            .filter(cond)
            .exec(db)
            .await?;

        Ok(res.rows_affected)
    }
    #[allow(dead_code)]
    pub async fn update_with_map_by_id<E>(
        db: &DbConn,
        updates: HashMap<String, Value>,
    ) -> anyhow::Result<E::Model>
    where
        E: EntityTrait,
        E::Model: Default + IntoActiveModel<E::ActiveModel>,
        <E as EntityTrait>::ActiveModel: Send,
    {
        let mut is_update = false;
        // step1. 生成默认的 Model 和 ActiveModel
        let model: E::Model = Default::default();
        // 通过 Default 创建默认模型
        let mut active_model = model.into_active_model();
        // step2. 遍历 HashMap，并动态设置字段值
        E::Column::iter().for_each(|e| {
            if e.as_str() == FIELD_UPDATED_AT {
                Self::set_now_time::<E>(&mut active_model, e);
            } else {
                if let Some(v) = updates.get(e.as_str()) {
                    active_model.set(e, v.clone())
                }
                is_update = true;
            }
        });
        // step3. 执行更新操作
        if is_update {
            Ok(active_model.update(db).await?)
        } else {
            anyhow::bail!("no field update")
        }
    }
}

const FIELD_ID: &str = "id";
const FIELD_CREATED_AT: &str = "created_at";
const FIELD_UPDATED_AT: &str = "updated_at";
const FIELD_DELETED: &str = "deleted";

fn default_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn default_time() -> DateTime<Local> {
    Local::now()
}

#[cfg(test)]
mod tests {
    use crate::config::Db::Mysql;
    use crate::config::{init_shared_app_state, AppConfig, AppState, MysqlConfig};
    use crate::repo::model::user;
    use crate::repo::sea::{instruct, SeaRepo};
    use sea_orm::sea_query::{ConditionExpression, Expr};
    use sea_orm::ActiveValue::{Set, Unchanged};
    use sea_orm::{Condition, IntoActiveModel};
    use serde_json::json;
    use std::time::Duration;
    use tracing::info;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore]
    async fn test_for_page() {
        tracing_subscriber::fmt().init();
        let state = init_state().await;
        let cond = Condition::all().add(Expr::col(user::Column::Name).like("yang%"));
        let res = SeaRepo::page_with_default::<user::Entity>(
            &state.conn,
            (0, 2),
            Some(vec![ConditionExpression::Condition(cond)]),
        )
        .await
        .unwrap();
        info!("res:{:?}", res)
    }
    #[tokio::test]
    #[ignore]
    async fn test_from_json() {
        tracing_subscriber::fmt().init();
        let state = init_state().await;
        // TEST JSON
        let value = json!({
            "name": "from_json1",
            "username": "from_json1",
        });
        // use crate::adapter::cmd::user::UserRegisterCmd;
        // let mut value = UserRegisterCmd::default();
        // value.name = "registerName".to_string();
        // value.phone = "registerPhone".to_string();
        let x = SeaRepo::insert_with_default::<user::Entity, _>(&state.conn, value)
            .await
            .unwrap();
        info!("x:{:?}", x);
        tokio::time::sleep(Duration::from_secs(5)).await;
        let am = user::Model::default();
        let mut am = am.into_active_model();
        am.id = Unchanged(x.clone());
        am.name = Set("newName".to_string());
        let res = SeaRepo::update_with_default::<user::Entity>(&state.conn, am)
            .await
            .unwrap();
        info!("update res:{:?}", res)
    }
    #[tokio::test]
    #[ignore]
    async fn test_repo() {
        let state = init_state().await;

        let new_instruct = crate::repo::model::instruct::Model {
            id: Uuid::new_v4().to_string(),
            data: "data".to_string(),
            name: "name".to_string(),
            des: "123123".to_string(),
            ..Default::default()
        };
        let x = instruct::InstructRepo::insert_instruct_one(&state.conn, new_instruct).await;
        match x {
            Ok(inserted_record) => {
                println!("Record inserted: {inserted_record:?}");
            }
            Err(e) => {
                println!("insert error:{e:?}")
            }
        }

        match instruct::InstructRepo::get_instruct_by_id(&state.conn, "1234").await {
            Ok(m) => {
                println!("query: {m:?}")
            }
            Err(e) => {
                println!("error: {e:?}")
            }
        }
    }

    async fn init_state() -> AppState {
        let mut config = AppConfig {
            server: Default::default(),
            db_config: Default::default(),
            jwt_config: Default::default(),
            tracing: Default::default(),
        };
        let mysql_config = MysqlConfig {
            host: "127.0.0.1:13306".to_string(),
            username: "genesis".to_string(),
            password: "genesis".to_string(),
            database: "BmLC89g6".to_string(),
        };
        config.db_config = Mysql(mysql_config);
        init_shared_app_state(&config).await.unwrap()
    }
}
