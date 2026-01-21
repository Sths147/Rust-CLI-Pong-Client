use anyhow::{Result, anyhow};
use crossterm::event::{self, Event, KeyCode, poll};
use ratatui::{DefaultTerminal, Frame, buffer::Buffer, layout::Rect, widgets::Widget};
use reqwest::header::HeaderMap;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use tokio::time::Duration;
use crate::CurrentScreen;
use crate::context::Context;
use crate::friends::Friends;
use crate::game::Game;
use crate::game_demo::Demo;
use crate::infos_events::EventHandler;
use crate::login::Auth;
use crate::screen_displays::ScreenDisplayer;
use crate::utils::should_exit;

#[derive(Default)]
pub(crate)struct Infos {
    pub(crate) context: Rc<Context>,
    pub(crate) authent: Rc<RefCell<Auth>>,
    pub(crate) friend: Friends,
    pub(crate) screen: Rc<Cell<CurrentScreen>>,
    pub(crate) game: Game,
    pub(crate) demo: Demo,
    pub(crate) post_error_screen: CurrentScreen,
    pub(crate) error: String,
    pub(crate) exit: bool,
}

impl Infos {
    pub(crate)fn new(
        context: Rc<Context>,
        auth: Rc<RefCell<Auth>>,
        screen: Rc<Cell<CurrentScreen>>,
        friends: Friends,
    ) -> Infos {
        Infos {
            context,
            authent: auth,
            screen,
            friend: friends,
            ..Default::default()
        }
    }
    pub(crate) async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            if self.screen.get() == CurrentScreen::FriendsDisplay {
                self.friend.update_friends_index(terminal).await?;
            }
            if let Err(e) = terminal.draw(|frame| self.draw(frame)) {
                self.error(e.to_string());
            }
            match self.screen.get() {
                CurrentScreen::FirstScreen
                | CurrentScreen::GameChoice
                | CurrentScreen::SocialLife
                | CurrentScreen::Welcome => {
                    self.demo.update();
                    if event::poll(Duration::from_millis(16))?
                        && let Err(e) = self.handle_events().await
                    {
                        self.error(e.to_string());
                    }
                }
                _ => {
                    if let Err(e) = self.handle_events().await {
                        self.error(e.to_string());
                    }
                }
            }
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
    async fn handle_events(&mut self) -> Result<()> {
        match self.screen.get() {
            CurrentScreen::FirstScreen => {
                if let Err(e) = self.handle_first_events().await {
                    self.authent.borrow_mut().clear();
                    return Err(e);
                }
            }
            CurrentScreen::SignUp => {
                if let Err(e) = self.handle_signup_events().await {
                    self.authent.borrow_mut().clear();
                    return Err(e);
                }
            }
            CurrentScreen::Login => {
                if let Err(e) = self.handle_login_events().await {
                    self.authent.borrow_mut().clear();
                    return Err(e);
                }
            }
            CurrentScreen::Welcome => self.handle_welcome_events()?,
            CurrentScreen::GameChoice => self.handle_gamechoice_events()?,
            CurrentScreen::SocialLife => self.handle_social_events().await?,
            CurrentScreen::FriendsDisplay => self.handle_friends_events()?,
            CurrentScreen::StartGame => self.launch_game().await?,
            CurrentScreen::EndGame => self.handle_endgame()?,
            CurrentScreen::CreateGame => self.create_game("online").await?,
            CurrentScreen::PlayGame => self.handle_game_events().await?,
            CurrentScreen::ErrorScreen => self.handle_errors().await?,
            CurrentScreen::AddFriend => self.friend.add_friend().await?,
            CurrentScreen::DeleteFriend => self.friend.delete_friend().await?,
        }
        Ok(())
    }
    pub(crate) fn error(&mut self, error: String) {
        self.post_error_screen = self.screen.get();
        self.error = error;
        self.screen.set(CurrentScreen::ErrorScreen);
    }
    async fn handle_errors(&mut self) -> Result<()> {
        loop {
            let event = event::read()?;
            if let Event::Key(_) = event {
                break;
            }
        }
        self.screen.set(self.post_error_screen);
        Ok(())
    }
    pub(crate) async fn create_game(&mut self, mode: &str) -> Result<()> {
        send_post_game_request(self, mode).await?;
        loop {
            match poll(Duration::from_millis(16)) {
                Ok(true) => {
                    if !self
                        .authent
                        .borrow_mut()
                        .receiver
                        .as_mut()
                        .expect("empty receiver")
                        .is_empty()
                    {
                        break;
                    }
                    let event = event::read()?;
                    if let Ok(true) = should_exit(&event) {
                        self.send_remove_from_queue_request().await?;
                        self.screen.set(CurrentScreen::GameChoice);
                        return Ok(());
                    }
                }
                Ok(false) => {
                    if !self
                        .authent
                        .borrow_mut()
                        .receiver
                        .as_mut()
                        .expect("empty receiver")
                        .is_empty()
                    {
                        break;
                    }
                }
                _ => return Err(anyhow!("error in poll".to_string())),
            };
        }
        let response = self
            .authent
            .borrow_mut()
            .receiver
            .as_mut()
            .expect("empty receiver")
            .try_recv()?;
        let game = Game::new(self, response).await?;
        self.game = game;
        self.screen.set(crate::CurrentScreen::StartGame);
        Ok(())
    }
    pub(crate) async fn launch_game(&mut self) -> Result<()> {
        self.game.start_game().await?;
        self.screen.set(crate::CurrentScreen::PlayGame);
        Ok(())
    }
    pub(crate) async fn handle_game_events(&mut self) -> Result<()> {
        let mut state_receiver = match self.game.receiver.clone() {
            Some(receiver) => receiver,
            _ => {
                return Err(anyhow!("State receiver is empty"));
            }
        };
        if let Some(checker) = &mut self.game.game_checker
            && let Ok(true) = checker.has_changed()
        {
            self.screen.set(crate::CurrentScreen::GameChoice);
        };
        if let Some(sender) = &self.game.game_sender {
            state_receiver.changed().await?;
            let (bytes, text) = state_receiver.borrow_and_update().clone();
            match (bytes, text) {
                (Some(bytes), _none) => {
                    self.game.decode_and_update(bytes)?;
                }
                (_none, Some(text)) => {
                    self.game.end_game(text, sender.clone()).await?;
                    self.screen.set(crate::CurrentScreen::EndGame);
                }
                _ => {}
            };
        }
        Ok(())
    }
    pub(crate) fn handle_endgame(&mut self) -> Result<()> {
        if poll(Duration::from_millis(16))? {
            let event = event::read()?;
            if should_exit(&event)? {
                self.screen.set(crate::CurrentScreen::GameChoice);
            } else if let Event::Key(keyevent) = event
                && keyevent.code == KeyCode::Enter
            {
                self.screen.set(crate::CurrentScreen::GameChoice);
            }
        }
        Ok(())
    }
    async fn send_remove_from_queue_request(&self) -> Result<()> {
        let mut map = HashMap::new();
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/json".parse()?);
        let id: &str = &self.authent.borrow().id.to_string();
        map.insert("id", id);
        let url = format!("https://{}/api/chat/removeQueue", self.context.location);
        self.context
            .client
            .delete(url)
            .headers(headers)
            .json(&map)
            .send()
            .await?;
        Ok(())
    }
}

