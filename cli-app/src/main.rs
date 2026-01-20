mod game;
mod friends;
mod infos_events;
mod screen_displays;
mod game_demo;
mod context;
mod login;
mod utils;
mod infos;

use std::rc::Rc;
use std::cell::{Cell, RefCell};
use anyhow::{Result, anyhow};
use infos::Infos;
use context::Context;
use friends::Friends;
use login::Auth;
use utils::{
    LOGO,
    CurrentScreen,
    get_location,
  };


#[tokio::main]
async fn main() -> Result<()> {
  let location = match get_location() {
    Ok(result) => result,
    Err(e) => {return Err(anyhow!("{}", e));},
  };
  let context = Rc::new(Context::new(location.clone()));
  let auth = Rc::new(RefCell::new(Auth::default()));
  let screen = Rc::new(Cell::new(CurrentScreen::default()));
  let friends = Friends::new(context.clone(), auth.clone(), screen.clone());
  let mut terminal = ratatui::init();
  let game_main = Infos::new(context, auth, screen, friends);
  let app_result = game_main.run(&mut terminal).await;
  ratatui::restore();
  app_result
}