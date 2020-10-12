use std::io;

use actix_web::error::ResponseError;
use actix_web::HttpResponse;
use failure::Fail;
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Fail, Debug, Deserialize, Serialize, Clone)]
#[fail(display = "api error")]
pub struct ApiError {
    pub message: String,
}

impl From<io::Error> for ApiError {
    fn from(item: io::Error) -> Self {
        ApiError {
            message: item.to_string(),
        }
    }
}

impl From<std::string::FromUtf8Error> for ApiError {
    fn from(item: std::string::FromUtf8Error) -> Self {
        ApiError {
            message: item.to_string(),
        }
    }
}

impl ResponseError for ApiError {
    fn render_response(&self) -> HttpResponse {
        error!("{}", self.message);
        return HttpResponse::InternalServerError()
            .content_type("application/json")
            .json(json!({ "message": self.message }));
    }
}
