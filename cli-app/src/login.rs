use std::collections::HashMap;
use std::rc::Rc;
use anyhow::{Result, anyhow};
use tokio_tungstenite::{
    Connector,
    MaybeTlsStream,
    WebSocketStream,
    connect_async_tls_with_config,
    tungstenite::protocol::Message,
};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use futures_util::StreamExt;
use crate::Context;

#[derive(Default, PartialEq)]
pub enum Field {
    #[default]
    Mail,
    Username,
    Password,
    Totp,
}

#[derive(Default)]
pub struct Auth {
    context: Rc<Context>,
    token: String,
    email: String,
    password: String,
    username: String,
    totp: String,
    field: Field,
    pub id: u64,
    pub blink: bool,
    pub receiver: Option<mpsc::Receiver<serde_json::Value>>,
}

impl Auth {
    pub fn new(context: Rc<Context>) -> Self {
        Auth {
            context,
            ..Default::default()
        }
    }
    pub fn up_field_signup(&mut self) {
        match self.field {
            Field::Password => {self.field = Field::Username},
            Field::Username => {self.field = Field::Mail},
            _ => {},
        }
    }
    pub fn down_field_signup(&mut self) {
        match self.field {
            Field::Mail => {self.field = Field::Username},
            Field::Username => {self.field = Field::Password},
            _ => {},
        }
    }
    pub fn up_field_login(&mut self) {
        match self.field {
            Field::Password => {self.field = Field::Mail},
            Field::Totp => {self.field = Field::Password},
            _ => {},
        }
    }
    pub fn down_field_login(&mut self) {
        match self.field {
            Field::Mail => {self.field = Field::Password},
            Field::Password => {self.field = Field::Totp},
            _ => {},
        }
    }
    pub fn add(&mut self, c: char) {
        match self.field {
            Field::Mail => {if self.email.len() < 50 {self.email.push(c)};},
            Field::Password => {if self.password.len() < 50 {self.password.push(c)};},
            Field::Username => {if self.username.len() < 50 {self.username.push(c);}},
            Field::Totp => {if self.totp.len() < 50 {self.totp.push(c);}},
        }        
    }
    pub fn pop(&mut self) {
        match self.field {
            Field::Mail => {self.email.pop();},
            Field::Password => {self.password.pop();},
            Field::Username => {self.username.pop();},
            Field::Totp => {self.totp.pop();},
        }
    }
    pub fn set_token(&mut self, token: &str) {
        self.token = token.to_string();
    }
    pub fn get_token(&self) -> &str {
        &self.token
    }
    pub fn get_email(&self) -> &str {
        &self.email
    }
    pub fn get_password(&self) -> &str {
        &self.password
    }
    pub fn get_username(&self) -> &str {
        &self.username
    }
    pub fn get_totp(&self) -> &str {
        &self.totp
    }
    pub fn get_field(&self) -> &Field {
        &self.field
    }
    pub fn tick(&mut self) {
        self.blink = !self.blink;
    }
    pub fn blinks(&self, field: Field) -> bool {
        self.blink && field == self.field
    }
    pub fn clear(&mut self) {
        self.email.clear();
        self.password.clear();
        self.username.clear();
        self.totp.clear();
        self.field = Field::Mail;
    }
    pub async fn signup(& self) -> Result<(String, u64, mpsc::Receiver<serde_json::Value>)> {
        let apiloc = format!("https://{}/api/user/create", self.context.location);
        let mut body: HashMap<&str, &str> = HashMap::new();
        body.insert("username", self.get_username());
        body.insert("passw", self.get_password());
        body.insert("email", self.get_email());
        let response = self.context.client.post(apiloc)
                                                .header("content-type", "application/json")
                                                .json(&body)
                                                .send()
                                                .await?;
        let body: serde_json::Value = response.json().await?;
        if body["token"].as_str().is_some() {
            self.login().await
        } else if let Some(error) = body["message"].as_str() {
            Err(anyhow!(error.to_string()))
        } else {
            Err(anyhow!("Error signing up"))
        }
    }
    pub async fn login(&self) -> Result<(String, u64, mpsc::Receiver<serde_json::Value>)> {
        let apiloc = format!("https://{}/api/user/login", self.context.location);
        let mut body: HashMap<&str, &str> = HashMap::new();
        body.insert("email", self.get_email());
        body.insert("passw", self.get_password());
        if !self.get_totp().is_empty() {
            body.insert("totp", self.get_totp());
        }
        let response = self.context.client.post(apiloc)
                                                .header("content-type", "application/json")
                                                .json(&body)
                                                .send()
                                                .await?;
        let body: serde_json::Value = response.json().await?;
        if let Some(token) = body["token"].as_str() {
            let (id, receiver) = self.get_id_and_launch_chat(token.to_string()).await?;
            Ok((token.to_string(), id, receiver))
        } else if let Some(error) = body["message"].as_str() {
            Err(anyhow!(error.to_string()))
        } else {
            Err(anyhow!("Error signing up"))
        }
    }
    pub async fn create_guest_session(&self) -> Result<(String, u64, mpsc::Receiver<serde_json::Value>)> {
        let apiloc = format!("https://{}/api/user/create_guest", self.context.location);
        let res = self.context.client.post(apiloc)
            .send()
            .await?;
        let body: serde_json::Value = res.json().await?;
        if let Some(token) = body["token"].as_str() {
            let (id, receiver) = self.get_id_and_launch_chat(token.to_string()).await?;
            Ok((token.to_string(), id, receiver))
        } else if let Some(error) = body["message"].as_str() {
            return Err(anyhow!(error.to_string()));
        } else {
            return Err(anyhow!("Error signing up"));
        }
    }
    pub async fn get_id_and_launch_chat(&self, token: String) -> Result<(u64, mpsc::Receiver<serde_json::Value>)> {
        let apiloc = format!("https://{}/api/user/get_profile_token", self.context.location);
        let mut body = HashMap::new();
        body.insert("token", token);
        let res = self.context.client.post(apiloc)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;
        let value: serde_json::Value = res.json().await?;
        let player_id = match value["id"].as_u64(){
            Some(nbr) => nbr,
            _ => return Err(anyhow!("Error from server, no data received")),
        };
        let receiver = enter_chat_room(&self.context.location, player_id).await?;
        Ok((player_id, receiver))
    }
    pub fn set_credentials(&mut self, credentials: (String, u64, mpsc::Receiver<serde_json::Value>)) {
        self.token = credentials.0;
        self.id = credentials.1;
        self.receiver = Some(credentials.2);
    }
}


