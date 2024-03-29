use axum::{async_trait, Extension, Router as AxumRouter};
use loco_rs::{
    app::{AppContext, Initializer},
    Result,
};
use openfga;

pub struct OpenFgaInitializer;

#[async_trait]
impl Initializer for OpenFgaInitializer {
    fn name(&self) -> String {
        "axum-session".to_string()
    }

    async fn after_routes(&self, router: AxumRouter, _ctx: &AppContext) -> Result<AxumRouter> {
        let config = openfga::openfga::OpenFGAConfig {
            base_url: "http://localhost:8080".to_string(),
            api_token: None,
        };

        let client = openfga::openfga::create_openfga_client_full(
            config,
            "01HT4PESMVJPG4KY0127WPA668".to_string(),
        )
        .await
        .unwrap();
        Ok(router.layer(Extension(client)))
    }
}
