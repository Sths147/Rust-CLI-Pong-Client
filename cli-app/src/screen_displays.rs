use ratatui::widgets::canvas::{Circle, Rectangle};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    prelude::{Color, Constraint, Direction, Layout},
    style::{Modifier, Style, Stylize},
    symbols::{Marker, border},
    text::Line,
    text::Span,
    widgets::{Block, Borders, Paragraph, Widget, canvas::Canvas},
};

use crate::Infos;
use crate::LOGO;
use crate::login::Field;

pub(crate) trait ScreenDisplayer {
    fn display_welcome_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_gamechoice_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_social_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_friends_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_waiting_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_first_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_played_game(&self, area: Rect, buf: &mut Buffer);
    fn display_endgame(&self, area: Rect, buf: &mut Buffer);
    fn display_signup_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_login_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_error_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_addfriends_screen(&self, area: Rect, buf: &mut Buffer);
    fn display_delete_friends_screen(&self, area: Rect, buf: &mut Buffer);
    fn print_demo(&self, area: Rect, buf: &mut Buffer);
}

impl ScreenDisplayer for Infos {
    fn display_first_screen(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(10), Constraint::Fill(1)])
            .split(area);
        self.print_demo(layout[1], buf);
        let instructions = Line::from(vec![
            " Menu: ↑ Sign up ".bold(),
            "↓ Login ".bold(),
            "→  Sign in as guest ".bold(),
            "ESC. Quit ".bold(),
        ]);
        print_block(instructions, layout[0], buf);
    }
    fn display_welcome_screen(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(10), Constraint::Fill(1)])
            .split(area);
        self.print_demo(layout[1], buf);
        let instructions = Line::from(vec![
            " Menu: ↑ Game ".bold(),
            "→ Social Life ".bold(),
            "ESC. Quit ".bold(),
        ]);
        print_block(instructions, layout[0], buf);
    }
    fn display_gamechoice_screen(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(10), Constraint::Fill(1)])
            .split(area);
        self.print_demo(layout[1], buf);
        let instructions = Line::from(vec![
            " Menu: → Online ".bold(),
            "← Back  ".bold(),
            "ESC. Quit ".bold(),
        ]);
        print_block(instructions, layout[0], buf);
    }
    fn display_social_screen(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Max(10), Constraint::Fill(1)])
            .split(area);
        self.print_demo(layout[1], buf);
        let instructions = Line::from(vec![
            " Menu: → Your Friends  ".bold(),
            "← Back  ".bold(),
            "ESC. Quit ".bold(),
        ]);
        print_block(instructions, layout[0], buf);
    }
    fn display_waiting_screen(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::THICK);
        Paragraph::new(Line::from("Searching for opponent".bold()))
            .centered()
            .block(block)
            .render(area, buf);
    }
    fn display_friends_screen(&self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            " Menu: ↑ Add friend ".bold(),
            "↓ Delete friend ".bold(),
            "← Previous ".bold(),
            "→ Next ".bold(),
            "ESC. Back".bold(),
        ]);
        let block = Block::bordered()
            .title(Line::from("Your Friends").bold().centered())
            .title_bottom(instructions.centered())
            .border_set(border::THICK);
        let mut friends_display: Vec<String> = vec![];
        let height: usize = (area.height - 2) as usize;
        let max: usize =
            match (self.friend.index * height + height + 1) >= self.friend.friends_list.len() {
                true => self.friend.friends_list.len(),
                false => (self.friend.index * height) + height + 1,
            };
        let min = match self.friend.index * height < self.friend.friends_list.len() {
            true => self.friend.index * height,
            false => self.friend.friends_list.len(),
        };
        for friend in &self.friend.friends_list[min..max] {
            friends_display.push(friend.clone());
        }
        let lines: Vec<Line> = friends_display
            .iter()
            .map(|friend| Line::from(friend.clone().bold()))
            .collect();
        Paragraph::new(lines)
            .centered()
            .block(block)
            .render(area, buf);
    }
    fn display_played_game(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Max(3)])
            .split(area);
        Canvas::default()
            .block(Block::bordered().title("Pong".bold()))
            .marker(Marker::Braille)
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .paint(|ctx| {
                ctx.draw(&Circle {
                    x: self.game.game_stats.ball_x as f64,
                    y: (100.0 - self.game.game_stats.ball_y) as f64,
                    radius: 0.5,
                    color: Color::Yellow,
                });
                ctx.draw(&Rectangle {
                    x: 1.5,
                    y: (95.0 - self.game.game_stats.left_y) as f64,
                    width: 2.0,
                    height: 10.0,
                    color: Color::Green,
                });
                ctx.draw(&Rectangle {
                    x: 97.0,
                    y: (95.0 - self.game.game_stats.right_y) as f64,
                    width: 2.0,
                    height: 10.0,
                    color: Color::Green,
                });
            })
            .render(layout[0], buf);
        let line = Line::from(vec![
            format!("You: {}", self.game.game_stats.player1_score).bold(),
            "    |     ".bold(),
            format!(
                "{}: {}",
                self.game.opponent_name, self.game.game_stats.player2_score
            )
            .bold(),
        ]);
        Paragraph::new(line)
            .block(
                Block::bordered()
                    .border_set(border::THICK)
                    .title("Score".bold()),
            )
            .centered()
            .render(layout[1], buf);
    }
    fn display_endgame(&self, area: Rect, buf: &mut Buffer) {
        let sentence: &str = match self.game.game_stats.winner {
            true => "You Win :)",
            false => "You lose :(",
        };
        let block = Block::bordered().border_set(border::THICK);
        let spanlist: Vec<Span> = vec![sentence.bold(), " Press Enter to Continue".bold()];
        Paragraph::new(Line::from(spanlist))
            .centered()
            .block(block)
            .render(area, buf);
    }
    fn display_signup_screen(&self, area: Rect, buf: &mut Buffer) {
        let mail = format!(
            "{}{}",
            self.authent.borrow().get_email(),
            if self.authent.borrow().blinks(Field::Mail) {
                "|"
            } else {
                ""
            }
        );
        let username = format!(
            "{}{}",
            self.authent.borrow().get_username(),
            if self.authent.borrow().blinks(Field::Username) {
                "|"
            } else {
                ""
            }
        );
        let mut password = String::new();
        for _ in 0..self.authent.borrow().get_password().len() {
            password.push('*');
        }
        if self.authent.borrow().blinks(Field::Password) {
            password.push('|')
        }
        let content = vec![
            Line::from(Span::styled(
                "Create an account",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Email:     ", Style::default().fg(Color::Gray)),
                Span::raw(mail),
            ]),
            Line::from(vec![
                Span::styled("Username:  ", Style::default().fg(Color::Gray)),
                Span::raw(username),
            ]),
            Line::from(vec![
                Span::styled("Password:  ", Style::default().fg(Color::Gray)),
                Span::raw(password),
            ]),
        ];
        Paragraph::new(content)
            .block(Block::default().title("Signup").borders(Borders::ALL))
            .alignment(Alignment::Left)
            .render(area, buf);
    }
    fn display_login_screen(&self, area: Rect, buf: &mut Buffer) {
        let mail = format!(
            "{}{}",
            self.authent.borrow().get_email(),
            if self.authent.borrow().blinks(Field::Mail) {
                "|"
            } else {
                ""
            }
        );
        let mut password = String::new();
        for _ in 0..self.authent.borrow().get_password().len() {
            password.push('*');
        }
        if self.authent.borrow().blinks(Field::Password) {
            password.push('|')
        }
        let totp = format!(
            "{}{}",
            self.authent.borrow().get_totp(),
            if self.authent.borrow().blinks(Field::Totp) {
                "|"
            } else {
                ""
            }
        );
        let content = vec![
            Line::from(Span::styled(
                "Login as user",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Email:     ", Style::default().fg(Color::Gray)),
                Span::raw(mail),
            ]),
            Line::from(vec![
                Span::styled("Password:  ", Style::default().fg(Color::Gray)),
                Span::raw(password),
            ]),
            Line::from(vec![
                Span::styled("2FA Code:  ", Style::default().fg(Color::Gray)),
                Span::raw(totp),
            ]),
        ];
        Paragraph::new(content)
            .block(
                Block::default()
                    .title("Signup".bold())
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .render(area, buf);
    }
    fn display_error_screen(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::bordered().border_set(border::THICK);
        let linelist: Vec<Line> = vec![
            ("Error: ".bold() + self.error.as_str().bold()),
            Line::from("Press any key to continue".bold()),
        ];
        Paragraph::new(linelist)
            .centered()
            .block(block)
            .render(area, buf);
    }
    fn display_addfriends_screen(&self, area: Rect, buf: &mut Buffer) {
        let friend = format!(
            "{}{}",
            self.friend.friend_tmp,
            if self.friend.blink { "|" } else { "" }
        );
        let content = vec![
            Line::from(Span::styled(
                "Add a friend",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Friend:     ", Style::default().fg(Color::Gray)),
                Span::raw(friend),
            ]),
        ];
        Paragraph::new(content)
            .block(
                Block::default()
                    .title("Add Friend".bold())
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .render(area, buf);
    }
    fn display_delete_friends_screen(&self, area: Rect, buf: &mut Buffer) {
        let friend = format!(
            "{}{}",
            self.friend.friend_tmp,
            if self.friend.blink { "|" } else { "" }
        );
        let content = vec![
            Line::from(Span::styled(
                "Delete a friend",
                Style::default().add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled("Friend:     ", Style::default().fg(Color::Gray)),
                Span::raw(friend),
            ]),
        ];
        Paragraph::new(content)
            .block(
                Block::default()
                    .title("Delete friend".bold())
                    .borders(Borders::ALL),
            )
            .alignment(Alignment::Left)
            .render(area, buf);
    }
    fn print_demo(&self, area: Rect, buf: &mut Buffer) {
        Canvas::default()
            .block(Block::bordered())
            .marker(Marker::Braille)
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 100.0])
            .paint(|ctx| {
                ctx.draw(&Circle {
                    x: self.demo.ball_x,
                    y: self.demo.ball_y,
                    radius: 0.5,
                    color: Color::Yellow,
                });
                ctx.draw(&Rectangle {
                    x: 1.5,
                    y: self.demo.paddle_left_y,
                    width: 2.0,
                    height: 10.0,
                    color: Color::Green,
                });
                ctx.draw(&Rectangle {
                    x: 95.0,
                    y: self.demo.paddle_right_y,
                    width: 2.0,
                    height: 10.0,
                    color: Color::Green,
                });
            })
            .render(area, buf);
    }
}

fn print_block(instructions: Line, area: Rect, buf: &mut Buffer) {
    let block = Block::bordered()
        .title_bottom(instructions.centered())
        .border_set(border::THICK);
    Paragraph::new(LOGO)
        .centered()
        .block(block)
        .render(area, buf);
}
