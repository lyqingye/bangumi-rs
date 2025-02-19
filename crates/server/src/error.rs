use crate::model::Resp;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("bangumi not found")]
    BangumiNotFound,

    #[error("actix error: {0}")]
    ActixError(#[from] actix_web::Error),

    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),

    #[error("db error: {0}")]
    DbError(#[from] sea_orm::DbErr),

    #[error("network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        let resp: Resp<()> = Resp::err_msg(self.to_string());
        HttpResponse::Ok().json(resp)
    }
}
