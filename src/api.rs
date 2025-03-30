use crate::creature::{CreatureItem, Faction, Status};
use json::JsonValue;
use rand::random_range;
use reqwest::{Client, Response};

pub async fn search_for_creature(name: &str) -> Option<Vec<CreatureItem>> {
    let creatures_resp: Response = get_api_response(name).await;
    let parsed = json::parse(&creatures_resp.text().await.ok()?).ok()?;
    parse_json_response(&parsed)
}

async fn get_api_response(name: &str) -> Response {
    let client = Client::new();
    client
        .get(format!("https://api.open5e.com/monsters/?search={}", name))
        .send()
        .await
        .unwrap()
}

fn parse_json_response(json_resp: &JsonValue) -> Option<Vec<CreatureItem>> {
    let mut creature_list: Vec<CreatureItem> = vec![];
    if let JsonValue::Array(results) = &json_resp["results"] {
        for value in results.iter() {
            let name = value["name"]
                .as_str()
                .expect("'name' field is not present.")
                .to_string();

            let dexterity: i64 = value["dexterity"].as_i64().unwrap();
            let initiative_modifier: i64 = (dexterity - 10) / 2;
            let initiative: Option<i64> = Some(random_range(1..21) + initiative_modifier);

            let hit_points: Option<i64> = Some(value["hit_points"].as_i64().unwrap());

            let hit_dice: Option<String> = Some(value["hit_dice"].to_string());

            let armor_class: Option<i64> = value["armor_class"].as_i64();

            let armor_desc: Option<String> = match &value["armor_desc"] {
                JsonValue::String(armor_desc) => Some(armor_desc.as_str().to_string()),
                _ => None,
            };

            let size: Option<String> = Some(value["size"].as_str().unwrap().to_string());

            creature_list.push(CreatureItem {
                name,
                status: Status::Alive,
                faction: Faction::Npc,
                initiative,
                hit_points,
                hit_dice,
                armor_class,
                armor_desc,
                desc: None,
                size,
                creature_type: None,
                sub_creature_type: None,
                strength: None,
                dexterity: None,
                constitution: None,
                intelligence: None,
                wisdom: None,
                charisma: None,
                strength_save: None,
                dexterity_save: None,
                constitution_save: None,
                intelligence_save: None,
                wisdom_sav: None,
                charisma_save: None,
                perception: None,
                damage_vulnerabilities: None,
                damage_resistances: None,
                damage_immunities: None,
                condition_immunities: None,
                languages: None,
                actions: None,
                legendary_actions: None,
                reactions: None,
            });
        }
    }
    Some(creature_list)
}
