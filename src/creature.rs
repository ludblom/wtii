use rand::random_range;
use ratatui::prelude::Color;
use ratatui::style::palette::tailwind::{GREEN, RED, YELLOW};
use ratatui::{
    text::Line,
    widgets::{ListItem, ListState},
};
use serde::Deserialize;
use std::{fmt, fs};
use yaml_rust2::YamlLoader;

const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;
const DEAD_TEXT_FG_COLOR: Color = RED.c500;
const NO_INITIATIVE_STYLE: Color = YELLOW.c300;

pub struct CreatureList {
    pub items: Vec<CreatureItem>,
    pub state: ListState,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Reaction {
    pub name: String,
    pub desc: String,
}

impl fmt::Display for Reaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.desc)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Speed {
    pub walk: Option<i64>,
    pub fly: Option<i64>,
    pub swim: Option<i64>,
    pub burrow: Option<i64>,
}

impl fmt::Display for Speed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(val) = self.walk {
            parts.push(format!("Walk: {val}"));
        }
        if let Some(val) = self.fly {
            parts.push(format!("Fly: {val}"));
        }
        if let Some(val) = self.swim {
            parts.push(format!("Swim: {val}"));
        }
        if let Some(val) = self.burrow {
            parts.push(format!("Burrow: {val}"));
        }
        write!(f, "{}", parts.join(", "))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Skills {
    pub athletics: Option<i64>,
    pub perception: Option<i64>,
    pub stealth: Option<i64>,
}

impl fmt::Display for Skills {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if let Some(val) = self.athletics {
            parts.push(format!("Athletics: {val}"));
        }
        if let Some(val) = self.perception {
            parts.push(format!("Perception: {val}"));
        }
        if let Some(val) = self.stealth {
            parts.push(format!("Stealth: {val}"));
        }
        write!(f, "{}", parts.join(", "))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Action {
    pub name: String,
    pub desc: String,
    pub attack_bonus: Option<i64>,
    pub damage_dice: Option<String>,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        parts.push(format!("{}: {}", self.name, self.desc));
        if let Some(val) = self.attack_bonus {
            parts.push(format!("(Attack Bonus: {val})"));
        }
        if let Some(val) = self.damage_dice.clone() {
            parts.push(format!("(Damage Dice: {val})"));
        }
        write!(f, "{}", parts.join(", "))
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpecialAbility {
    pub name: String,
    pub desc: String,
}

impl fmt::Display for SpecialAbility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.name, self.desc)
    }
}

#[derive(Debug, Deserialize)]
pub struct ApiCreatureSearchItem {
    pub name: String,
    pub desc: Option<String>,
    pub size: Option<String>,
    pub subtype: Option<String>,
    pub group: Option<String>,
    pub alignment: Option<String>,
    pub armor_class: Option<i64>,
    pub armor_desc: Option<String>,
    pub hit_points: Option<u64>,
    pub hit_dice: Option<String>,
    pub speed: Option<Speed>,
    pub strength: Option<i64>,
    pub dexterity: Option<i64>,
    pub constitution: Option<i64>,
    pub intelligence: Option<i64>,
    pub wisdom: Option<i64>,
    pub charisma: Option<i64>,
    pub strength_save: Option<i64>,
    pub dexterity_save: Option<i64>,
    pub constitution_save: Option<i64>,
    pub intelligence_save: Option<i64>,
    pub wisdom_save: Option<i64>,
    pub charisma_save: Option<i64>,
    pub perception: Option<i64>,
    pub skills: Option<Skills>,
    pub damage_vulnerabilities: Option<String>,
    pub damage_resistances: Option<String>,
    pub damage_immunities: Option<String>,
    pub condition_immunities: Option<String>,
    pub senses: Option<String>,
    pub languages: Option<String>,
    pub challenge_rating: Option<String>,
    pub actions: Option<Vec<Action>>,
    pub reactions: Option<Vec<Reaction>>,
    pub legendary_desc: Option<String>,
    pub legendary_actions: Option<Vec<Action>>,
    pub special_abilities: Option<Vec<SpecialAbility>>,
    pub spell_list: Option<Vec<String>>,
    pub document_slug: Option<String>,
    pub document_title: Option<String>,
    pub document_license_url: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct CreatureItem {
    pub name: String,
    pub desc: Option<String>,
    pub status: Status,
    pub faction: Faction,
    pub initiative: Option<i64>,
    pub max_hit_points: u64,
    pub hit_points: u64,
    pub hit_dice: Option<String>,
    pub armor_class: Option<i64>,
    pub armor_desc: Option<String>,
    pub speed: Option<Speed>,
    pub size: Option<String>,
    pub strength: Option<i64>,
    pub dexterity: Option<i64>,
    pub constitution: Option<i64>,
    pub intelligence: Option<i64>,
    pub wisdom: Option<i64>,
    pub charisma: Option<i64>,
    pub strength_save: Option<i64>,
    pub dexterity_save: Option<i64>,
    pub constitution_save: Option<i64>,
    pub intelligence_save: Option<i64>,
    pub wisdom_save: Option<i64>,
    pub charisma_save: Option<i64>,
    pub perception: Option<i64>,
    pub skills: Option<Skills>,
    pub damage_vulnerabilities: Option<String>,
    pub damage_resistances: Option<String>,
    pub damage_immunities: Option<String>,
    pub condition_immunities: Option<String>,
    pub senses: Option<String>,
    pub languages: Option<String>,
    pub challenge_rating: Option<String>,
    pub actions: Option<Vec<Action>>,
    pub legendary_actions: Option<Vec<Action>>,
    pub reactions: Option<Vec<Reaction>>,
    pub special_abilities: Option<Vec<SpecialAbility>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Attack {
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Default)]
pub enum Status {
    #[default]
    Alive,
    Dead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Default)]
