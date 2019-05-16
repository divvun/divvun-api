use actix_web::{HttpResponse, HttpRequest, web};
use juniper::http::graphiql::graphiql_source;
use juniper::http::GraphQLRequest;
use futures::future::Future;

use crate::state::State;

pub fn graphiql(req: HttpRequest) -> HttpResponse {

    let html = graphiql_source(&format!("http://{}/graphql", &req.connection_info().host()));
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

pub fn graphql(
    state: web::Data<State>,
    request: web::Json<GraphQLRequest>
) -> impl Future<Item=HttpResponse, Error=actix_web::Error> {

    let schema = state.graphql_schema.clone();
    web::block(move || {
        let res = request.execute(&schema, &());
        Ok::<_, serde_json::error::Error>(serde_json::to_string(&res)?)
    })
    .map_err(actix_web::Error::from)
    .and_then(|query| {
        Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(query))
    })
}
