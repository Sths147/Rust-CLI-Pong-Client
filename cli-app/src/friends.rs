use crate::Auth;
use crate::Context;
use crate::CurrentScreen;
use crate::utils::{get_name_from_id, get_id_from_name};
use crate::utils::should_exit;
use anyhow::{Result, anyhow};
use crossterm::event::poll;
use crossterm::event::{self, Event, KeyCode};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

#[derive(Default)]
pub(crate) struct Friends {
    auth: Rc<RefCell<Auth>>,
    context: Rc<Context>,
    screen: Rc<Cell<CurrentScreen>>,
    pub(crate) index: usize,
    pub(crate) index_max: usize,
    pub(crate) friends_list: Vec<String>,
    pub(crate) friend_tmp: String,
    pub(crate) blink: bool,
}

impl Friends {
    pub(crate) fn new(
        context: Rc<Context>,
        auth: Rc<RefCell<Auth>>,
        screen: Rc<Cell<CurrentScreen>>,
    ) -> Self {
        Friends {
            auth,
            context,
            screen,
            ..Default::default()
        }
    }
    ///Get friends list with pending attribute if friend is not accepted yet
    pub(crate) async fn get_indexed_friends(&mut self) -> Result<()> {
        let friends_list = self.get_all_friends().await?;
        let mut printable: Vec<String> = vec![];
        let mut _str_tmp: String = String::new();
        for element in &friends_list[..] {
            _str_tmp = element.0.clone();
            if !element.1 {
                _str_tmp += " (Pending)";
            }
            printable.push(_str_tmp);
        }
        self.friends_list = printable;
        Ok(())
    }
    ///Update friends display depending on the terminal's size
    pub(crate) async fn update_friends_index(
        &mut self,
        terminal: &mut ratatui::DefaultTerminal,
    ) -> Result<()> {
        self.get_indexed_friends().await?;
        let height: usize = (terminal.get_frame().area().height - 2) as usize;
        let len = self.friends_list.len();
        let modulo: usize = match height {
            0 => 0,
            _ => match len % height {
                0 => 0,
                _ => 1,
            },
        };
        if height < len && height != 0 {
            self.index_max = len / height + modulo;
        } else {
            self.index_max = 0;
        }
        if self.index > self.index_max {
            self.index = 0;
        }
        Ok(())
    }
    pub(crate) async fn add_friend(&mut self) -> Result<()> {
        if poll(Duration::from_millis(500))? {
            let event = event::read()?;
            if should_exit(&event)? {
                self.friend_tmp.clear();
                self.screen.set(CurrentScreen::FriendsDisplay);
            } else if let Event::Key(eventkey) = event {
                match eventkey.code {
                    KeyCode::Backspace => {
                        self.friend_tmp.pop();
                    }
                    KeyCode::Char(c) => self.friend_tmp.push(c),
                    KeyCode::Enter => {
                        self.send_friend_request().await?;
                        self.get_indexed_friends().await?;
                    }
                    _ => {}
                }
            }
        }
        self.tick();
        Ok(())
    }
    pub(crate) async fn delete_friend(&mut self) -> Result<()> {
        if poll(Duration::from_millis(500))? {
            let event = event::read()?;
            if should_exit(&event)? {
                self.friend_tmp.clear();
                self.screen.set(CurrentScreen::FriendsDisplay);
            } else if let Event::Key(eventkey) = event {
                match eventkey.code {
                    KeyCode::Backspace => {
                        self.friend_tmp.pop();
                    }
                    KeyCode::Char(c) => self.friend_tmp.push(c),
                    KeyCode::Enter => {
                        self.send_delete_friend_request().await?;
                        self.get_indexed_friends().await?;
                    }
                    _ => {}
                }
            }
        }
        self.tick();
        Ok(())
    }
    async fn send_friend_request(&mut self) -> Result<()> {
        let mut map = HashMap::new();
        let token = self.auth.borrow().token.to_string();
        map.insert("token", &token);
        let id = get_id_from_name(self.context.clone(), &self.friend_tmp).await?.to_string();
        map.insert("friend_id", &id);
        let url = format!("https://{}/api/friends/send_request", self.context.location);
        let response = self
            .context
            .client
            .post(url)
            .header("content-type", "application/json")
            .json(&map)
            .send()
            .await?;
        self.friend_tmp.clear();
        match response.status().as_u16() {
            200 => {
                self.screen.set(CurrentScreen::FriendsDisplay);
            }
            _ => {
                let message: serde_json::Value = response.json().await?;
                if let Some(error_message) = message["message"].as_str() {
                    return Err(anyhow!(error_message.to_string()));
                }
            }
        }
        Ok(())
    }
    async fn send_delete_friend_request(&mut self) -> Result<()> {
        let mut map = HashMap::new();
        let token = self.auth.borrow().token.to_string();
        map.insert("token", &token);
        let id = get_id_from_name(self.context.clone(), &self.friend_tmp).await?.to_string();
        map.insert("friend_id", &id);
        let url = format!("https://{}/api/friends/remove", self.context.location);
        let response = self
            .context
            .client
            .delete(url)
            .header("content-type", "application/json")
            .json(&map)
            .send()
            .await?;
        self.friend_tmp.clear();
        match response.status().as_u16() {
            200 => {
                self.screen.set(CurrentScreen::FriendsDisplay);
            }
            _ => {
                let message: serde_json::Value = response.json().await?;
                if let Some(message) = message["message"].as_str() {
                    return Err(anyhow!(message.to_string()));
                }
            }
        }
        Ok(())
    }
    async fn get_all_friends(&self) -> Result<Vec<(String, bool)>> {
        let url = format!(
            "https://{}/api/friends/get?user_id={}",
            self.context.location,
            self.auth.borrow().id
        );
        let response = self.context.client.get(url).send().await?;
        let mut result: Vec<(String, bool)> = vec![];
        match response.status().as_u16() {
            200 => {
                let response_array: serde_json::Value = response.json().await?;
                if response_array.is_array() {
                    let response_array = match response_array.as_array() {
                        Some(array) => array,
                        _ => {
                            return Err(anyhow!("empty array"));
                        }
                    };
                    for object in response_array {
                        let map = match object.as_object() {
                            Some(map) => map,
                            _ => {
                                continue;
                            }
                        };
                        let name = self.look_for_name(object).await?;
                        match map["pending"].as_u64() {
                            Some(0) => {
                                result.push((name, true));
                            }
                            Some(1) => {
                                result.push((name, false));
                            }
                            _ => {}
                        }
                    }
                }
            }
            404 => {
                eprintln!("No friends found :(");
            }
            err => {
                eprintln!("Error {} from server :(", err);
            }
        }
        Ok(result)
    }
    ///Given a json containing two user_id including ours, return other id name
    ///
    /// The server sends a friends list containing 2 ids: ours and the friend's
    ///
    /// Function first compares to both id to get the friend's id
    /// then sends a request to get name from id
    async fn look_for_name(&self, object: &serde_json::Value) -> Result<String> {
        let id_to_find = match object["user1_id"].as_u64() {
            Some(user1) => {
                let id = self.auth.borrow().id;
                if user1 != id {
                    user1
                } else {
                    match object["user2_id"].as_u64() {
                        Some(user2) => {
                            if user2 != id {
                                user2
                            } else {
                                return Err(anyhow!("from user ids"));
                            }
                        }
                        _ => {
                            return Err(anyhow!("from user ids"));
                        }
                    }
                }
            }
            _ => {
                return Err(anyhow!("from user ids"));
            }
        };
        get_name_from_id(self.context.clone(), id_to_find).await
    }
    pub(crate) fn tick(&mut self) {
        self.blink = !self.blink;
    }
}
