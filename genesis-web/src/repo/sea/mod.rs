mod instruct;
pub use instruct::*;

mod node;
mod user;

pub use node::*;
pub use user::*;
#[cfg(test)]
mod tests {
    use crate::config::{init_shared_app_state, AppConfig};
    use crate::repo::model::instruct::Model;
    use crate::repo::sea::instruct;
    use uuid::Uuid;

    #[tokio::test]
    #[ignore]
    async fn test_repo() {
        let mut config = AppConfig {
            server: Default::default(),
            mysql_config: Default::default(),
            jwt_config: Default::default(),
        };
        config.mysql_config.host = "127.0.0.1:13306".to_string();
        config.mysql_config.database = "genesis".to_string();
        config.mysql_config.username = "genesis".to_string();
        config.mysql_config.password = "BmLC89g6".to_string();

        let state = init_shared_app_state(&config).await.unwrap();

        let new_instruct = Model {
            id: Uuid::new_v4().to_string(),
            data: "data".to_string(),
            name: "name".to_string(),
            des: "123123".to_string(),
            ..Default::default()
        };
        let x = instruct::InstructRepo::insert_instruct_one(&state.conn, new_instruct.into()).await;
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
}
