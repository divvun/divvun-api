use actix_web::{HttpResponse, HttpRequest};
use juniper::http::graphiql::graphiql_source;

use crate::server::State;

pub fn graphiql(req: &HttpRequest<State>) -> HttpResponse {
    let html = graphiql_source(&format!("{}/graphql", &req.connection_info().host()));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
