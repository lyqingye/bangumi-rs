use crate::model::Resp;
use actix_web::{HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("bangumi not found")]
    BangumiNotFound,

    #[error("internal error: {0}")]
    Internal(#[from] actix_web::Error),

    #[error("internal error: {0}")]
    Internal2(#[from] anyhow::Error),

    #[error("internal error: {0}")]
    Internal3(#[from] sea_orm::DbErr),
}

impl ResponseError for ServerError {
    fn error_response(&self) -> HttpResponse {
        let resp: Resp<()> = Resp::err_msg(self.to_string());
        HttpResponse::Ok().json(resp)
    }
}
