use reqwest::header::HeaderMap;
use crossterm::event::{self, poll, Event, KeyCode, KeyEventKind};
use futures::stream::{StreamExt};
use futures_util::{SinkExt, stream::SplitStream};
use futures_util::stream::SplitSink;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use bytes::Bytes;
use std::rc::Rc;
use tokio_tungstenite::{
	connect_async_tls_with_config,
	MaybeTlsStream,
	WebSocketStream,
	Connector,
	tungstenite::{
		Utf8Bytes,
		protocol::Message,
		client::IntoClientRequest,
	},
};
use anyhow::{Result, anyhow};
use std::cell::RefCell;
use tokio::sync::{mpsc, watch};
use tokio::net::TcpStream;
use crate::{Auth, Context};
use crate::utils::should_exit;
use crate::CurrentScreen;
use crate::Infos;

#[derive(Default)]
pub struct Game {
	auth: Rc<RefCell<Auth>>,
	context: Rc<Context>,
	game_id: String,
	pub opponent_name: String,
	player_side: u64,
	receiver: Option<watch::Receiver<(Option<Bytes>, Option<Utf8Bytes>)>>,
	pub game_checker: Option<watch::Receiver<bool>>,
	pub game_stats: GameStats,
	game_sender: Option<mpsc::Sender<u8>>,
}

#[derive(Default)]
pub struct GameStats {
	pub left_y: f32,
	pub right_y: f32,
	pub ball_x: f32,
	pub ball_y: f32,
	pub speed_x: f32,
	pub speed_y: f32,
	pub player1_score: u8,
	pub player2_score: u8,
	pub winner: bool,
}

pub trait Gameplay {
	async fn create_game(&mut self, mode: &str) -> Result<()>;
	async fn launch_game(&mut self) -> Result<()>;
	async fn handle_game_events(&mut self) -> Result<()>;
	async fn send_remove_from_queue_request(&self) -> Result<()>;
	fn handle_endgame(&mut self) -> Result<()>; 
}

