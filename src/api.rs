use crate::creature::CreatureItem;
use json::JsonValue;
use rand::random_range;
use reqwest::{Client, Error, Response};

pub async fn search_for_creature(name: &str) -> Result<Vec<CreatureItem>, String> {
    let creatures_resp: Response = match get_api_response(name).await {
        Ok(resp) => resp,
        Err(e) => return Err(format!("Unable to make API request: {}", e.to_string())),
    };

    let resp = creatures_resp.text().await;

    let resp_str = match &resp {
        Ok(text) => text.as_str(),
        Err(e) => return Err(e.to_string()),
    };

    let parsed = json::parse(resp_str);

    let parsed_values = match &parsed {
        Ok(val) => val,
        Err(e) => return Err(e.to_string()),
    };

    parse_json_response(&parsed_values).ok_or("Unable to parse json response from API".to_string())
}

async fn get_api_response(name: &str) -> Result<Response, Error> {
    let client = Client::new();
    client
        .get(format!("https://api.open5e.com/monsters/?search={}", name))
        .send()
        .await
}

fn parse_json_response(json_resp: &JsonValue) -> Option<Vec<CreatureItem>> {
    let mut creature_list: Vec<CreatureItem> = vec![];
    if let JsonValue::Array(results) = &json_resp["results"] {
        for value in results.iter() {
            let mut creature: CreatureItem = CreatureItem::default();

            let name = value["name"]
                .as_str()
                .expect("'name' field is not present.")
                .to_string();
            creature.name = name;

            let strength: Option<i64> = value["strength"].as_i64().or(None);
            creature.strength = strength;

            let constitution: Option<i64> = value["constitution"].as_i64().or(None);
            creature.constitution = constitution;

            let intelligence: Option<i64> = value["intelligence"].as_i64().or(None);
            creature.intelligence = intelligence;

            let wisdom: Option<i64> = value["wisdom"].as_i64().or(None);
            creature.wisdom = wisdom;

            let charisma: Option<i64> = value["charisma"].as_i64().or(None);
            creature.charisma = charisma;

            let dexterity: i64 = value["dexterity"].as_i64().unwrap();
            creature.dexterity = Some(dexterity);

            let initiative_modifier: i64 = (dexterity - 10) / 2;
            let initiative: Option<i64> = Some(random_range(1..21) + initiative_modifier);
            creature.initiative = initiative;

            let hit_points: Option<u64> = value["hit_points"].as_u64().or(None);
            creature.hit_points = hit_points;

            let hit_dice: Option<String> = match &value["hit_dice"] {
                JsonValue::String(hit_dice) => Some(hit_dice.as_str().to_string()),
                _ => None,
            };
            creature.hit_dice = hit_dice;

            let armor_class: Option<i64> = value["armor_class"].as_i64().or(None);
            creature.armor_class = armor_class;

            let armor_desc: Option<String> = match &value["armor_desc"] {
                JsonValue::String(armor_desc) => Some(armor_desc.as_str().to_string()),
                _ => None,
            };
            creature.armor_desc = armor_desc;

            let size: Option<String> = match &value["size"] {
                JsonValue::String(size) => Some(size.as_str().to_string()),
                _ => None,
            };
            creature.size = size;

            let strength_save: Option<i64> = value["strength_save"].as_i64().or(None);
            creature.strength_save = strength_save;

            let dexterity_save: Option<i64> = value["dexterity_save"].as_i64().or(None);
            creature.dexterity_save = dexterity_save;

            let constitution_save: Option<i64> = value["constitution_save"].as_i64().or(None);
            creature.constitution_save = constitution_save;

            let intelligence_save: Option<i64> = value["intelligence_save"].as_i64().or(None);
            creature.intelligence_save = intelligence_save;

            let wisdom_save: Option<i64> = value["wisdom_save"].as_i64().or(None);
            creature.wisdom_save = wisdom_save;

            let charisma_save: Option<i64> = value["charisma_save"].as_i64().or(None);
            creature.charisma_save = charisma_save;

            let perception: Option<String> = match &value["perception"] {
                JsonValue::String(perception) => Some(perception.as_str().to_string()),
                _ => None,
            };
            creature.perception = perception;

            let languages: Option<String> = match &value["languages"] {
                JsonValue::String(languages) => Some(languages.as_str().to_string()),
                _ => None,
            };
            creature.languages = languages;

            // TODO Need to be implemented (damage_vulnerabilities validated)
            let damage_vulnerabilities: Option<String> = match &value["damage_vulnerabilities"] {
                JsonValue::String(damage_vulnerabilities) => {
                    Some(damage_vulnerabilities.as_str().to_string())
                }
                _ => None,
            };
            creature.damage_vulnerabilities = damage_vulnerabilities;

            // let damage_resistances: Option<Vec<String>> =
            // let damage_immunities: Option<Vec<String>> =
            // let condition_immunities: Option<Vec<String>> =
            // let actions: Option<Vec<Attack>> =
            // let legendary_actions: Option<Vec<Attack>> =
            // let reactions: Option<Vec<Attack>> =

            creature_list.push(creature);
        }
    }
    Some(creature_list)
}