async fn enter_chat_room(location: &String, id: u64) -> Result<mpsc::Receiver<serde_json::Value>> {
    let connector = Connector::NativeTls(
			native_tls::TlsConnector::builder()
				.danger_accept_invalid_certs(true)
				.build()?
		);
	let request = format!("wss://{}/api/chat?userid={}", location, id);
	let (ws_stream, _) = connect_async_tls_with_config(
			request,
			None,
			false,
			Some(connector),
			)
            .await?;
    let (sender, receiver): (mpsc::Sender<serde_json::Value>, mpsc::Receiver<serde_json::Value>)  = mpsc::channel(1024);
    tokio::spawn(async move {
        if let Err(e) = chat(ws_stream, sender).await {
            eprintln!("Error: {e}");
        }
    });
    Ok(receiver)
}

async fn   chat(mut ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>, sender: mpsc::Sender<serde_json::Value>) -> Result<()> {
    while let Some(msg) =  ws_stream.next().await {
        let last_message = match msg {
            Ok(Message::Text(result)) => result,
            _ => {continue;},
        };
        let message: serde_json::Value = serde_json::from_str(last_message.as_str())?;
        match message["gameId"].as_str() {
            Some(_) => {sender.send(message).await?},
            _ => {continue;}
        };
    }
    Ok(())
}