impl Gameplay for Infos {
	async fn create_game(&mut self, mode: &str) -> Result<()> {
		send_post_game_request(&self, mode).await?;
		loop {
			match poll(Duration::from_millis(16)) {
				Ok(true) => {
					if !self.authent.borrow_mut().receiver.as_mut().expect("empty receiver").is_empty() {
						break ;
					}
					let event = event::read()?;
					match should_exit(&event) {
						Ok(true) => {
							self.send_remove_from_queue_request().await?;
							self.screen.set(CurrentScreen::GameChoice);
							return Ok(());
						}
						_ => {},
					}
				},
				Ok(false) => {
						if !self.authent.borrow_mut().receiver.as_mut().expect("empty receiver").is_empty() {
							break ;
					}
				},
				_ => return Err(anyhow!("error in poll".to_string()))
			};
		}
		let response = self.authent.borrow_mut().receiver.as_mut().expect("empty receiver").try_recv()?;
		let game = Game::new(&self, response).await?;
		self.game = game;
		self.screen.set(crate::CurrentScreen::StartGame);
		Ok(())
	}
	async fn launch_game(&mut self) -> Result<()> {
		self.game.start_game().await?;
		self.screen.set(crate::CurrentScreen::PlayGame);
		Ok(())
	}
	async fn handle_game_events(&mut self) -> Result<()> {
		let mut state_receiver = match self.game.receiver.clone() {
			Some(receiver) => receiver,
			_ => {return Err(anyhow!("State receiver is empty"));}
		};
		if let Some(checker) = &mut self.game.game_checker {
			if let Ok(true) = checker.has_changed() {
				self.screen.set(crate::CurrentScreen::GameChoice);
			}
		};
		if let Some(sender) = &self.game.game_sender {
			state_receiver.changed().await?;
			let (bytes, text) = state_receiver.borrow_and_update().clone();
			match (bytes, text) {
				(Some(bytes), _none) => {self.game.decode_and_update(bytes)?;},
				(_none, Some(text)) => {
					self.game.end_game(text, sender.clone()).await?;
					self.screen.set(crate::CurrentScreen::EndGame);
				},
				_ => {}
			};
		}
		Ok(())
	}
	fn handle_endgame(&mut self) -> Result<()> {
		if poll(Duration::from_millis(16))? {
			let event = event::read()?;
			if should_exit(&event)? {
				self.screen.set(crate::CurrentScreen::GameChoice);
			} else if let Event::Key(keyevent) = event {
				match keyevent.code {
					KeyCode::Enter => {self.screen.set(crate::CurrentScreen::GameChoice)},
					_ => {},
				}
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
		self.context.client.delete(url)
			.headers(headers)
			.json(&map)
			.send()
			.await?;
		Ok(())
	}
}

impl Game {
	async fn new(info: &Infos, value: serde_json::Value) -> Result<Game> {
		let game_id: String = match value["gameId"].as_str() {
			Some(id) => id.to_string(),
			_ => return Err(anyhow!("No game Id in response")),
		};
		let opponent_id = match value["opponentId"].as_u64() {
			Some(id) => id,
			_ => return Err(anyhow!("No opponent id in response")),
		};
		let opponent_name: String = get_opponent_name(info, opponent_id).await?;
		let player_side: u64 = match value["playerSide"].as_u64() {
			Some(nbr) => nbr,
			_ => return Err(anyhow!("No player Id in response")),
		};
		Ok(Game{
			context: info.context.clone(),
			auth: info.authent.clone(),
			game_id,
			player_side: player_side,
			opponent_name: opponent_name,
			..Default::default()
		})
	}
	// async fn launch_countdown(&self) -> Result<()> {
	// 	//3...2....1....0 -->
	// 	//Affiche le compte a rebours puis a 0 START GAME
	// 	self.start_game().await?;
	// 	Ok(())
	// }
	async fn start_game(&mut self) -> Result<()> {
		let url = format!("https://{}/api/start-game/{}", self.context.location, self.game_id);
		self.context.client.post(url).send().await?;
		let request = format!("wss://{}/api/game/{}/{}", self.context.location, self.game_id, self.player_side).into_client_request()?;
		let connector = Connector::NativeTls(
			native_tls::TlsConnector::builder()
				.danger_accept_invalid_certs(true)
				.build()?
		);
		let (ws_stream, _) = connect_async_tls_with_config(
			request,
			None,
			false,
			Some(connector),
			).await?;
		let (ws_write, ws_read) = ws_stream.split();
		let (sender, receiver): (mpsc::Sender<u8>, mpsc::Receiver<u8>) = mpsc::channel(1);
		let (state_sender, state_receiver): 
			(watch::Sender<(Option<Bytes>, Option<Utf8Bytes>)>, watch::Receiver<(Option<Bytes>, Option<Utf8Bytes>)>) 
				= watch::channel((None, None));
		self.receiver = Some(state_receiver);
		let (game_sender, game_checker): (watch::Sender<bool>, watch::Receiver<bool>) = watch::channel(true);
		self.game_checker = Some(game_checker);
		let socket_checker = game_sender.subscribe();
		tokio::task::spawn(async move {
			if let Err(e) = Self::send_game(ws_write, receiver, game_sender).await {
				eprintln!("Error: {}", e);
			}
		});
		tokio::spawn(async move {
			Self::read_socket(ws_read, state_sender, socket_checker).await;
		});
		self.game_sender = Some(sender);
		Ok(())
	}
	async fn end_game(&mut self, text: Utf8Bytes, sender: mpsc::Sender<u8>) -> Result<()> {
		let value = serde_json::to_string(text.as_str())?;
		match value.find(&self.auth.borrow().id.to_string()) {
			Some(_) => self.game_stats.winner = true,
			_ => self.game_stats.winner = false,
		};
		let u: u8 = 1;
		sender.send(u).await?;
		Ok(())
	}
	fn decode_and_update(&mut self, msg: Bytes) -> Result<()> {
		if msg.len() == 26 {
			let (left_y, right_y, ball_x, ball_y, speed_x, speed_y, score1, score2 ) = Self::decode(msg)?;
			self.game_stats.left_y = left_y;
			self.game_stats.right_y = right_y;
			self.game_stats.ball_x = ball_x;
			self.game_stats.ball_y = ball_y;
			self.game_stats.speed_x = speed_x;
			self.game_stats.speed_y = speed_y;
			self.game_stats.player1_score = score1;
			self.game_stats.player2_score = score2;
		}
		Ok(())
	}
	fn decode(msg: Bytes) -> Result<(f32, f32, f32, f32, f32, f32, u8, u8)> {
		let left_y: f32 = f32::from_le_bytes(msg[0..4].try_into()?);
		let right_y: f32 = f32::from_le_bytes(msg[4..8].try_into()?);
		let ball_x: f32 = f32::from_le_bytes(msg[8..12].try_into()?);
		let ball_y: f32 = f32::from_le_bytes(msg[12..16].try_into()?);
		let _speed_x: f32 = f32::from_le_bytes(msg[16..20].try_into()?);
		let _speed_y: f32 = f32::from_le_bytes(msg[20..24].try_into()?);
		let player1_score: u8 =  msg[24];
		let player2_score: u8 =  msg[25];
		Ok((left_y, right_y, ball_x, ball_y, _speed_x, _speed_y, player1_score, player2_score))
	}
	async fn send_game(mut ws_write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>, mut receiver: mpsc::Receiver<u8>, game_sender: watch::Sender<bool>) -> Result<()> {
		let mut up: (bool, Instant, u128) = (false, std::time::Instant::now(), 0);
		let mut down: (bool, Instant, u128) = (false, std::time::Instant::now(), 0);
		let mut to_send = String::new();
		loop {
			if let Ok(_) = receiver.try_recv() {
				break;
			}
			to_send.clear();
			if up.0 == true {
				to_send.insert_str(0, "U");
			}
			if down.0 == true {
				to_send.insert_str(0, "D");
			}
			if !to_send.is_empty() {
				let send_it = to_send.clone();
				ws_write.send(send_it.into()).await?;
			}
			if poll(Duration::from_millis(16))? {
				let event = event::read()?;
				if should_exit(&event)? == true {
					game_sender.send(true)?;
					break;
				} 
				else if let Event::Key(key_event) = event {
					match key_event.code {
						KeyCode::Up => match key_event.kind {
							KeyEventKind::Press => {up = (true, std::time::Instant::now(), 150)},
							KeyEventKind::Repeat => {up = (true, std::time::Instant::now(), 150)},
							KeyEventKind::Release => {up = (false, std::time::Instant::now(), 150)},
						},
						KeyCode::Down => match key_event.kind {
							KeyEventKind::Press => {down = (true, std::time::Instant::now(), 150)},
							KeyEventKind::Repeat => {down = (true, std::time::Instant::now(), 150)},
							KeyEventKind::Release => {down = (false, std::time::Instant::now(), 150)},
						},						
						_ => {continue;},
					}
				};
			}
			if up.1.elapsed().as_millis() > up.2 {
				up.0 = false;
			}
			if down.1.elapsed().as_millis() > down.2 {
				down.0 = false;
			}
			tokio::time::sleep(Duration::from_millis(1)).await;
		}
		ws_write.close().await?;
		Ok(())
	}
	async fn read_socket(mut ws_read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>, 
			state_sender: watch::Sender<(Option<Bytes>, Option<Utf8Bytes>)>, socket_checker: watch::Receiver<bool>) {
		loop {
			match ws_read.next().await {
				Some(msg) => {
						match msg {
							Ok(Message::Binary(b)) => {
								if let Err(_) = state_sender.send((Some(b), None)) {
									break;
							}
						}
						Ok(Message::Text(s)) => {
							if let Err(_) = state_sender.send((None, Some(s))) {
								break;
							}
						}
						Ok(Message::Close(_)) => {},
					_ => {}
					}
				}
				_ => {}
			}
			match socket_checker.has_changed() {
				Ok(false) => {},
				_ => break,
			}
		}
	}
}

async fn get_opponent_name(infos: &Infos, id: u64) -> Result<String> {
	let apiloc = format!("https://{}/api/user/get_profile_id?user_id={}", infos.context.location, id);
	let response = infos.context.client.get(apiloc)
			.send()
			.await?;
	let response: serde_json::Value = response.json().await?;
	if let Some(result) = response["name"].as_str() {
		return Ok(result.to_string());
	}
	Err(anyhow!("Opponent as no name"))
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
	game_main.context.client.post(url)
        .headers(headers)
        .json(&map)
        .send()
        .await?;
	Ok(())
}