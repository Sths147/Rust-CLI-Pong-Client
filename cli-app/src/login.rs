use crate::Context;
use crate::game::WsStream;
use anyhow::{Result, anyhow};
use futures_util::StreamExt;
use std::collections::HashMap;
use std::rc::Rc;
use tokio::sync::mpsc;
use tokio_tungstenite::{Connector, connect_async_tls_with_config, tungstenite::protocol::Message};

#[derive(Default, PartialEq)]
pub(crate) enum Field {
    #[default]
    Mail,
    Username,
    Password,
    Totp,
}

#[derive(Default)]
pub(crate) struct Auth {
    pub(crate) token: String,
    email: String,
    password: String,
    username: String,
    totp: String,
    field: Field,
    pub(crate) id: u64,
    pub(crate) blink: bool,
    pub(crate) receiver: Option<mpsc::Receiver<serde_json::Value>>,
}

impl Auth {
    pub(crate) fn up_field_signup(&mut self) {
        match self.field {
            Field::Password => self.field = Field::Username,
            Field::Username => self.field = Field::Mail,
            _ => {}
        }
    }
    pub(crate) fn down_field_signup(&mut self) {
        match self.field {
            Field::Mail => self.field = Field::Username,
            Field::Username => self.field = Field::Password,
            _ => {}
        }
    }
    pub(crate) fn up_field_login(&mut self) {
        match self.field {
            Field::Password => self.field = Field::Mail,
            Field::Totp => self.field = Field::Password,
            _ => {}
        }
    }
    pub(crate) fn down_field_login(&mut self) {
        match self.field {
            Field::Mail => self.field = Field::Password,
            Field::Password => self.field = Field::Totp,
            _ => {}
        }
    }
    pub(crate) fn add(&mut self, c: char) {
        match self.field {
            Field::Mail => {
                if self.email.len() < 50 {
                    self.email.push(c)
                };
            }
            Field::Password => {
                if self.password.len() < 50 {
                    self.password.push(c)
                };
            }
            Field::Username => {
                if self.username.len() < 50 {
                    self.username.push(c);
                }
            }
            Field::Totp => {
                if self.totp.len() < 50 {
                    self.totp.push(c);
                }
            }
        }
    }
    pub(crate) fn pop(&mut self) {
        match self.field {
            Field::Mail => {
                self.email.pop();
            }
            Field::Password => {
                self.password.pop();
            }
            Field::Username => {
                self.username.pop();
            }
            Field::Totp => {
                self.totp.pop();
            }
        }
    }
    pub(crate) fn get_email(&self) -> &str {
        &self.email
    }
    pub(crate) fn get_password(&self) -> &str {
        &self.password
    }
    pub(crate) fn get_username(&self) -> &str {
        &self.username
    }
    pub(crate) fn get_totp(&self) -> &str {
        &self.totp
    }
    pub(crate) fn get_field(&self) -> &Field {
        &self.field
    }
    pub(crate) fn tick(&mut self) {
        self.blink = !self.blink;
    }
    pub(crate) fn blinks(&self, field: Field) -> bool {
        self.blink && field == self.field
    }
    pub(crate) fn clear(&mut self) {
        self.email.clear();
        self.password.clear();
        self.username.clear();
        self.totp.clear();
        self.field = Field::Mail;
    }
    pub(crate) fn get_signup_infos(&self) -> (String, String, String) {
        (
            self.get_username().to_string(),
            self.get_password().to_string(),
            self.get_email().to_string(),
        )
    }
    pub(crate) fn get_login_infos(&self) -> (String, String, String) {
        (
            self.get_email().to_string(),
            self.get_password().to_string(),
            self.get_totp().to_string(),
        )
    }
    pub(crate) fn set_credentials(
        &mut self,
        credentials: (String, u64, mpsc::Receiver<serde_json::Value>),
    ) {
        self.token = credentials.0;
        self.id = credentials.1;
        self.receiver = Some(credentials.2);
    }
}

