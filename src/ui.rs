use crate::creature::Faction;
use crate::creature::{CreatureItem, CreatureList, Status};
use color_eyre::Result;
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
use tui_input::Input;

const WTII_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;

pub struct App {
    creature_list: CreatureList,
    should_exit: bool,
    show_creature_search_popup: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            show_creature_search_popup: false,
            creature_list: CreatureList::default(),
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
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => match self.show_creature_search_popup {
                true => self.show_creature_search_popup = false,
                false => self.should_exit = true,
            },
            KeyCode::Char('u') => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('h') | KeyCode::Left => self.lower_health(),
            KeyCode::Char('l') | KeyCode::Right => self.increase_health(),
            KeyCode::Char('i') => self.show_creature_search_popup = true,
            KeyCode::Char('c') => self.insert_new(),
            KeyCode::Char('d') => self.delete_creature(),
            KeyCode::Char('e') => self.new_encounter(),
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
        self.creature_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.creature_list.state.select_previous();
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
        let creature =
            CreatureItem::new_npc("Proto Drake", Some("Big dragon"), 11, Some(44), Some(18));
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
            let area = App::popup_area(main_area);
            App::clear_area(area, buf);
            App::render_creature_search_popup(&self, area, buf);
        }
    }
}

/// Rendering logic for the app
impl App {
    fn clear_area(area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);
    }
    fn render_creature_search_popup(&self, area: Rect, buf: &mut Buffer) {
        Self::popup_area(area);
        Block::bordered()
            .title("Creature Search")
            .borders(Borders::ALL)
            .bg(NORMAL_ROW_BG)
            .render(area, buf);
    }

    fn popup_area(area: Rect) -> Rect {
        let vertical = Layout::vertical([Constraint::Percentage(90)]).flex(Flex::Center);
        let horizontal = Layout::horizontal([Constraint::Percentage(90)]).flex(Flex::Center);
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
        Paragraph::new("Use e for new encounter, i to insert new creature, h and l to change health and n for next creature.")
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
            match self.creature_list.items[i].faction {
                Faction::Npc => format!(
                    " Initiative: {}\n Name: {}\n HP: {}\n AC: {}\n Description: {}",
                    if self.creature_list.items[i].initiative.is_some() {
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
                    if self.creature_list.items[i].initiative.is_some() {
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
