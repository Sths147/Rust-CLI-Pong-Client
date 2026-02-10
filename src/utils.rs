use crate::Context;
use anyhow::{Result, anyhow};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use std::rc::Rc;

pub(crate) const LOGO: &str = r#"
  ██████╗  ██████╗ ███╗   ██╗ ██████╗ 
  ██╔══██╗██╔═══██╗████╗  ██║██╔════╝ 
  ██████╔╝██║   ██║██╔██╗ ██║██║  ███╗
  ██╔═══╝ ██║   ██║██║╚██╗██║██║   ██║
  ██║     ╚██████╔╝██║ ╚████║╚██████╔╝
  ╚═╝      ╚═════╝ ╚═╝  ╚═══╝ ╚═════╝ 
  "#;

#[derive(Clone, Copy, PartialEq, Default)]
pub(crate) enum CurrentScreen {
    #[default]
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

pub(crate) fn get_location() -> Result<String> {
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

///Checks for ESC of Ctrl+C event
pub(crate) fn should_exit(event: &Event) -> Result<bool> {
    if let Event::Key(key_event) = event
        && (key_event.code == KeyCode::Esc
            || (key_event.code == KeyCode::Char('c')
                && key_event.modifiers == KeyModifiers::CONTROL))
    {
        return Ok(true);
    }
    Ok(false)
}

pub(crate) async fn get_name_from_id(context: Rc<Context>, id: u64) -> Result<String> {
    let apiloc = format!(
        "https://{}/api/user/get_profile_id?user_id={}",
        context.location, id
    );
    let response = context.client.get(apiloc).send().await?;
    let response: serde_json::Value = response.json().await?;
    if let Some(result) = response["name"].as_str() {
        return Ok(result.to_string());
    }
    Err(anyhow!("Opponent as no name"))
}

pub(crate) async fn get_id_from_name(context: Rc<Context>, name: &String) -> Result<i64> {
    let apiloc = format!(
        "https://{}/api/user/get_profile_name?profile_name={}",
        context.location, name
    );
    let response = context.client.get(apiloc).send().await?;
    let response: serde_json::Value = response.json().await?;
    let result: i64 = match response["id"].as_i64() {
        Some(id) => id,
        _ => return Err(anyhow!("Friend not found")),
    };
    Ok(result)
}