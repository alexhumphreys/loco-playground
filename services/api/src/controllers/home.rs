use axum::{
    extract::{Extension, Host, Request},
    http::{Method, StatusCode},
    middleware::Next,
    response::Response,
};
use loco_rs::prelude::*;
use openfga::openfga::make_tuple;
use openfga::openfga::{OpenFGAClient, TupleKeys};

use crate::views::home::HomeResponse;

async fn current(Extension(c): Extension<OpenFGAClient>) -> Result<Json<HomeResponse>> {
    let tuple = make_tuple("user:alex", "reader", "document:id123789");
    let tuple = TupleKeys {
        tuple_keys: vec![tuple.clone()],
    };
    let _x = c.write_relationship_tuple(tuple).await;
    format::json(HomeResponse::new("loco"))
}

async fn hello(
    Path(some_id): Path<String>,
    Extension(_c): Extension<OpenFGAClient>,
) -> Result<Json<HomeResponse>> {
    format::json(HomeResponse::new(&some_id.to_string()))
}

pub async fn auth(
    Path(resource_id): Path<(String)>,
    method: Method,
    mut req: Request,
    next: Next,
) -> Result<Response> {
    println!("{:?}", req.uri());
    let auth_action = match method {
        Method::GET => "read",
        Method::POST => "write",
        Method::DELETE => "delete",
        _ => "todo_not_matched_from_method",
    };
    println!(
        "check if user `{}` can `{}` on resource `{}` with id `{}`",
        "todo_user_id", auth_action, "todo_resource_name", resource_id
    );
    Ok(next.run(req).await)
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/", get(current))
        .add("/hello/:some_id", get(hello))
    // TODO investigate https://docs.rs/axum/0.2.3/axum/#to-individual-handlers
    // https://github.com/tokio-rs/axum/issues/298
    //
    //.route_layer(middleware::from_fn(auth))
}
