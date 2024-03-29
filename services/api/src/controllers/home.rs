use axum::Extension;
use loco_rs::prelude::*;
use openfga::openfga::make_tuple;
use openfga::openfga::{OpenFGAClient, TupleKeys};

use crate::views::home::HomeResponse;

async fn current(Extension(c): Extension<OpenFGAClient>) -> Result<Json<HomeResponse>> {
    let tuple = make_tuple("user:alex", "reader", "document:id123789");
    let relationship_action = openfga::openfga::RelationshipAction::Writes(TupleKeys {
        tuple_keys: vec![tuple.clone()],
    });
    let x = c.write_relationship_tuple(relationship_action).await;
    format::json(HomeResponse::new("loco"))
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(current))
}
