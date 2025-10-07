use crate::api::{search_for_creature, ApiError, MonsterSearch};
use crate::creature::{ApiCreatureSearchItem, Faction};
use crate::creature::{CreatureItem, CreatureList, Status};
use color_eyre::Result;
use ratatui::layout::Direction;
use ratatui::text::Text;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Flex, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, Clear, HighlightSpacing, List, ListItem, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};
use std::cmp::PartialEq;
use std::time::Duration;
use tokio::sync::mpsc;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

// Colors
const WTII_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;

// Keybindings
const NEW_ENCOUNTER_KEY: char = 'e';
const SET_INITIATIVE_KEY: char = 'i';
const QUIT_APP_KEY: char = 'q';
const UNSELECT_ALL_KEY: char = 'u';
const MOVE_DOWN_KEY: char = 'j';
const MOVE_UP_KEY: char = 'k';
const PEEK_DOWN_KEY: char = 'J';
const PEEK_UP_KEY: char = 'K';
const LOWER_HEALTH_KEY: char = 'h';
const INCREASE_HEALTH_KEY: char = 'l';
const SEARCH_FOR_NEW_CREATURE_KEY: char = 's';
const INSERT_NEW_PLAYER_KEY: char = 'c';
const DELETE_CREATURE_KEY: char = 'd';
const DUPLICATE_CREATURE_KEY: char = 'x';

#[derive(PartialEq)]
enum TextFormatting {
    Line,
    NewLine,
}

pub struct App {
    creature_list: CreatureList,
    should_exit: bool,
    show_creature_search_popup: bool,
    show_initiative_popup: bool,
    initiative_input: Input,
    creature_search_input: String,
    creature_search_result: Vec<ApiCreatureSearchItem>,
    creature_search_selected: Option<usize>,
    creature_search_loading: bool,
    creature_search_result_rx:
        Option<mpsc::UnboundedReceiver<Result<Vec<ApiCreatureSearchItem>, ApiError>>>,
    increasing_or_decreasing_health: bool,
    health_change: i64,
    creature_info_scroll: u16,

    save_creature_viewing: Option<usize>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            show_creature_search_popup: false,
            creature_list: CreatureList::default(),
            show_initiative_popup: false,
            initiative_input: Input::default(),
            creature_search_input: String::default(),
            creature_search_result: Vec::new(),
            creature_search_selected: None,
            creature_search_loading: false,
            creature_search_result_rx: None,
            increasing_or_decreasing_health: false,
            health_change: 0,
            creature_info_scroll: 0,
            save_creature_viewing: None,
        }
    }
}

impl App {
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;

