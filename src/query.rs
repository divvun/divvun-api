use actix_web::{HttpResponse, HttpRequest};
use juniper::http::graphiql::graphiql_source;

pub fn graphiql(req: HttpRequest) -> HttpResponse {
    let html = graphiql_source(&format!("{}/graphql", &req.connection_info().host()));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}
