use std::rc::Rc;
use std::cell::{Cell, RefCell};
use anyhow::Result;
use crossterm::event::{self, Event};
use tokio::time::Duration;
use ratatui::{
  buffer::Buffer,
  layout::Rect,
  widgets::Widget,
  DefaultTerminal, Frame,
};
use crate::CurrentScreen;
use crate::context::Context;
use crate::infos_events::EventHandler;
use crate::screen_displays::ScreenDisplayer;
use crate::friends::Friends;
use crate::game_demo::Demo;
use crate::game::{Game, Gameplay};
use crate::login::Auth;

#[derive(Default)]
pub struct Infos {
  pub context: Rc<Context>,
  pub authent: Rc<RefCell<Auth>>,
  pub friend: Friends,
  pub screen: Rc<Cell<CurrentScreen>>,
  pub game: Game,
  pub demo: Demo,
  pub post_error_screen: CurrentScreen,
  pub error: String,
  pub exit: bool,
}

impl Infos {
  pub fn new(context: Rc<Context>, auth: Rc<RefCell<Auth>>, 
      screen: Rc<Cell<CurrentScreen>>, friends: Friends) -> Infos {
    Infos {
      context,
      authent: auth,
      screen,
      friend: friends,
      ..Default::default()
    }
  }
  pub async fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
    while !self.exit {
        if self.screen.get() == CurrentScreen::FriendsDisplay {
          self.friend.update_friends_index(terminal).await?;
        }
        if let Err(e) = terminal.draw(|frame| self.draw(frame)) {
          self.error(e.to_string());
        }
        match self.screen.get() {
          CurrentScreen::FirstScreen | CurrentScreen::GameChoice | 
            CurrentScreen::SocialLife | CurrentScreen::Welcome => {
              self.demo.update();
              if event::poll(Duration::from_millis(16))?
                && let Err(e) = self.handle_events().await {
                  self.error(e.to_string());
              }
            },
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
      CurrentScreen::FirstScreen => {if let Err(e) = self.handle_first_events().await {
          self.authent.borrow_mut().clear();
          return Err(e)
        }
      },
      CurrentScreen::SignUp => {if let Err(e) = self.handle_signup_events().await {
          self.authent.borrow_mut().clear();
          return Err(e)
        }
      },
      CurrentScreen::Login => {if let Err(e) = self.handle_login_events().await {
          self.authent.borrow_mut().clear();
          return Err(e)
        }
      },
      CurrentScreen::Welcome => {self.handle_welcome_events()?},
      CurrentScreen::GameChoice => {self.handle_gamechoice_events()?},
      CurrentScreen::SocialLife => {self.handle_social_events().await?},
      CurrentScreen::FriendsDisplay => {self.handle_friends_events()?},
      CurrentScreen::StartGame => {self.launch_game().await?},
      CurrentScreen::EndGame => {self.handle_endgame()?},
      CurrentScreen::CreateGame => {self.create_game("online").await?},
      CurrentScreen::PlayGame => {self.handle_game_events().await?},
      CurrentScreen::ErrorScreen => {self.handle_errors().await?},
      CurrentScreen::AddFriend => {self.friend.add_friend().await?},
      CurrentScreen::DeleteFriend => {self.friend.delete_friend().await?},
    }
  Ok(())
  }
  pub fn get_context(&self) -> &Context {
    &self.context
  }
  pub fn error(&mut self, error: String) {
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
}

impl Widget for &Infos {
  fn render(self, area: Rect, buf: &mut Buffer) {
    match self.screen.get() {
      CurrentScreen::FirstScreen => {self.display_first_screen(area, buf)},
      CurrentScreen::SignUp => {self.display_signup_screen(area, buf)},
      CurrentScreen::Login => {self.display_login_screen(area, buf)},
      CurrentScreen::Welcome => {self.display_welcome_screen(area, buf)}, 
      CurrentScreen::GameChoice => {self.display_gamechoice_screen(area, buf)}, 
      CurrentScreen::SocialLife => {self.display_social_screen(area, buf)}, 
      CurrentScreen::FriendsDisplay => {self.display_friends_screen(area, buf)},
      CurrentScreen::StartGame => {},
      CurrentScreen::EndGame => {self.display_endgame(area, buf)},
      CurrentScreen::CreateGame => {self.display_waiting_screen(area, buf)},
      CurrentScreen::PlayGame => {self.display_played_game(area, buf)},
      CurrentScreen::ErrorScreen => {self.display_error_screen(area, buf)},
      CurrentScreen::AddFriend => {self.display_addfriends_screen(area, buf)},
      CurrentScreen::DeleteFriend => {self.display_delete_friends_screen(area, buf)},
    }
  }
}