pub enum Faction {
    Player,
    #[default]
    Npc,
}

impl CreatureItem {
    pub fn new_player(name: &str, desc: Option<&str>) -> Self {
        Self {
            status: Status::Alive,
            faction: Faction::Player,
            name: name.to_string(),
            initiative: None,
            max_hit_points: 1,
            hit_points: 1,
            hit_dice: None,
            armor_class: None,
            armor_desc: None,
            desc: desc.map(|d| d.to_string()),
            speed: None,
            size: None,
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
            wisdom_save: None,
            charisma_save: None,
            perception: None,
            skills: None,
            damage_vulnerabilities: None,
            damage_resistances: None,
            damage_immunities: None,
            condition_immunities: None,
            senses: None,
            languages: None,
            challenge_rating: None,
            actions: None,
            legendary_actions: None,
            reactions: None,
            special_abilities: None,
        }
    }

    pub fn new_npc(api_creature: &ApiCreatureSearchItem) -> Self {
        Self {
            status: Status::Alive,
            faction: Faction::Npc,
            name: api_creature.name.clone(),
            initiative: {
                let initiative_modifier: i64 = (api_creature.dexterity.unwrap_or(0) - 10) / 2;
                Some(random_range(1..21) + initiative_modifier)
            },
            max_hit_points: api_creature.hit_points.unwrap(),
            hit_points: api_creature.hit_points.unwrap(),
            hit_dice: api_creature.hit_dice.clone(),
            armor_class: api_creature.armor_class,
            armor_desc: api_creature.armor_desc.clone(),
            desc: None,
            speed: api_creature.speed.clone(),
            size: api_creature.size.clone(),
            strength: api_creature.strength,
            dexterity: api_creature.dexterity,
            constitution: api_creature.constitution,
            intelligence: api_creature.intelligence,
            wisdom: api_creature.wisdom,
            charisma: api_creature.charisma,
            strength_save: api_creature.strength_save,
            dexterity_save: api_creature.dexterity_save,
            constitution_save: api_creature.constitution_save,
            intelligence_save: api_creature.intelligence_save,
            wisdom_save: api_creature.wisdom_save,
            charisma_save: api_creature.charisma_save,
            perception: api_creature.perception,
            skills: api_creature.skills.clone(),
            damage_vulnerabilities: api_creature.damage_vulnerabilities.clone(),
            damage_resistances: api_creature.damage_resistances.clone(),
            damage_immunities: api_creature.damage_immunities.clone(),
            condition_immunities: api_creature.condition_immunities.clone(),
            senses: api_creature.senses.clone(),
            languages: api_creature.languages.clone(),
            challenge_rating: api_creature.challenge_rating.clone(),
            actions: api_creature.actions.clone(),
            legendary_actions: api_creature.legendary_actions.clone(),
            reactions: api_creature.reactions.clone(),
            special_abilities: api_creature.special_abilities.clone(),
        }
    }
}

impl CreatureList {
    pub fn add_new_creature(&mut self, creature_item: CreatureItem) {
        if creature_item.initiative.is_none() {
            self.items.insert(0, creature_item);
        } else {
            self.items.push(creature_item);
        }
        self.sort_creature_list();
    }