            // TODO: This is a workaround, use async event streaming instead
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key(key).await;
                };
            }

            if let Some(rx) = &mut self.creature_search_result_rx {
                if let Ok(result) = rx.try_recv() {
                    self.creature_search_loading = false;
                    self.creature_search_result_rx = None;
                    match result {
                        Ok(results) => {
                            self.creature_search_result = results;
                            self.creature_search_selected =
                                if self.creature_search_result.is_empty() {
                                    None
                                } else {
                                    Some(0)
                                };
                        }
                        Err(_) => {
                            self.creature_search_result.clear();
                            self.creature_search_selected = None;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if self.show_creature_search_popup {
            self.handle_creature_search_input(&key).await;
            return;
        }

        if self.show_initiative_popup {
            self.handle_initiative_input(&key);
            return;
        }

        self.handle_general_input(&key);
    }

    fn handle_general_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Char(QUIT_APP_KEY) | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char(UNSELECT_ALL_KEY) => self.select_none(),
            KeyCode::Char(MOVE_DOWN_KEY) | KeyCode::Down => self.select_next(),
            KeyCode::Char(PEEK_DOWN_KEY) => self.peek_on_next(),
            KeyCode::Char(MOVE_UP_KEY) | KeyCode::Up => self.select_previous(),
            KeyCode::Char(PEEK_UP_KEY) => self.peek_on_previous(),
            KeyCode::Char(LOWER_HEALTH_KEY) | KeyCode::Left => self.lower_health(),
            KeyCode::Char(INCREASE_HEALTH_KEY) | KeyCode::Right => self.increase_health(),
            KeyCode::Char(SEARCH_FOR_NEW_CREATURE_KEY) => {
                self.creature_search_selected = None;
                self.show_creature_search_popup = true;
            }
            KeyCode::Char(INSERT_NEW_PLAYER_KEY) => self.insert_new(),
            KeyCode::Char(DELETE_CREATURE_KEY) => self.delete_creature(),
            KeyCode::Char(NEW_ENCOUNTER_KEY) => self.new_encounter(),
            KeyCode::Char(SET_INITIATIVE_KEY) => {
                if self.creature_list.state.selected().is_some() {
                    self.show_initiative_popup = true;
                }
            }
            KeyCode::Char(DUPLICATE_CREATURE_KEY) => self.duplicate_creature(),
            KeyCode::PageDown => self.creature_info_scroll += 1,
            KeyCode::PageUp => {
                if self.creature_info_scroll > 0 {
                    self.creature_info_scroll -= 1;
                }
            }
            _ => {}
        }
    }

    fn duplicate_creature(&mut self) {
        if let Some(i) = self.creature_list.state.selected() {
            if let Some(mut creature) = self.creature_list.items.get(i).cloned() {
                let initiative_modifier: i64 = (creature.dexterity.unwrap_or(0) - 10) / 2;
                creature.initiative = Some(rand::random_range(1..21) + initiative_modifier);
                self.creature_list.items.insert(i + 1, creature);
                self.creature_list.state.select(Some(i + 1));
            }
        }
        self.creature_list.sort_creature_list();
    }

    fn handle_initiative_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Enter => {
                if let Some(i) = self.creature_list.state.selected() {
                    if let Ok(new_initiative) = self.initiative_input.value().parse::<i64>() {
                        self.creature_list.items[i].initiative = Some(new_initiative);
                        self.creature_list.sort_creature_list();
                    }
                }
                self.show_initiative_popup = false;
                self.initiative_input = Input::default();
            }
            KeyCode::Char(QUIT_APP_KEY) | KeyCode::Esc => {
                self.show_initiative_popup = false;
                self.initiative_input = Input::default();
            }
            _ => {
                self.initiative_input.handle_event(&Event::Key(*key));
            }
        }
    }

    async fn handle_creature_search_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Tab => {
                if !self.creature_search_result.is_empty() {
                    if self.creature_search_selected.is_some() {
                        self.creature_search_selected = None;
                    } else {
                        self.creature_search_selected = Some(0);
                    }
                } else {
                    self.creature_search_selected = None;
                }
            }
            KeyCode::Esc => {
                self.show_creature_search_popup = false;
            }
            KeyCode::Backspace => {
                self.creature_search_input.pop();
            }
            KeyCode::Enter => {
                if let Some(selected) = self.creature_search_selected {
                    if let Some(selected_creature) = self.creature_search_result.get(selected) {
                        self.creature_list
                            .add_new_creature(CreatureItem::new_npc(selected_creature));
                        self.show_creature_search_popup = false;
                        return;
                    }
                }

                self.creature_search_loading = true;
                let (tx, rx) = mpsc::unbounded_channel();
                self.creature_search_result_rx = Some(rx);
                let input = self.creature_search_input.clone();
                tokio::spawn(async move {
                    let api = MonsterSearch;
                    let result = search_for_creature(&api, &input).await;
                    let _ = tx.send(result);
                });
            }
            KeyCode::Char(c) => {
                if c == MOVE_DOWN_KEY && self.creature_search_selected.is_some() {
                    if let Some(selected) = self.creature_search_selected {
                        if selected + 1 < self.creature_search_result.len() {
                            self.creature_search_selected = Some(selected + 1);
                        }
                    }
                } else if c == MOVE_UP_KEY && self.creature_search_selected.is_some() {
                    if let Some(selected) = self.creature_search_selected {
                        if selected > 0 {
                            self.creature_search_selected = Some(selected - 1);
                        }
                    }
                } else if self.creature_search_selected.is_none() {
                    self.creature_search_input.push(c);
                }
            }
            _ => {}
        }
    }

    fn new_encounter(&mut self) {
        self.creature_list = CreatureList::default();
    }

    fn select_none(&mut self) {
        self.creature_list.state.select(None);
    }

    fn select_next(&mut self) {
        if self.save_creature_viewing.is_some() {
            self.creature_list
                .state
                .select(self.save_creature_viewing.take());
            self.save_creature_viewing = None;
            return;
        }
        self.health_change = 0;
        self.increasing_or_decreasing_health = false;

        self.move_down_selected_creature();
    }

    fn peek_on_next(&mut self) {
        if self.save_creature_viewing.is_none() {
            self.save_creature_viewing = self.creature_list.state.selected();
        }
        self.move_down_selected_creature();
    }

    fn move_down_selected_creature(&mut self) {
        let creature_count: usize = self.creature_list.items.len();
        match self.creature_list.state.selected() {
            Some(i) => {
                if i + 1 < creature_count {
                    self.creature_list.state.select_next();
                } else {
                    self.creature_list.state.select_first();
                }
            }
            None => {
                if creature_count > 0 {
                    self.creature_list.state.select_first();
                } else {
                    self.creature_list.state.select(None);
                }
            }
        }
    }

    fn select_previous(&mut self) {
        self.health_change = 0;
        self.increasing_or_decreasing_health = false;

        self.move_up_selected_creature();
    }

    fn peek_on_previous(&mut self) {
        if self.save_creature_viewing.is_none() {
            self.save_creature_viewing = self.creature_list.state.selected();
        }
        self.move_up_selected_creature();
    }

    fn move_up_selected_creature(&mut self) {
        let creature_count: usize = self.creature_list.items.len();

        match self.creature_list.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.creature_list.state.select_last();
                } else {
                    self.creature_list.state.select_previous();
                }
            }
            None => {
                if creature_count > 0 {
                    self.creature_list.state.select_last();
                } else {
                    self.creature_list.state.select(None);
                }
            }
        }
    }

    fn delete_creature(&mut self) {
        if let Some(i) = self.creature_list.state.selected() {
            self.creature_list.items.remove(i);
        }
    }

    fn lower_health(&mut self) {
        if let Some(i) = self.creature_list.state.selected() {
            if self.creature_list.items[i].hit_points > 0 {
                self.increasing_or_decreasing_health = true;
                self.creature_list.items[i].hit_points -= 1;
                self.health_change -= 1;
            }
            if self.creature_list.items[i].hit_points == 0 {
                self.creature_list.items[i].status = Status::Dead;
            }
        }
    }

    fn increase_health(&mut self) {
        if let Some(i) = self.creature_list.state.selected() {
            if self.creature_list.items[i].hit_points < self.creature_list.items[i].max_hit_points {
                self.increasing_or_decreasing_health = true;
                self.creature_list.items[i].hit_points += 1;
                self.health_change += 1;
            }
            if self.creature_list.items[i].hit_points > 0 {
                self.creature_list.items[i].status = Status::Alive;
            }
        }
    }

    fn insert_new(&mut self) {
        let creature = CreatureItem::new_npc(&ApiCreatureSearchItem::default());
        self.creature_list.add_new_creature(creature);
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Percentage(30), Constraint::Percentage(70)])
                .areas(main_area);

        App::render_header(header_area, buf);
        App::render_footer(footer_area, buf);

        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);

        if self.show_creature_search_popup {
            let area = App::popup_search_area(main_area);
            App::clear_area(area, buf);
            App::render_creature_search_popup(&self, area, buf);
        }

        if self.show_initiative_popup {
            let area = App::popup_initiative_area(area);
            App::clear_area(area, buf);
            App::render_initiative_popup(&self, area, buf);
        }
    }
}

