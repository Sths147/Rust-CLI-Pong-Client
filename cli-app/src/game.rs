use crate::Infos;
use crate::utils::{get_name_from_id, should_exit};
use crate::{Auth, Context};
use anyhow::{Result, anyhow};
use bytes::Bytes;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, poll};
use futures::stream::StreamExt;
use futures_util::{
    SinkExt,
    stream::{SplitSink, SplitStream},
};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::{
    Connector, MaybeTlsStream, WebSocketStream, connect_async_tls_with_config,
    tungstenite::{Utf8Bytes, client::IntoClientRequest, protocol::Message},
};

type GameChannel = (
    watch::Sender<(Option<Bytes>, Option<Utf8Bytes>)>,
    watch::Receiver<(Option<Bytes>, Option<Utf8Bytes>)>,
);
pub(crate) type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Default)]
pub(crate) struct Game {
    auth: Rc<RefCell<Auth>>,
    context: Rc<Context>,
    game_id: String,
    pub(crate) opponent_name: String,
    player_side: u64,
    pub(crate) receiver: Option<watch::Receiver<(Option<Bytes>, Option<Utf8Bytes>)>>,
    pub(crate) game_checker: Option<watch::Receiver<bool>>,
    pub(crate) game_stats: GameStats,
    pub(crate) game_sender: Option<mpsc::Sender<u8>>,
}

#[derive(Default)]
pub(crate) struct GameStats {
    pub(crate) left_y: f32,
    pub(crate) right_y: f32,
    pub(crate) ball_x: f32,
    pub(crate) ball_y: f32,
    pub(crate) player1_score: u8,
    pub(crate) player2_score: u8,
    pub(crate) winner: bool,
}

