use rand::random_range;
use ratatui::prelude::Color;
use ratatui::style::palette::tailwind::{GREEN, RED, YELLOW};
use ratatui::{
    text::Line,
    widgets::{ListItem, ListState},
};
use serde::Deserialize;

const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;
const DEAD_TEXT_FG_COLOR: Color = RED.c500;
const NO_INITIATIVE_STYLE: Color = YELLOW.c300;

pub struct CreatureList {
    pub items: Vec<CreatureItem>,
    pub state: ListState,
}

#[derive(Debug, Deserialize)]
pub struct Reaction {
    pub name: String,
    pub desc: String,
}

#[derive(Debug, Deserialize)]
pub struct Speed {
    pub walk: Option<i64>,
    pub fly: Option<i64>,
    pub swim: Option<i64>,
    pub burrow: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Skills {
    pub athletics: Option<i64>,
    pub perception: Option<i64>,
    pub stealth: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct Action {
    pub name: String,
    pub desc: String,
    pub attack_bonus: Option<i64>,
    pub damage_dice: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SpecialAbility {
    pub name: String,
    pub desc: String,
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

#[derive(Debug, Deserialize, Default)]
pub struct CreatureItem {
    pub name: String,
    pub desc: Option<String>,
    pub status: Status,
    pub faction: Faction,
    pub initiative: Option<i64>,
    pub hit_points: Option<u64>,
    pub hit_dice: Option<String>,
    pub armor_class: Option<i64>,
    pub armor_desc: Option<String>,
    pub speed: Option<Speed>,
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
    pub skills: Option<Skills>,
    pub damage_vulnerabilities: Option<String>,
    pub damage_resistances: Option<Vec<String>>,
    pub damage_immunities: Option<Vec<String>>,
    pub condition_immunities: Option<Vec<String>>,
    pub senses: Option<String>,
    pub languages: Option<String>,
    pub challenge_rating: Option<String>,
    pub actions: Option<Vec<Attack>>,
    pub legendary_actions: Option<Vec<Attack>>,
    pub reactions: Option<Vec<Attack>>,
    pub special_abilities: Option<Vec<SpecialAbility>>,
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
    pub fn new_player(name: &str, desc: Option<&str>) -> Self {
        Self {
            status: Status::Alive,
            faction: Faction::Player,
            name: name.to_string(),
            initiative: None,
            hit_points: Some(1),
            hit_dice: None,
            armor_class: None,
            armor_desc: None,
            desc: if desc.is_none() {
                None
            } else {
                Some(desc.unwrap().to_string())
            },
            speed: None,
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
            hit_points: api_creature.hit_points,
            hit_dice: None,
            armor_class: api_creature.armor_class,
            armor_desc: None,
            desc: api_creature.desc.clone(),
            speed: None,
            size: None,
            creature_type: None,
            sub_creature_type: None,
            strength: None,
            dexterity: api_creature.dexterity,
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
                (Some(creature_a_val), Some(creature_b_val)) => creature_b_val.cmp(&creature_a_val),
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

impl Default for CreatureList {
    fn default() -> Self {
        CreatureList::from_iter([
            // Status, Name, Description, HP, Faction, Armor Class
            // TODO: Load from a config file
            ("Samson", Some("A real bastard.")),
            ("Thaurun", Some("Very nice guy!")),
            ("Borbur", Some("A king.")),
        ])
    }
}

impl From<&CreatureItem> for ListItem<'_> {
    fn from(value: &CreatureItem) -> Self {
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

impl FromIterator<(&'static str, Option<&'static str>)> for CreatureList {
    fn from_iter<I: IntoIterator<Item = (&'static str, Option<&'static str>)>>(iter: I) -> Self {
        let items = iter
            .into_iter()
            .map(|(name, desc)| CreatureItem::new_player(name, desc))
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}