/// Rendering logic for the app
impl App {
    fn clear_area(area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
    }

    fn render_initiative_popup(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered()
            .title("Set Initiative")
            .borders(Borders::ALL)
            .bg(NORMAL_ROW_BG);

        let input = Paragraph::new(self.initiative_input.value())
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .alignment(ratatui::layout::Alignment::Center);

        input.render(area, buf);
    }

    fn render_creature_search_popup(&self, area: Rect, buf: &mut Buffer) {
        // Draw the popup background and border
        Block::bordered()
            .title("Creature Search")
            .borders(Borders::ALL)
            .bg(NORMAL_ROW_BG)
            .render(area, buf);

        // Split the popup into input and results areas
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .margin(1)
            .split(area);

        // Render the search input
        let input = Paragraph::new(Text::from(self.creature_search_input.as_str()))
            .block(Block::default().borders(Borders::ALL).title("Search"));
        input.render(chunks[0], buf);

        // Prepare the results as a list of items
        let results: Vec<ListItem> = self
            .creature_search_result
            .iter()
            .map(|item| ListItem::new(item.name.clone()))
            .collect();

        // Set up the selection state
        let mut state = ratatui::widgets::ListState::default();
        state.select(self.creature_search_selected);

        // Render the results list with highlight for the selected row
        let list = List::new(results)
            .block(Block::default().borders(Borders::ALL).title("Results"))
            .highlight_style(SELECTED_STYLE);

        // Render the loading state if applicable
        if self.creature_search_loading {
            // Render a loading spinner or text near the input
            let loading = Paragraph::new("Loading...").block(Block::default());
            loading.render(chunks[0], buf);
        }

        StatefulWidget::render(list, chunks[1], buf, &mut state);
    }

