use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub new_encounter: char,
    pub set_initiative: char,
    pub quit_app: char,
    pub unselect_all: char,
    pub move_down: char,
    pub move_up: char,
    pub peek_down: char,
    pub peek_up: char,
    pub lower_health: char,
    pub increase_health: char,
    pub search_for_new_creature: char,
    pub insert_new_player: char,
    pub delete_creature: char,
    pub set_creature_description: char,
    pub duplicate_creature: char,
}

impl Default for KeyBindings {
    fn default() -> Self {
        let default = Self {
            new_encounter: 'e',
            set_initiative: 'i',
            quit_app: 'q',
            unselect_all: 'u',
            move_down: 'j',
            move_up: 'k',
            peek_down: 'J',
            peek_up: 'K',
            lower_health: 'h',
            increase_health: 'l',
            search_for_new_creature: 's',
            insert_new_player: 'c',
            delete_creature: 'D',
            set_creature_description: 'd',
            duplicate_creature: 'x',
        };
    }
}

#[cfg(unix)]
fn get_keymap_config_location() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(format!(
        "{home}/.config/wtii/keybindings.yml"
    )))
}
