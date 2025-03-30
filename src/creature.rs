use ratatui::prelude::Color;
use ratatui::style::palette::tailwind::{GREEN, RED};
use ratatui::{
    text::Line,
    widgets::{ListItem, ListState},
};
use serde::Deserialize;

const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;
const DEAD_TEXT_FG_COLOR: Color = RED.c500;

pub struct CreatureList {
    pub items: Vec<CreatureItem>,
    pub state: ListState,
}

#[derive(Debug, Deserialize, Default)]
pub struct CreatureItem {
    pub name: String,
    pub status: Status,
    pub faction: Faction,
    pub initiative: Option<i64>,
    pub hit_points: Option<i64>,
    pub hit_dice: Option<String>,
    pub armor_class: Option<i64>,
    pub armor_desc: Option<String>,
    pub desc: Option<String>,
    // pub speed: Option<String>,
    pub size: Option<String>,
    pub creature_type: Option<String>,
    pub sub_creature_type: Option<String>,
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
    pub perception: Option<String>,
    // pub skills: Option<String>,
    pub damage_vulnerabilities: Option<String>,
    pub damage_resistances: Option<Vec<String>>,
    pub damage_immunities: Option<Vec<String>>,
    pub condition_immunities: Option<Vec<String>>,
    //pub senses: Option<String>,
    pub languages: Option<String>,
    //pub challenge_rating: Option<String>,
    pub actions: Option<Vec<Attack>>,
    pub legendary_actions: Option<Vec<Attack>>,
    pub reactions: Option<Vec<Attack>>,
}

#[derive(Debug, Deserialize)]
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
    pub fn new(status: Status, name: &str, desc: Option<&str>) -> Self {
        Self {
            status,
            faction: Faction::Player,
            name: name.to_string(),
            initiative: None,
            hit_points: None,
            hit_dice: None,
            armor_class: None,
            armor_desc: None,
            desc: if desc.is_none() {
                None
            } else {
                Some(desc.unwrap().to_string())
            },
            size: None,
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
            wisdom_save: None,
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
        }
    }
}

impl From<&CreatureItem> for ListItem<'_> {
    fn from(value: &CreatureItem) -> Self {
        let line = match value.status {
            Status::Alive => Line::styled(format!(" ✓ {}", value.name), COMPLETED_TEXT_FG_COLOR),
            Status::Dead => Line::styled(format!(" X {}", value.name), DEAD_TEXT_FG_COLOR),
        };
        ListItem::new(line)
    }
}

impl FromIterator<(Status, &'static str, Option<&'static str>)> for CreatureList {
    fn from_iter<I: IntoIterator<Item = (Status, &'static str, Option<&'static str>)>>(
        iter: I,
    ) -> Self {
        let items = iter
            .into_iter()
            .map(|(status, name, desc)| CreatureItem::new(status, name, desc))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}
