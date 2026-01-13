use std::time::Duration;
use anyhow::Result;
use crossterm::event::{self, poll, Event, KeyCode, KeyEventKind};
use crate::login::Field;
use crate::CurrentScreen;
use crate::utils::should_exit;
use crate::infos::Infos;

pub trait EventHandler {
    fn handle_welcome_events(&mut self) -> Result<()>;
    fn handle_gamechoice_events(&mut self) -> Result<()>;
    fn handle_friends_events(&mut self) -> Result<()>;
    async fn handle_social_events(&mut self) -> Result<()>;
    async fn handle_first_events(&mut self) -> Result<()>;
    async fn handle_signup_events(&mut self) -> Result<()>;
    async fn handle_login_events(&mut self) -> Result<()>;
}

impl EventHandler for Infos {
  fn handle_welcome_events(&mut self) -> Result<()> {
    let event = event::read()?;
    if should_exit(&event)? == true {
      self.exit = true;
    }
    else if let Event::Key(key_event) = event {
      if key_event.kind == KeyEventKind::Press {
        match key_event.code {
            KeyCode::Up => {self.screen.set(CurrentScreen::GameChoice);},
            KeyCode::Right => {self.screen.set(CurrentScreen::SocialLife);},
            _ => {},
        }
      }
    }
    Ok(())
  }
  fn handle_gamechoice_events(&mut self) -> Result<()> {
    let event = event::read()?;
    if should_exit(&event)? == true {
      self.exit = true;
    }
    else if let Event::Key(key_event) = event {
      if key_event.kind == KeyEventKind::Press {
          match key_event.code {
              KeyCode::Right => {self.screen.set(CurrentScreen::CreateGame);},
              KeyCode::Left => {self.screen.set(CurrentScreen::Welcome);},
              _ => {},
          }
      }
    }
    Ok(())
  }
  async fn handle_first_events(&mut self) -> Result<()> {
    let event = event::read()?;
    if should_exit(&event)? == true {
      self.exit = true;
    }
    else if let Event::Key(key_event) = event {
      if key_event.kind == KeyEventKind::Press {
          match key_event.code {
              KeyCode::Up => {self.screen.set(CurrentScreen::SignUp);},
              KeyCode::Down => {self.screen.set(CurrentScreen::Login);},
              KeyCode::Right => {
                self.authent.borrow_mut().create_guest_session().await?;
                self.screen.set(CurrentScreen::Welcome);
              },
              _ => {},
          }
      }
    }
    Ok(())
  }
  async fn handle_social_events(&mut self) -> Result<()> {
    self.friend.borrow_mut().get_indexed_friends().await?;
    let event = event::read()?;
    if should_exit(&event)? == true {
        self.exit = true;
    }
    else if let Event::Key(key_event) = event {
      match key_event.code {
        KeyCode::Right => {
          self.screen.set(CurrentScreen::FriendsDisplay)
        },
        KeyCode::Left => {self.screen.set(CurrentScreen::Welcome)},
        _ => {},
      }
    }
    Ok(()) 
  }
  async fn handle_signup_events(&mut self) -> Result<()> {
      if poll(Duration::from_millis(500))? == true {
        let event = event::read()?;
        if should_exit(&event)? {
          self.authent.borrow_mut().clear();
          self.screen.set(CurrentScreen::FirstScreen);
        } else if let Event::Key(eventkey) = event {
          match eventkey.code {
            KeyCode::Up => {self.authent.borrow_mut().up_field_signup()},
            KeyCode::Down => {self.authent.borrow_mut().down_field_signup()},
            KeyCode::Char(c) => {self.authent.borrow_mut().add(c)},
            KeyCode::Backspace => {self.authent.borrow_mut().pop()},
            KeyCode::Tab => {self.authent.borrow_mut().down_field_signup()}
            KeyCode::Enter => {if *self.authent.borrow_mut().get_field() == Field::Password {
              self.authent.borrow_mut().signup().await?;
              self.screen.set(CurrentScreen::Welcome);
            } else {self.authent.borrow_mut().down_field_signup()}} 
            _ => {},
          }
        }
      }
      self.authent.borrow_mut().tick();
      Ok(())
  }
  async fn handle_login_events(&mut self) -> Result<()> {
      if poll(Duration::from_millis(500))? == true {
        let event = event::read()?;
        if should_exit(&event)? {
          self.authent.borrow_mut().clear();
          self.screen.set(CurrentScreen::FirstScreen);
        } else if let Event::Key(eventkey) = event {
          match eventkey.code {
            KeyCode::Up => {self.authent.borrow_mut().up_field_login()},
            KeyCode::Down => {self.authent.borrow_mut().down_field_login()},
            KeyCode::Char(c) => {self.authent.borrow_mut().add(c)},
            KeyCode::Backspace => {self.authent.borrow_mut().pop();},
            KeyCode::Tab => {self.authent.borrow_mut().down_field_login()},
            KeyCode::Enter => {if *self.authent.borrow_mut().get_field() == Field::Totp {
                self.authent.borrow_mut().login().await?;
                self.screen.set(CurrentScreen::Welcome);
              } else {
                self.authent.borrow_mut().down_field_login()
              }
            }  
            _ => {},
          }
        }
      }
      self.authent.borrow_mut().tick();
      Ok(())
  }
  fn handle_friends_events(&mut self) -> Result<()> {
      let event = event::read()?;
      if should_exit(&event)? == true {
        self.screen.set(CurrentScreen::SocialLife)     
      }
      else if let Event::Key(key_event) = event {
          match key_event.code {
          KeyCode::Up => {
            self.screen.set(CurrentScreen::AddFriend)
          },
          KeyCode::Down => {
            self.screen.set(CurrentScreen::DeleteFriend)
          },
          KeyCode::Right => {if self.friend.borrow().index < self.friend.borrow().index_max {self.friend.borrow_mut().index += 1}},
          KeyCode::Left => {if self.friend.borrow().index > usize::MIN {self.friend.borrow_mut().index -= 1}},
          _ => {},
          }
      }
      else if let Event::Resize(_, _) = event {
        self.friend.borrow_mut().index = 0;
      }
      Ok(())
  }
}