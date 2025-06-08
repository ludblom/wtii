use crate::api::{search_for_creature, MonsterSearch};
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
const LOWER_HEALTH_KEY: char = 'h';
const INCREASE_HEALTH_KEY: char = 'l';
const SEARCH_FOR_NEW_CREATURE_KEY: char = 's';
const INSERT_NEW_PLAYER_KEY: char = 'c';
const DELETE_CREATURE_KEY: char = 'd';

pub struct App {
    creature_list: CreatureList,
    should_exit: bool,
    show_creature_search_popup: bool,
    show_initiative_popup: bool,
    initiative_input: Input,
    creature_search_input: String,
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
        }
    }
}

impl App {
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.should_exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        if self.show_creature_search_popup {
            self.handle_creature_search_input(&key);
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
            KeyCode::Char(MOVE_UP_KEY) | KeyCode::Up => self.select_previous(),
            KeyCode::Char(LOWER_HEALTH_KEY) | KeyCode::Left => self.lower_health(),
            KeyCode::Char(INCREASE_HEALTH_KEY) | KeyCode::Right => self.increase_health(),
            KeyCode::Char(SEARCH_FOR_NEW_CREATURE_KEY) => self.show_creature_search_popup = true,
            KeyCode::Char(INSERT_NEW_PLAYER_KEY) => self.insert_new(),
            KeyCode::Char(DELETE_CREATURE_KEY) => self.delete_creature(),
            KeyCode::Char(NEW_ENCOUNTER_KEY) => self.new_encounter(),
            KeyCode::Char(SET_INITIATIVE_KEY) => {
                if self.creature_list.state.selected().is_some() {
                    self.show_initiative_popup = true;
                }
            }
            _ => {}
        }
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

    fn handle_creature_search_input(&mut self, key: &KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.show_creature_search_popup = false;
            }
            KeyCode::Char(c) => {
                self.creature_search_input.push(c);
            }
            KeyCode::Backspace => {
                self.creature_search_input.pop();
            }
            KeyCode::Enter => {
                let input = self.creature_search_input.clone();
                let api = MonsterSearch;

                tokio::spawn(async move {
                    match search_for_creature(&api, &input).await {
                        // TODO: Take the result and present it
                        Ok(results) => {
                            println!("Found {} matches!", results.len());
                        }
                        Err(e) => {
                            println!("Nothing found!!! Error: {}", e);
                        }
                    }
                });
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
            if let Some(hp) = self.creature_list.items[i].hit_points.as_mut() {
                if *hp > 0 {
                    *hp -= 1;
                }
                if *hp == 0 {
                    self.creature_list.items[i].status = Status::Dead;
                }
            }
        }
    }

    fn increase_health(&mut self) {
        if let Some(i) = self.creature_list.state.selected() {
            if let Some(hp) = self.creature_list.items[i].hit_points.as_mut() {
                if *hp < u64::MAX {
                    *hp += 1;
                }
                if *hp > 0 {
                    self.creature_list.items[i].status = Status::Alive;
                }
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
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

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
        Block::bordered()
            .title("Creature Search")
            .borders(Borders::ALL)
            .bg(NORMAL_ROW_BG)
            .render(area, buf);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .margin(1)
            .split(area);
        let input = Paragraph::new(Text::from(self.creature_search_input.as_str()))
            .block(Block::default().borders(Borders::ALL).title("Search"));

        input.render(chunks[0], buf);
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
                Faction::Npc => format!(
                    " Initiative: {}\n Name: {}\n HP: {}\n AC: {}\n Description: {}",
                    if initiative_is_set {
                        self.creature_list.items[i].initiative.unwrap().to_string()
                    } else {
                        "Not set yet".to_string()
                    },
                    self.creature_list.items[i].name,
                    self.creature_list.items[i].hit_points.unwrap(),
                    self.creature_list.items[i].armor_class.unwrap(),
                    match &self.creature_list.items[i].desc {
                        Some(desc) => desc.to_string(),
                        None => "".to_string(),
                    },
                ),
                Faction::Player => format!(
                    " Initiative: {}\n Name: {}\n HP: {}\n Description: {}",
                    if initiative_is_set {
                        self.creature_list.items[i].initiative.unwrap().to_string()
                    } else {
                        "Not set yet".to_string()
                    },
                    self.creature_list.items[i].name,
                    match &self.creature_list.items[i].hit_points {
                        Some(hit_points) => hit_points.to_string(),
                        None => "".to_string(),
                    },
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
            .render(area, buf);
    }
}

const fn alternate_colors(i: usize) -> Color {
    if i % 2 == 0 {
        NORMAL_ROW_BG
    } else {
        ALT_ROW_BG_COLOR
    }
}