pub(crate) async fn signup(
    context: Rc<Context>,
    signup_infos: (String, String, String),
) -> Result<(String, u64, mpsc::Receiver<serde_json::Value>)> {
    let apiloc = format!("https://{}/api/user/create", context.location);
    let mut body: HashMap<&str, &str> = HashMap::new();
    body.insert("username", &signup_infos.0);
    body.insert("passw", &signup_infos.1);
    body.insert("email", &signup_infos.2);
    let response = context
        .client
        .post(apiloc)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;
    let body: serde_json::Value = response.json().await?;
    if body["token"].as_str().is_some() {
        login(context, (signup_infos.2, signup_infos.1, String::new())).await
    } else if let Some(error) = body["message"].as_str() {
        Err(anyhow!(error.to_string()))
    } else {
        Err(anyhow!("Error signing up"))
    }
}

pub(crate) async fn login(
    context: Rc<Context>,
    login_infos: (String, String, String),
) -> Result<(String, u64, mpsc::Receiver<serde_json::Value>)> {
    let apiloc = format!("https://{}/api/user/login", context.location);
    let mut body: HashMap<&str, &str> = HashMap::new();
    body.insert("email", &login_infos.0);
    body.insert("passw", &login_infos.1);
    if !login_infos.2.is_empty() {
        body.insert("totp", &login_infos.2);
    }
    let response = context
        .client
        .post(apiloc)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;
    let body: serde_json::Value = response.json().await?;
    if let Some(token) = body["token"].as_str() {
        let (id, receiver) = get_id_and_launch_chat(context.clone(), token.to_string()).await?;
        Ok((token.to_string(), id, receiver))
    } else if let Some(error) = body["message"].as_str() {
        Err(anyhow!(error.to_string()))
    } else {
        Err(anyhow!("Error signing up"))
    }
}

pub(crate) async fn get_id_and_launch_chat(
    context: Rc<Context>,
    token: String,
) -> Result<(u64, mpsc::Receiver<serde_json::Value>)> {
    let apiloc = format!("https://{}/api/user/get_profile_token", context.location);
    let mut body = HashMap::new();
    body.insert("token", token);
    let res = context
        .client
        .post(apiloc)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await?;
    let value: serde_json::Value = res.json().await?;
    let player_id = match value["id"].as_u64() {
        Some(nbr) => nbr,
        _ => return Err(anyhow!("Error from server, no data received")),
    };
    let receiver = enter_chat_room(&context.location, player_id).await?;
    Ok((player_id, receiver))
}

pub(crate) async fn create_guest_session(
    context: Rc<Context>,
) -> Result<(String, u64, mpsc::Receiver<serde_json::Value>)> {
    let apiloc = format!("https://{}/api/user/create_guest", context.location);
    let res = context.client.post(apiloc).send().await?;
    let body: serde_json::Value = res.json().await?;
    if let Some(token) = body["token"].as_str() {
        let (id, receiver) = get_id_and_launch_chat(context, token.to_string()).await?;
        Ok((token.to_string(), id, receiver))
    } else if let Some(error) = body["message"].as_str() {
        Err(anyhow!(error.to_string()))
    } else {
        Err(anyhow!("Error signing up"))
    }
}

async fn enter_chat_room(location: &String, id: u64) -> Result<mpsc::Receiver<serde_json::Value>> {
    let connector = Connector::NativeTls(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .build()?,
    );
    let request = format!("wss://{}/api/chat?userid={}", location, id);
    let (ws_stream, _) =
        connect_async_tls_with_config(request, None, false, Some(connector)).await?;
    let (sender, receiver): (
        mpsc::Sender<serde_json::Value>,
        mpsc::Receiver<serde_json::Value>,
    ) = mpsc::channel(1024);
    tokio::spawn(async move {
        if let Err(e) = chat(ws_stream, sender).await {
            eprintln!("Error: {e}");
        }
    });
    Ok(receiver)
}

async fn chat(mut ws_stream: WsStream, sender: mpsc::Sender<serde_json::Value>) -> Result<()> {
    while let Some(msg) = ws_stream.next().await {
        let last_message = match msg {
            Ok(Message::Text(result)) => result,
            _ => {
                continue;
            }
        };
        let message: serde_json::Value = serde_json::from_str(last_message.as_str())?;
        match message["gameId"].as_str() {
            Some(_) => sender.send(message).await?,
            _ => {
                continue;
            }
        };
    }
    Ok(())
}
