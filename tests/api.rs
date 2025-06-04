use reqwest::Client;
use std::fs;

fn load_mock_creature_json(file: &str) -> String {
    fs::read_to_string(format!("tests/fixtures/{}", file))
        .expect(format!("Failed to read the json file: {}", file).as_str())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;
    use reqwest::{Error, Response, StatusCode};
    use wtii::api::{search_for_creature, ApiCall};

    struct MockMonsterSearchOneCreature;
    struct MockMonsterSearchTimeoutError;

    impl ApiCall for MockMonsterSearchOneCreature {
        async fn monster_search(&self, name: &str) -> Result<Response, Error> {
            let endpoint: &str = &format!("/monsters/?search={}", name);
            let mut server = mockito::Server::new_async().await;
            let _m = server
                .mock("GET", endpoint)
                .with_status(200)
                .with_header("content-type", "application/json")
                .with_body(load_mock_creature_json("single_creature_response.json"))
                .create_async()
                .await;
            let client = Client::new();
            client
                .get(format!("{}{}", &server.url(), endpoint))
                .send()
                .await
        }
    }

    impl ApiCall for MockMonsterSearchTimeoutError {
        async fn monster_search(&self, name: &str) -> Result<Response, Error> {
            let endpoint: &str = &format!("/monsters/?search={}", name);
            let mut server = mockito::Server::new_async().await;
            let _m = server
                .mock("GET", endpoint)
                .with_status(408)
                .create_async()
                .await;
            let client = Client::new();
            client
                .get(format!("{}{}", &server.url(), endpoint))
                .send()
                .await
        }
    }

    #[tokio::test]
    async fn test_search_for_one_creature_ok() {
        let mock = MockMonsterSearchOneCreature;
        let res = search_for_creature(&mock, "mock-call").await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_search_for_one_creature_timeout_error() {
        let mock = MockMonsterSearchTimeoutError;
        let res = search_for_creature(&mock, "mock-call").await;
        assert!(res.is_err());
    }
}