    fn popup_search_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(90)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(90)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    fn popup_initiative_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(30)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(40)]).flex(Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    }

    fn render_header(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Who's Turn Is It?")
            .bold()
            .centered()
            .render(area, buf);
    }

    fn render_footer(area: Rect, buf: &mut Buffer) {
        Paragraph::new(format!(
            "Use {} for new encounter, {} to set initiative, {} and {} to change \
                health, {} and {} to switch between creatures.",
            NEW_ENCOUNTER_KEY,
            SET_INITIATIVE_KEY,
            LOWER_HEALTH_KEY,
            INCREASE_HEALTH_KEY,
            MOVE_DOWN_KEY,
            MOVE_UP_KEY
        ))
        .centered()
        .render(area, buf);
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Order").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(WTII_HEADER_STYLE)
            .bg(NORMAL_ROW_BG);

        let items: Vec<ListItem> = self
            .creature_list
            .items
            .iter()
            .enumerate()
            .map(|(i, creature)| {
                let color = alternate_colors(i);
                ListItem::from(creature).bg(color)
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.creature_list.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let info = if let Some(i) = self.creature_list.state.selected() {
            let initiative_is_set: bool = self.creature_list.items[i].initiative.is_some();
            match self.creature_list.items[i].faction {
                Faction::Npc => {
                    let lines = npc_info(self, i);
                    lines
                        .into_iter()
                        .map(|(k, v, f)| {
                            if f == TextFormatting::Line {
                                format!("{}: {}", k, v)
                            } else {
                                format!("\n==={}===\n{}", k, v)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                Faction::Player => format!(
                    " Initiative: {}\n Name: {}\n HP: {}\n Description: {}",
                    if initiative_is_set {
                        self.creature_list.items[i].initiative.unwrap().to_string()
                    } else {
                        "Not set yet".to_string()
                    },
                    self.creature_list.items[i].name,
                    self.creature_list.items[i].hit_points.to_string(),
                    match &self.creature_list.items[i].desc {
                        Some(desc) => desc.to_string(),
                        None => "".to_string(),
                    },
                ),
            }
        } else {
            "Nothing selected...".to_string()
        };

        let block = Block::new()
            .title(Line::raw("Creature Info").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(WTII_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        Paragraph::new(info)
            .block(block)
            .fg(TEXT_FG_COLOR)
            .wrap(Wrap { trim: false })
            .scroll((self.creature_info_scroll, 0))
            .render(area, buf);
    }
}

fn npc_info(app: &App, i: usize) -> Vec<(String, String, TextFormatting)> {
    let c = &app.creature_list.items[i];
    let mut lines = Vec::new();

    let initiative = if let Some(val) = c.initiative {
        val.to_string()
    } else {
        "Not set yet".to_string()
    };
    lines.push(("Initiative".to_string(), initiative, TextFormatting::Line));
    lines.push(("Name".to_string(), c.name.clone(), TextFormatting::Line));

    let mut hp_str = c.hit_points.to_string();
    if app.increasing_or_decreasing_health {
        hp_str.push_str(&format!(" ({})", app.health_change));
    }
    lines.push(("HP".to_string(), hp_str, TextFormatting::Line));

    if let Some(ac) = c.armor_class {
        lines.push(("AC".to_string(), ac.to_string(), TextFormatting::Line));
    }
    if let Some(val) = c.strength {
        lines.push((
            "Strength".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.strength_save {
        lines.push((
            "Strength Save".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.dexterity {
        lines.push((
            "Dexterity".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.dexterity_save {
        lines.push((
            "Dexterity Save".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.constitution {
        lines.push((
            "Constitution".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.constitution_save {
        lines.push((
            "Constitution Save".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.intelligence {
        lines.push((
            "Intelligence".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.intelligence_save {
        lines.push((
            "Intelligence Save".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.wisdom {
        lines.push(("Wisdom".to_string(), val.to_string(), TextFormatting::Line));
    }
    if let Some(val) = c.wisdom_save {
        lines.push((
            "Wisdom Save".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.charisma {
        lines.push((
            "Charisma".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(val) = c.charisma_save {
        lines.push((
            "Charisma Save".to_string(),
            val.to_string(),
            TextFormatting::Line,
        ));
    }
    if let Some(speed) = &c.speed {
        lines.push((
            "Speed".to_string(),
            format!("{}", speed),
            TextFormatting::NewLine,
        ));
    }
    if let Some(size) = &c.size {
        lines.push(("Size".to_string(), size.clone(), TextFormatting::NewLine));
    }
    if let Some(skills) = &c.skills {
        lines.push((
            "Skills".to_string(),
            skills.to_string(),
            TextFormatting::NewLine,
        ));
    }
    if let Some(val) = &c.damage_vulnerabilities {
        let s = val.to_string();
        if !s.is_empty() {
            lines.push((
                "Damage Vulnerabilities".to_string(),
                s,
                TextFormatting::NewLine,
            ));
        }
    }
    if let Some(val) = &c.damage_resistances {
        let s = val.to_string();
        if !s.is_empty() {
            lines.push(("Damage Resistance".to_string(), s, TextFormatting::NewLine));
        }
    }
    if let Some(val) = &c.damage_immunities {
        let s = val.to_string();
        if !s.is_empty() {
            lines.push(("Damage Immunities".to_string(), s, TextFormatting::NewLine));
        }
    }
    if let Some(val) = &c.condition_immunities {
        let s = val.to_string();
        if !s.is_empty() {
            lines.push((
                "Condition Immunities".to_string(),
                s,
                TextFormatting::NewLine,
            ));
        }
    }
    if let Some(val) = &c.senses {
        let s = val.to_string();
        if !s.is_empty() {
            lines.push(("Senses".to_string(), s, TextFormatting::NewLine));
        }
    }
    if let Some(val) = &c.languages {
        let s = val.to_string();
        if !s.is_empty() {
            lines.push(("Languages".to_string(), s, TextFormatting::NewLine));
        }
    }
    if let Some(val) = &c.challenge_rating {
        let s = val.to_string();
        if !s.is_empty() {
            lines.push(("Challenge Rating".to_string(), s, TextFormatting::NewLine));
        }
    }
    if let Some(actions) = &c.actions {
        lines.push((
            "Actions".to_string(),
            actions
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join("\n--------------------------------\n"),
            TextFormatting::NewLine,
        ));
    }
    if let Some(legendary_actions) = &c.legendary_actions {
        lines.push((
            "Legendary Actions".to_string(),
            legendary_actions
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join("\n--------------------------------\n"),
            TextFormatting::NewLine,
        ));
    }
    if let Some(reactions) = &c.reactions {
        lines.push((
            "Reactions".to_string(),
            reactions
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<_>>()
                .join("\n--------------------------------\n"),
            TextFormatting::NewLine,
        ));
    }
    if let Some(special_abilities) = &c.special_abilities {
        lines.push((
            "Special Abilities".to_string(),
            special_abilities
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join("\n--------------------------------\n"),
            TextFormatting::NewLine,
        ));
    }
    lines
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
