mod context;
mod friends;
mod game;
mod game_demo;
mod infos;
mod infos_events;
mod login;
mod screen_displays;
mod utils;

use anyhow::{Result, anyhow};
use context::Context;
use friends::Friends;
use infos::Infos;
use login::Auth;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use utils::{CurrentScreen, LOGO, get_location};

#[tokio::main]
async fn main() -> Result<()> {
    let location = match get_location() {
        Ok(result) => result,
        Err(e) => {
            return Err(anyhow!("{}", e));
        }
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
