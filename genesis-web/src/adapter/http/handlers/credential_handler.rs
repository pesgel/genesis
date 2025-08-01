use crate::adapter::cmd::credential::CredentialSaveCmd;
use crate::adapter::query::credential::CredentialListQuery;
use crate::adapter::vo::credential::{CredentialListItemVO, CredentialVO};
use crate::adapter::{ResList, Response, ResponseSuccess};
use crate::config::AppState;
use crate::error::{AppError, AppJson};
use crate::repo::model::credential;
use crate::repo::sea;
use crate::repo::sea::{CredentialRepo, SeaRepo};
use axum::extract::{Path, State};
use axum::Json;

pub async fn save_credential(
    State(state): State<AppState>,
    AppJson(param): AppJson<CredentialSaveCmd>,
) -> Result<Response<String>, AppError> {
    let model = genesis_common::copy::<_, credential::Model>(&param)?;
    CredentialRepo::save_credential(&state.conn, model)
        .await
        .map(|id| Ok(Response::success(id)))?
}

pub async fn get_credential_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CredentialVO>, AppError> {
    CredentialRepo::get_credential_by_id(&state.conn, &id)
        .await
        .map(|d| {
            Ok(Json(
                genesis_common::copy(&d).unwrap_or(CredentialVO::default()),
            ))
        })?
}

pub async fn delete_credential_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<ResponseSuccess, AppError> {
    SeaRepo::delete_by_id::<credential::Entity>(&state.conn, &id)
        .await
        .map(|_| Ok(ResponseSuccess::default()))?
}

pub async fn list_asset_credential(
    State(state): State<AppState>,
    Json(query): Json<CredentialListQuery>,
) -> Result<ResList<CredentialListItemVO>, AppError> {
    let (sql, values) = sea::SqlBuilder::new(
        r#"
SELECT
    a.id AS asset_id,
	a.`name` AS asset_name,
	a.address AS address,
	aa.protocol AS protocol,
	aa.`port` AS `port`,
	aa.id AS protocol_id,
	aa.id,
	aa.status,
	aa.principal,
	aa.updated_by,
	aa.created_by,
	aa.created_at,
	aa.updated_at,
	aa.remark AS remark,
	aa.auth_type AS auth_type
FROM
	asset_account aa
	LEFT JOIN asset a ON aa.asset_id = a.id
WHERE
    a.deleted = 0 and aa.deleted = 0
    "#,
    )
    .try_filter_like("aa.principal", query.principal)
    .try_filter_like("aa.address", query.address)
    .try_filter_like("a.name", query.asset_name)
    .try_filter_eq("aa.protocol", query.protocol)
    .try_filter_eq("aa.auth_type", query.auth_type)
    .build();
    let res = SeaRepo::page_with_sql::<CredentialListItemVO>(
        &state.conn,
        query.page_query.init(),
        sql,
        values,
        query.page_query.order_clause(),
    )
    .await?;
    println!("res {res:?}");
    Ok(ResList::new(res.0, res.1))
}
