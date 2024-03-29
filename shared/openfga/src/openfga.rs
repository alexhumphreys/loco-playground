use crate::error::Errors;
use crate::error::OpenFGAError;
use reqwest::header::HeaderMap;
use reqwest::{self, Method, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use tracing;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum ReqwestResponse<T> {
    Error(OpenFGAError),
    Success(T),
}

#[derive(Debug, Clone)]
pub struct OpenFGAClient {
    client: reqwest::Client,
    config: OpenFGAConfig,
    store_id: Option<String>,
    model_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OpenFGAConfig {
    pub base_url: String,
    pub api_token: Option<String>,
}

impl OpenFGAClient {
    async fn make_request<T, B>(
        &self,
        path: &str,
        method: Method,
        body: Option<B>,
    ) -> Result<T, Errors>
    where
        T: for<'de> Deserialize<'de>,
        B: Serialize + Debug,
    {
        let url = Url::parse(&format!("{}{}", self.config.base_url, path))?;
        let req = self
            .client
            .request(method, url)
            .headers(self.auth_headers());
        let req = match body {
            Some(body) => {
                println!("body: {:?}", body);
                req.json(&body)
            }
            None => req,
        };
        println!("request: {:?}", req);
        tracing::debug!("request being sent: {:?}", req);
        let res = req.send().await?;

        match res.json::<ReqwestResponse<T>>().await? {
            ReqwestResponse::Success(body) => Ok(body),
            ReqwestResponse::Error(err) => {
                println!("error: {:?}", err);
                Err(Errors::OpenFGAErrorRespone(err))
            }
        }
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.config.api_token {
            headers.insert("Authorization", token.parse().unwrap());
        }
        headers
    }
}

impl OpenFGAClient {
    pub async fn create_data_store(
        &self,
        store: CreateDataStoreSchema,
    ) -> Result<CreateDataStoreResponse, Errors> {
        let res: CreateDataStoreResponse = self
            .make_request("/stores", Method::POST, Some(store))
            .await?;
        Ok(res)
    }

    pub async fn update_relationship_tuple(
        &self,
        tuples: RelationshipAction,
    ) -> Result<WriteRelationshipTupleResponse, Errors> {
        let body = WriteRelationshipTupleSchema {
            authorization_model_id: self.model_id.clone().ok_or(Errors::MissingModelId)?,
            relationship_action: tuples,
        };
        let res: WriteRelationshipTupleResponse = self
            .make_request(
                &format!(
                    "/stores/{}/write",
                    self.store_id.clone().ok_or(Errors::MissingStoreId)?
                ),
                Method::POST,
                Some(body),
            )
            .await?;
        Ok(res)
    }

    pub async fn write_relationship_tuple(
        &self,
        tuples: TupleKeys,
    ) -> Result<WriteRelationshipTupleResponse, Errors> {
        self.update_relationship_tuple(RelationshipAction::Writes(tuples))
            .await
    }

    pub async fn delete_relationship_tuple(
        &self,
        tuples: TupleKeys,
    ) -> Result<WriteRelationshipTupleResponse, Errors> {
        self.update_relationship_tuple(RelationshipAction::Deletes(tuples))
            .await
    }

    pub async fn write_authorization_model(
        &self,
        model: Value,
    ) -> Result<WriteAuthorizationModelResponse, Errors> {
        let res: WriteAuthorizationModelResponse = self
            .make_request(
                &format!(
                    "/stores/{}/authorization-models",
                    self.store_id.clone().ok_or(Errors::MissingStoreId)?
                ),
                Method::POST,
                Some(model),
            )
            .await?;
        Ok(res)
    }

    pub async fn get_authorization_models(&self) -> Result<GetAuthorizationModelsResponse, Errors> {
        let body: Option<()> = None;
        let res: GetAuthorizationModelsResponse = self
            .make_request(
                &format!(
                    "/stores/{}/authorization-models",
                    self.store_id.clone().ok_or(Errors::MissingStoreId)?
                ),
                Method::GET,
                body,
            )
            .await?;
        Ok(res)
    }

    // TODO: some way to only call this when you have a store/model id?
    pub async fn check(&self, tuple: RelationshipTuple) -> Result<CheckResponse, Errors> {
        match (&self.store_id, &self.model_id) {
            (Some(store_id), Some(model_id)) => {
                let res: CheckResponse = self
                    .make_request(
                        &format!("/stores/{}/check", store_id),
                        Method::POST,
                        Some(CheckRequest {
                            authorization_model_id: model_id.clone(),
                            tuple_key: tuple,
                        }),
                    )
                    .await?;
                Ok(res)
            }
            (None, _) => Err(Errors::MissingStoreId),
            (_, None) => Err(Errors::MissingStoreId),
        }
    }
}

pub fn create_openfga_client(
    config: OpenFGAConfig,
    store_id: Option<String>,
    model_id: Option<String>,
) -> OpenFGAClient {
    OpenFGAClient {
        client: reqwest::Client::new(),
        config,
        store_id,
        model_id,
    }
}

pub async fn create_openfga_client_full(
    config: OpenFGAConfig,
    store_id: String,
) -> Result<OpenFGAClient, Errors> {
    let client = OpenFGAClient {
        client: reqwest::Client::new(),
        config: config.clone(),
        store_id: Some(store_id.clone()),
        model_id: None,
    };
    let models = client.get_authorization_models().await?;
    match models.authorization_models.first() {
        Some(model) => Ok(OpenFGAClient {
            client: reqwest::Client::new(),
            config,
            store_id: Some(store_id),
            model_id: Some(model.id.clone()),
        }),
        None => Err(Errors::NoModelForClient),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDataStoreSchema {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateDataStoreResponse {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    // {"id":"01HJ3FP23AS376NVDBECMZBAPT", "name":"FGA Demo Store", "created_at":"2023-12-20T11:27:27.850720014Z", "updated_at":"2023-12-20T11:27:27.850720014Z"}%
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RelationshipTuple {
    user: String,
    relation: String,
    object: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TupleKeys {
    pub tuple_keys: Vec<RelationshipTuple>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum RelationshipAction {
    Writes(TupleKeys),
    Deletes(TupleKeys),
}
#[derive(Serialize, Deserialize, Debug)]
pub struct WriteRelationshipTupleSchema {
    pub authorization_model_id: String,

    #[serde(flatten)]
    pub relationship_action: RelationshipAction,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WriteRelationshipTupleResponse {
    // {}%
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct WriteAuthorizationModelResponse {
    // {"authorization_model_id":"01HJ8QTTREQ7QJ9P5BCK0RHH5F"}
    authorization_model_id: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AuthorizationModel {
    id: String,
    type_definitions: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GetAuthorizationModelsResponse {
    /*
         * {
      "authorization_models": [
        {
          "id": "01G50QVV17PECNVAHX1GG4Y5NC",
          "type_definitions": [...]
        },
        {
          "id": "01G4ZW8F4A07AKQ8RHSVG9RW04",
          "type_definitions": [...]
        },
      ],
      "continuation_token": "eyJwayI6IkxBVEVTVF9OU0NPTkZJR19hdXRoMHN0b3JlIiwic2siOiIxem1qbXF3MWZLZExTcUoyN01MdTdqTjh0cWgifQ=="
    }*/
    authorization_models: Vec<AuthorizationModel>,
    continuation_token: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckRequest {
    // {"allowed":true}
    authorization_model_id: String,
    tuple_key: RelationshipTuple,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CheckResponse {
    // {"allowed":true}
    allowed: bool,
}

pub fn make_tuple(user: &str, relation: &str, object: &str) -> RelationshipTuple {
    RelationshipTuple {
        user: user.to_string(),
        relation: relation.to_string(),
        object: object.to_string(),
    }
}

pub fn make_tuples(tuples: Vec<(String, String, String)>) -> TupleKeys {
    let t: Vec<RelationshipTuple> = tuples
        .iter()
        .map(|t| RelationshipTuple {
            user: t.0.to_string(),
            relation: t.1.to_string(),
            object: t.2.to_string(),
        })
        .collect();
    TupleKeys { tuple_keys: t }
}

#[cfg(test)]
mod tests {
    use crate::openfga::*;

    fn model_string() -> String {
        r#"
        {"schema_version":"1.1","type_definitions":[{"type":"user"},{"type":"document","relations":{"reader":{"this":{}},"writer":{"this":{}},"owner":{"this":{}}},"metadata":{"relations":{"reader":{"directly_related_user_types":[{"type":"user"}]},"writer":{"directly_related_user_types":[{"type":"user"}]},"owner":{"directly_related_user_types":[{"type":"user"}]}}}}]}
        "#
        .to_string()
    }

    #[tokio::test]
    async fn test_new_openfga_create_data_store() {
        let config = OpenFGAConfig {
            base_url: "http://127.0.0.1:8080".to_string(),
            api_token: None,
        };

        let openfga_client = create_openfga_client(config.clone(), None, None);

        let store_name = "newfoobar".to_string();

        println!("creating data store");
        let res = openfga_client
            .create_data_store(CreateDataStoreSchema { name: store_name })
            .await;

        println!("res {:?}", res);
        let res = res.unwrap();

        let store_id = res.id;
        let openfga_client = create_openfga_client(config.clone(), Some(store_id.clone()), None);

        println!("creating model");
        let v: Value = serde_json::from_str(&model_string()).unwrap();
        let res = openfga_client.write_authorization_model(v).await.unwrap();

        let model_id = res.authorization_model_id;
        let openfga_client = create_openfga_client(config, Some(store_id), Some(model_id));

        let tuple = make_tuple("user:789", "reader", "document:z");
        let _res = openfga_client
            .write_relationship_tuple(TupleKeys {
                tuple_keys: vec![tuple.clone()],
            })
            .await
            .unwrap();

        let res = openfga_client.check(tuple).await.unwrap();
        assert_eq!(res.allowed, true);

        let res = openfga_client.get_authorization_models().await.unwrap();
        assert_eq!(res.authorization_models.len() > 0, true);
    }

    #[tokio::test]
    async fn test_openfga_json_serialize() {
        let json = WriteRelationshipTupleSchema {
            authorization_model_id: "123".to_string(),
            relationship_action: RelationshipAction::Writes(TupleKeys {
                tuple_keys: vec![RelationshipTuple {
                    user: "user:789".to_string(),
                    relation: "reader".to_string(),
                    object: "document:z".to_string(),
                }],
            }),
        };
        println!("{:?}", serde_json::to_string(&json));
        assert_eq!(true, true);
    }
}
