use crate::creature::ApiCreatureSearchItem;
use reqwest::{Client, Error, Response};
use serde_json::{from_str, Value};

static API_BASE_URL: &str = "https://api.open5e.com";

pub trait ApiCall {
    fn monster_search(
        &self,
        name: &str,
    ) -> impl std::future::Future<Output = Result<Response, Error>> + Send;
}

pub struct MonsterSearch;

impl ApiCall for MonsterSearch {
    async fn monster_search(&self, name: &str) -> Result<Response, Error> {
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
) -> Result<Vec<ApiCreatureSearchItem>, String> {
    let creatures_resp: Response = match api.monster_search(name).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Unable to make API request: {}", e.to_string())),
    };

    let resp = creatures_resp.text().await;

    let resp_str: String = match &resp {
        Ok(text) => text.to_string(),
        Err(e) => return Err(format!("Unable to parse string: {}", e.to_string())),
    };

    let parsed_data = parse_json_response(resp_str);

    match parsed_data {
        Ok(data) => Ok(data),
        Err(e) => Err(format!("Unable to parse response: {}", e)),
    }
}

pub fn parse_json_response(data: String) -> Result<Vec<ApiCreatureSearchItem>, serde_json::Error> {
    let val: Value = from_str(&data)?;
    serde_json::from_value(val["results"].clone())
}