impl Game {
    ///Creates a new game instance
    ///
    /// #Parameters
    /// - info: Main structure
    /// - value: json game informations sent through websocket to infos.receiver
    ///
    /// #Returns
    /// New game instance
    ///
    /// #Errors
    /// Returns an error if no gameId, opponentId or playerSide found in request
    /// Returns an error if get_opponent_name returns an error
    ///
    pub(crate) async fn new(info: &Infos, value: serde_json::Value) -> Result<Game> {
        let game_id: String = match value["gameId"].as_str() {
            Some(id) => id.to_string(),
            _ => return Err(anyhow!("No game Id in response")),
        };
        let opponent_id = match value["opponentId"].as_u64() {
            Some(id) => id,
            _ => return Err(anyhow!("No opponent id in response")),
        };
        let opponent_name: String = get_name_from_id(info.context.clone(), opponent_id).await?;
        let player_side: u64 = match value["playerSide"].as_u64() {
            Some(nbr) => nbr,
            _ => return Err(anyhow!("No player Id in response")),
        };
        Ok(Game {
            context: info.context.clone(),
            auth: info.authent.clone(),
            game_id,
            player_side,
            opponent_name,
            ..Default::default()
        })
    }
    // async fn launch_countdown(&self) -> Result<()> {
    // 	//3...2....1....0 -->
    // 	//Affiche le compte a rebours puis a 0 START GAME
    // 	self.start_game().await?;
    // 	Ok(())
    // }
    /// Start a new game
    pub(crate) async fn start_game(&mut self) -> Result<()> {
        let ws_stream = self.connect_wss().await?;
        self.split_and_spawn_sockets(ws_stream).await?;
        Ok(())
    }
    ///Initiate websocket connection with game server
    async fn connect_wss(&self) -> Result<WsStream> {
        let url = format!(
            "https://{}/api/start-game/{}",
            self.context.location, self.game_id
        );
        self.context.client.post(url).send().await?;
        let request = format!(
            "wss://{}/api/game/{}/{}",
            self.context.location, self.game_id, self.player_side
        )
        .into_client_request()?;
        let connector = Connector::NativeTls(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .build()?,
        );
        let (ws_stream, _) =
            connect_async_tls_with_config(request, None, false, Some(connector)).await?;
        Ok(ws_stream)
    }
    ///Split the websocket stream and spawn two async tasks to independently read game state from server and send events
    ///
    /// #Parameters:
    /// ws_stream: websocket stream connected with game server
    async fn split_and_spawn_sockets(&mut self, ws_stream: WsStream) -> Result<()> {
        let (ws_write, ws_read) = ws_stream.split();
        let (sender, receiver): (mpsc::Sender<u8>, mpsc::Receiver<u8>) = mpsc::channel(1);
        let (state_sender, state_receiver): GameChannel = watch::channel((None, None));
        self.receiver = Some(state_receiver);
        let (game_sender, game_checker): (watch::Sender<bool>, watch::Receiver<bool>) =
            watch::channel(true);
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
    /// Get the winner's name and send shutdown signal to spawned task
    ///
    /// #Parameters:
    /// - text: text sent by the server containing the id of the winner
    /// - sender: sender to use to shut down task sending game events to server
    pub(crate) async fn end_game(
        &mut self,
        text: Utf8Bytes,
        sender: mpsc::Sender<u8>,
    ) -> Result<()> {
        let value = serde_json::to_string(text.as_str())?;
        match value.find(&self.auth.borrow().id.to_string()) {
            Some(_) => self.game_stats.winner = true,
            _ => self.game_stats.winner = false,
        };
        let u: u8 = 1;
        sender.send(u).await?;
        Ok(())
    }
    pub(crate) fn decode_and_update(&mut self, msg: Bytes) -> Result<()> {
        if msg.len() == 26 {
            self.game_stats = Self::decode(msg)?;
        }
        Ok(())
    }
    ///Transform a sequence of bytes into GameStats struct
    ///
    /// #Parameters:
    /// - msg: bytes sent by server containing informations in game positions
    ///
    /// #Return:
    /// New GameStats struct
    fn decode(msg: Bytes) -> Result<GameStats> {
        let left_y: f32 = f32::from_le_bytes(msg[0..4].try_into()?);
        let right_y: f32 = f32::from_le_bytes(msg[4..8].try_into()?);
        let ball_x: f32 = f32::from_le_bytes(msg[8..12].try_into()?);
        let ball_y: f32 = f32::from_le_bytes(msg[12..16].try_into()?);
        let player1_score: u8 = msg[24];
        let player2_score: u8 = msg[25];
        Ok(GameStats {
            left_y,
            right_y,
            ball_x,
            ball_y,
            player1_score,
            player2_score,
            winner: false,
        })
    }
    ///Send Game events to the server
    ///
    /// #Parameters:
    /// - ws_write: Writing part of the game websocket
    /// - receiver: End_game signal catcher
    /// - game_sender: Closer of the game websocket's reading part
    async fn send_game(
        mut ws_write: SplitSink<WsStream, Message>,
        mut receiver: mpsc::Receiver<u8>,
        game_sender: watch::Sender<bool>,
    ) -> Result<()> {
        let mut up: (bool, Instant, u128) = (false, std::time::Instant::now(), 0);
        let mut down: (bool, Instant, u128) = (false, std::time::Instant::now(), 0);
        let mut to_send = String::new();
        loop {
            if receiver.try_recv().is_ok() {
                break;
            }
            to_send.clear();
            if up.0 {
                to_send.insert(0, 'U');
            }
            if down.0 {
                to_send.insert(0, 'D');
            }
            if !to_send.is_empty() {
                let send_it = to_send.clone();
                ws_write.send(send_it.into()).await?;
            }
            if poll(Duration::from_millis(16))? {
                let event = event::read()?;
                if should_exit(&event)? {
                    game_sender.send(true)?;
                    break;
                } else if let Event::Key(key_event) = event {
                    match key_event.code {
                        KeyCode::Up => match key_event.kind {
                            KeyEventKind::Press => up = (true, std::time::Instant::now(), 150),
                            KeyEventKind::Repeat => up = (true, std::time::Instant::now(), 150),
                            KeyEventKind::Release => up = (false, std::time::Instant::now(), 150),
                        },
                        KeyCode::Down => match key_event.kind {
                            KeyEventKind::Press => down = (true, std::time::Instant::now(), 150),
                            KeyEventKind::Repeat => down = (true, std::time::Instant::now(), 150),
                            KeyEventKind::Release => down = (false, std::time::Instant::now(), 150),
                        },
                        _ => {
                            continue;
                        }
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
    ///Read incoming game state from server
    ///
    /// #Parameters:
    /// - ws_read: websocket's reading part
    /// - state_sender: game's state sender
    /// - socket_checker: end_game signal's receiver
    async fn read_socket(
        mut ws_read: SplitStream<WsStream>,
        state_sender: watch::Sender<(Option<Bytes>, Option<Utf8Bytes>)>,
        socket_checker: watch::Receiver<bool>,
    ) {
        loop {
            if let Some(msg) = ws_read.next().await {
                match msg {
                    Ok(Message::Binary(b)) => {
                        if state_sender.send((Some(b), None)).is_err() {
                            break;
                        }
                    }
                    Ok(Message::Text(s)) => {
                        if state_sender.send((None, Some(s))).is_err() {
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {}
                    _ => {}
                }
            }
            match socket_checker.has_changed() {
                Ok(false) => {}
                _ => break,
            }
        }
    }
}
