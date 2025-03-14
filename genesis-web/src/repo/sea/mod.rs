mod instruct;

use chrono::{DateTime, Local};
pub use instruct::*;
use sea_orm::sea_query::ConditionExpression;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Condition, DbConn, DbErr, EntityTrait, IdenStatic,
    IntoActiveModel, Order, PaginatorTrait, PrimaryKeyTrait, QueryOrder, Select, SelectModel,
    Value,
};
use sea_orm::{Iterable, QueryFilter};
use serde::{Deserialize, Serialize};
use std::option::Option;

mod execute;
mod node;
mod user;

pub use execute::*;
pub use node::*;
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
    use crate::config::{init_shared_app_state, AppConfig, AppState};
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
                println!("Record inserted: {:?}", inserted_record);
            }
            Err(e) => {
                println!("insert error:{:?}", e)
            }
        }

        match instruct::InstructRepo::get_instruct_by_id(&state.conn, "1234").await {
            Ok(m) => {
                println!("query: {:?}", m)
            }
            Err(e) => {
                println!("error: {:?}", e)
            }
        }
    }

    async fn init_state() -> AppState {
        let mut config = AppConfig {
            server: Default::default(),
            mysql_config: Default::default(),
            jwt_config: Default::default(),
            tracing: Default::default(),
        };
        config.mysql_config.host = "127.0.0.1:13306".to_string();
        config.mysql_config.database = "genesis".to_string();
        config.mysql_config.username = "genesis".to_string();
        config.mysql_config.password = "BmLC89g6".to_string();

        init_shared_app_state(&config).await.unwrap()
    }
}
