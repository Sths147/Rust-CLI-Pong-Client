use anyhow::{Result, anyhow};
use crossterm::event::{
    Event,
    KeyCode,
    KeyModifiers
};

pub const LOGO: &str = r#"
  ██████╗  ██████╗ ███╗   ██╗ ██████╗ 
  ██╔══██╗██╔═══██╗████╗  ██║██╔════╝ 
  ██████╔╝██║   ██║██╔██╗ ██║██║  ███╗
  ██╔═══╝ ██║   ██║██║╚██╗██║██║   ██║
  ██║     ╚██████╔╝██║ ╚████║╚██████╔╝
  ╚═╝      ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝ 
  "#;


#[derive(Clone, Copy, PartialEq)]
pub enum CurrentScreen {
  FirstScreen,
  Welcome,
  Login,
  SignUp,
  GameChoice,
  SocialLife,
  CreateGame,
  StartGame,
  PlayGame,
  EndGame,
  FriendsDisplay,
  AddFriend,
  DeleteFriend,
  ErrorScreen,
}

impl Default for CurrentScreen {
  fn default() -> Self {
      CurrentScreen::FirstScreen
  }
}

pub fn get_location() -> Result<String> {
    let mut args = std::env::args();
    args.next();
    let first = match args.next() {
        Some(addr) => addr,
        _ => {
            return Err(anyhow!("no argument provided"));
        }
    };
    Ok(first)
}

pub fn should_exit(event: &Event) -> Result<bool> {
  if let Event::Key(key_event) = event {
    if key_event.code == KeyCode::Esc || 
    (key_event.code == KeyCode::Char('c') 
    && key_event.modifiers == KeyModifiers::CONTROL) {
      return Ok(true);
    }
  }
  Ok(false)
}
