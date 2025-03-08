use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{
        palette::tailwind::{BLUE, GREEN, RED, SLATE},
        Color, Modifier, Style, Stylize,
    },
    symbols,
    text::Line,
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget, Wrap,
    },
    DefaultTerminal,
};

const WTII_HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(BLUE.c800);
const NORMAL_ROW_BG: Color = SLATE.c950;
const ALT_ROW_BG_COLOR: Color = SLATE.c900;
const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
const TEXT_FG_COLOR: Color = SLATE.c200;
const COMPLETED_TEXT_FG_COLOR: Color = GREEN.c500;
const DEAD_TEXT_FG_COLOR: Color = RED.c500;

pub struct App {
    should_exit: bool,
    creature_list: CreatureList,
}

struct CreatureList {
    items: Vec<CreatureItem>,
    state: ListState,
}

#[derive(Debug)]
struct CreatureItem {
    name: String,
    status: Status,
    initiative: i64,
    hp: i64,
    ac: i64,
    description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Status {
    Alive,
    Dead,
}

impl Default for App {
    fn default() -> Self {
        Self {
            should_exit: false,
            creature_list: CreatureList::from_iter([
                // Status, Name, Initiative, HP, AC, Description
                (Status::Alive, "Samson", 1, 2, 3, ""),
                (
                    Status::Alive,
                    "Red Proto Drake",
                    4,
                    5,
                    6,
                    "A big ass dragon",
                ),
            ]),
        }
    }
}

impl FromIterator<(Status, &'static str, i64, i64, i64, &'static str)> for CreatureList {
    fn from_iter<I: IntoIterator<Item = (Status, &'static str, i64, i64, i64, &'static str)>>(
        iter: I,
    ) -> Self {
        let items = iter
            .into_iter()
            .map(|(status, name, initiative, hp, ac, description)| {
                CreatureItem::new(status, name, initiative, hp, ac, description)
            })
            .collect();
        let state = ListState::default();
        Self { items, state }
    }
}

impl CreatureItem {
    fn new(
        status: Status,
        name: &str,
        initiative: i64,
        hp: i64,
        ac: i64,
        description: &str,
    ) -> Self {
        let new_initiative: i64;
        if initiative < 0 {
            new_initiative = rand::random_range(1..21);
        } else {
            new_initiative = initiative;
        }
        Self {
            status,
            name: name.to_string(),
            initiative: new_initiative,
            hp,
            ac,
            description: description.to_string(),
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
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Char('u') => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('h') | KeyCode::Left => self.lower_health(),
            KeyCode::Char('l') | KeyCode::Right => self.increase_health(),
            KeyCode::Char('i') => self.insert_new(),
            _ => {}
        }
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

    fn lower_health(&mut self) {
        if let Some(i) = self.creature_list.state.selected() {
            if self.creature_list.items[i].hp > i64::MIN {
                self.creature_list.items[i].hp -= 1;
            }
            if self.creature_list.items[i].hp <= 0 {
                self.creature_list.items[i].status = Status::Dead;
            }
        }
    }

    fn increase_health(&mut self) {
        if let Some(i) = self.creature_list.state.selected() {
            if self.creature_list.items[i].hp < i64::MAX {
                self.creature_list.items[i].hp += 1;
            }
            if self.creature_list.items[i].hp > 0 {
                self.creature_list.items[i].status = Status::Alive;
            }
        }
    }

    fn insert_new(&mut self) {
        let creature = CreatureItem::new(Status::Alive, "Borbur", -1, 30, 18, "A big ass dude");
        self.creature_list.items.push(creature);
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
    }
}

/// Rendering logic for the app
impl App {
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

        // Iterate through all elements in the `items` and stylize them.
        let items: Vec<ListItem> = self
            .creature_list
            .items
            .iter()
            .enumerate()
            .map(|(i, todo_item)| {
                let color = alternate_colors(i);
                ListItem::from(todo_item).bg(color)
            })
            .collect();

        // Create a List from all list items and highlight the currently selected one
        let list = List::new(items)
            .block(block)
            .highlight_style(SELECTED_STYLE)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        // We need to disambiguate this trait method as both `Widget` and `StatefulWidget` share the
        // same method name `render`.
        StatefulWidget::render(list, area, buf, &mut self.creature_list.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        // We get the info depending on the item's state.
        let info = if let Some(i) = self.creature_list.state.selected() {
            match self.creature_list.items[i].status {
                _ => format!(
                    " Initiative: {}\n Name: {}\n HP: {}\n AC: {}\n Description: {}",
                    self.creature_list.items[i].initiative,
                    self.creature_list.items[i].name,
                    self.creature_list.items[i].hp,
                    self.creature_list.items[i].ac,
                    self.creature_list.items[i].description,
                ),
            }
        } else {
            "Nothing selected...".to_string()
        };

        // We show the list item's info under the list in this paragraph
        let block = Block::new()
            .title(Line::raw("Creature Info").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(WTII_HEADER_STYLE)
            .bg(NORMAL_ROW_BG)
            .padding(Padding::horizontal(1));

        // We can now render the item info
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

impl From<&CreatureItem> for ListItem<'_> {
    fn from(value: &CreatureItem) -> Self {
        let line = match value.status {
            Status::Alive => Line::styled(format!(" ✓ {}", value.name), COMPLETED_TEXT_FG_COLOR),
            Status::Dead => Line::styled(format!(" X {}", value.name), DEAD_TEXT_FG_COLOR),
        };
        ListItem::new(line)
    }
}