    pub fn sort_creature_list(&mut self) {
        self.items.sort_by(|creature_a, creature_b| {
            match (&creature_a.initiative, &creature_b.initiative) {
                (None, None) => std::cmp::Ordering::Equal,
                (None, _) => std::cmp::Ordering::Less,
                (_, None) => std::cmp::Ordering::Greater,
                (Some(creature_a_val), Some(creature_b_val)) => creature_b_val.cmp(creature_a_val),
            }
        })
    }
}

impl Default for ApiCreatureSearchItem {
    fn default() -> Self {
        ApiCreatureSearchItem {
            name: "Test Creature".to_string(),
            desc: Some("Default desc".to_string()),
            size: None,
            subtype: None,
            group: None,
            alignment: None,
            armor_class: Some(18),
            armor_desc: None,
            hit_points: Some(20),
            hit_dice: None,
            speed: None,
            strength: None,
            dexterity: Some(30),
            constitution: Some(12),
            intelligence: Some(13),
            wisdom: Some(14),
            charisma: Some(15),
            strength_save: None,
            dexterity_save: None,
            constitution_save: None,
            intelligence_save: None,
            wisdom_save: None,
            charisma_save: None,
            perception: None,
            skills: None,
            damage_vulnerabilities: None,
            damage_resistances: None,
            damage_immunities: None,
            condition_immunities: None,
            senses: None,
            languages: None,
            challenge_rating: None,
            actions: None,
            reactions: None,
            legendary_desc: None,
            legendary_actions: None,
            special_abilities: None,
            spell_list: None,
            document_slug: None,
            document_title: None,
            document_license_url: None,
        }
    }
}

#[cfg(unix)]
fn get_config_file_location() -> Option<String> {
    let home = match std::env::var("HOME") {
        Ok(home) => home,
        Err(_) => return None,
    };
    Some(format!("{home}/.config/wtii/default.yml"))
}

#[cfg(windows)]
fn get_config_file_location() -> Option<String> {
    let userprofile = match std::env::var("USERPROFILE") {
        Ok(userprofile) => userprofile,
        Err(_) => return None,
    };
    Some(format!("{}\\Documents\\wtii\\default.yml", userprofile))
}

impl Default for CreatureList {
    fn default() -> Self {
        let empty_creature_list = Self {
            items: Vec::new(),
            state: ListState::default(),
        };

        let config_path: String = match get_config_file_location() {
            Some(path) => path,
            None => return empty_creature_list,
        };

        let yaml_str = match fs::read_to_string(config_path) {
            Ok(yaml_str) => yaml_str,
            Err(_) => return empty_creature_list,
        };

        let docs = match YamlLoader::load_from_str(&yaml_str) {
            Ok(docs) => docs,
            Err(_) => return empty_creature_list,
        };

        let doc = &docs[0];
        let mut items = Vec::new();

        if let Some(players) = doc["players"].as_vec() {
            for player in players {
                let name = player["name"].as_str().unwrap_or("Unknown");
                let desc = player["desc"].as_str();
                items.push(CreatureItem::new_player(name, desc));
            }
        }
        let state = ListState::default();
        Self { items, state }
    }
}

impl From<&CreatureItem> for ListItem<'_> {
    fn from(value: &CreatureItem) -> Self {
        if value.desc.is_some() {
            let line = match value.status {
                Status::Alive => {
                    if value.initiative.is_some() {
                        Line::styled(
                            format!(" ✓ {} ({})", value.name, value.desc.as_ref().unwrap()),
                            COMPLETED_TEXT_FG_COLOR,
                        )
                    } else {
                        Line::styled(
                            format!(" ✓ {} ({})", value.name, value.desc.as_ref().unwrap()),
                            NO_INITIATIVE_STYLE,
                        )
                    }
                }
                Status::Dead => Line::styled(
                    format!(" X {} ({})", value.name, value.desc.as_ref().unwrap()),
                    DEAD_TEXT_FG_COLOR,
                ),
            };
            ListItem::new(line)
        } else {
            let line = match value.status {
                Status::Alive => {
                    if value.initiative.is_some() {
                        Line::styled(format!(" ✓ {}", value.name), COMPLETED_TEXT_FG_COLOR)
                    } else {
                        Line::styled(format!(" ✓ {}", value.name), NO_INITIATIVE_STYLE)
                    }
                }
                Status::Dead => Line::styled(format!(" X {}", value.name), DEAD_TEXT_FG_COLOR),
            };
            ListItem::new(line)
        }
    }
}