impl Widget for &Infos {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.screen.get() {
            CurrentScreen::FirstScreen => self.display_first_screen(area, buf),
            CurrentScreen::SignUp => self.display_signup_screen(area, buf),
            CurrentScreen::Login => self.display_login_screen(area, buf),
            CurrentScreen::Welcome => self.display_welcome_screen(area, buf),
            CurrentScreen::GameChoice => self.display_gamechoice_screen(area, buf),
            CurrentScreen::SocialLife => self.display_social_screen(area, buf),
            CurrentScreen::FriendsDisplay => self.display_friends_screen(area, buf),
            CurrentScreen::StartGame => {}
            CurrentScreen::EndGame => self.display_endgame(area, buf),
            CurrentScreen::CreateGame => self.display_waiting_screen(area, buf),
            CurrentScreen::PlayGame => self.display_played_game(area, buf),
            CurrentScreen::ErrorScreen => self.display_error_screen(area, buf),
            CurrentScreen::AddFriend => self.display_addfriends_screen(area, buf),
            CurrentScreen::DeleteFriend => self.display_delete_friends_screen(area, buf),
        }
    }
}

async fn send_post_game_request(game_main: &Infos, mode: &str) -> Result<()> {
    let mut map = HashMap::new();
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);
    map.insert("mode", mode);
    let id: &str = &game_main.authent.borrow().id.to_string();
    map.insert("playerName", id);
    let mut url = game_main.context.location.clone();
    url = format!("https://{url}/api/create-game");
    game_main
        .context
        .client
        .post(url)
        .headers(headers)
        .json(&map)
        .send()
        .await?;
    Ok(())
}
