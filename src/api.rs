use crate::creature::ApiCreatureSearchItem;
use reqwest::{Client, Error as ReqwestError, Response};
use serde_json::{from_str, Value, Error as SerdeError};
use std::fmt;

static API_BASE_URL: &str = "https://api.open5e.com";

#[derive(Debug)]
pub enum ApiError {
    Request(ReqwestError),
    ResponseText(ReqwestError),
    Parse(SerdeError),
    Unexpected(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Request(e) => write!(f, "Unable to make API request: {}", e),
            ApiError::ResponseText(e) => write!(f, "Unable to parse string: {}", e),
            ApiError::Parse(e) => write!(f, "Unable to parse response: {}", e),
            ApiError::Unexpected(e) => write!(f, "Unexpected error: {}", e),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<ReqwestError> for ApiError {
    fn from(e: ReqwestError) -> Self {
        ApiError::Request(e)
    }
}

impl From<SerdeError> for ApiError {
    fn from(e: SerdeError) -> Self {
        ApiError::Parse(e)
    }
}

pub trait ApiCall {
    fn monster_search(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Response, ReqwestError>> + Send;
}

pub struct MonsterSearch;

impl ApiCall for MonsterSearch {
    async fn monster_search(&self, name: &str) -> Result<Response, ReqwestError> {
        let client = Client::new();
        client
            .get(format!("{}/monsters/?search={}", API_BASE_URL, name))
            .send()
            .await
    }
}

pub async fn search_for_creature<T: ApiCall>(
    api: &T,
    name: &str,
) -> Result<Vec<ApiCreatureSearchItem>, ApiError> {
    let creatures_resp = api.monster_search(name).await.map_err(ApiError::Request)?;
    let resp_str = creatures_resp.text().await.map_err(ApiError::ResponseText)?;
    let parsed_data = parse_json_response(&resp_str)?;
    Ok(parsed_data)
}

pub fn parse_json_response(data: &str) -> Result<Vec<ApiCreatureSearchItem>, SerdeError> {
    let val: Value = from_str(data)?;
    serde_json::from_value(val["results"].clone())
}
