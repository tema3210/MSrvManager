use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use futures::stream::{Stream, StreamExt};
use tokio::task::JoinHandle;
use std::pin::Pin;
use std::sync::atomic::AtomicI32;

use actix::prelude::*;

#[derive(Message,Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RconUp {
    pub rcon: Rcon
}

#[derive(Message,Debug)]
#[rtype(result = "()")]
pub struct RconDown;

#[derive(Message,Debug)]
#[rtype(result = "anyhow::Result<()>")]
pub struct RconMessage {
    pub cmd: String
}

#[derive(Message)]
#[rtype(result = "anyhow::Result<RconStream>")]
pub struct RconSubscription;


#[derive(Debug, Clone)]
pub enum RconOutput {
    CommandResponse(String),
    Error(String),
    ConnectionClosed,
}

#[derive(Debug)]
pub struct Rcon {
    command_sender: mpsc::Sender<String>, // Unicast channel
    output_receiver: broadcast::Receiver<RconOutput>, // Broadcast channel

    outgoing_task: JoinHandle<()>,
    incoming_task: JoinHandle<()>,
}

impl Drop for Rcon {
    fn drop(&mut self) {
        self.outgoing_task.abort();
        self.incoming_task.abort();
    }
}

pub type RconStream = Pin<Box<dyn Stream<Item = RconOutput> + Send + 'static>>;

#[derive(Debug, PartialEq, Eq)]
struct RconMessageType(i32);

impl RconMessageType {
    const SERVERDATA_AUTH: Self = Self(3);
    const SERVERDATA_EXECCOMMAND: Self = Self(2);
    const SERVERDATA_AUTH_RESPONSE: Self = Self(2);
    // const SERVERDATA_RESPONSE_VALUE: Self = Self(0);
}

impl Rcon {

    const ORDER: std::sync::atomic::Ordering = std::sync::atomic::Ordering::SeqCst;

    fn build_packet(request_id: i32, packet_type: RconMessageType, body: String) -> Result<Vec<u8>> {
        let mut packet = Vec::new();
        let size = (body.len() + 10) as i32;
        packet.extend_from_slice(&size.to_le_bytes());
        packet.extend_from_slice(&request_id.to_le_bytes());
        packet.extend_from_slice(&packet_type.0.to_le_bytes());
        packet.extend_from_slice(body.as_bytes());
        packet.push(0);
        packet.push(0);

        Ok(packet)
    }
}

impl Rcon {

    pub async fn new(port: u16, password: String) -> Result<Self> {
        let address = format!("127.0.0.1:{}", port);
        let mut stream = TcpStream::connect(address).await?;

        let auth_packet = Self::build_packet(0, RconMessageType::SERVERDATA_AUTH, password)?;

        stream.write_all(&auth_packet).await?;
        stream.flush().await?;

        let mut response = [0; 14];
        stream.read_exact(&mut response).await?;

        //rcon protocol authentication

        // let received_length = i32::from_le_bytes([response[0], response[1], response[2], response[3]]);
        let received_id = i32::from_le_bytes([response[4], response[5], response[6], response[7]]);
        let response_type = RconMessageType(i32::from_le_bytes([response[8], response[9], response[10], response[11]]));

        let request_id = AtomicI32::new(1);
        if received_id == 0 && response_type == RconMessageType::SERVERDATA_AUTH_RESPONSE {
            let (reader,writer) = stream.into_split();

            let (publish,subscribe) = broadcast::channel::<RconOutput>(100); //Initialize broadcast channel
            let (mpsc_sender, mut msg_recv) = mpsc::channel::<String>(100); // Initialize unicast channel

            let outgoing_task = tokio::spawn(async move {
                let mut stream = writer;
                while let Some(msg) = msg_recv.recv().await {
                    let packet = Self::build_packet(
                        request_id.load(Self::ORDER),
                        RconMessageType::SERVERDATA_EXECCOMMAND, 
                        msg
                    ).unwrap();
                    request_id.fetch_add(1,Self::ORDER);
                    stream.write_all(&packet).await.unwrap();
                    stream.flush().await.unwrap();
                }
            });

            let incoming_task = tokio::spawn(async move {
                let mut stream = reader;
                loop {
                    let mut size = [0; 4];
                    stream.read_exact(&mut size).await.unwrap();
                    let size = i32::from_le_bytes(size);

                    let mut response = vec![0; size as usize];
                    stream.read_exact(&mut response).await.unwrap();
                    // here we have 4bytes PID, 4bytes Type, Payload, 1 byte null terminator
                    let payload = &response[8..(response.len() - 2)];

                    let response = String::from_utf8(payload.to_vec()).unwrap();

                    if response.starts_with("Error") {
                        publish.send(RconOutput::Error(response)).unwrap();
                    } else {
                        publish.send(RconOutput::CommandResponse(response)).unwrap();
                    }
                }
            });

            log::info!("rcon connection established: {}", port);
            Ok(Self {
                output_receiver: subscribe,
                command_sender: mpsc_sender,
                outgoing_task,
                incoming_task,
            })
        } else {
            anyhow::bail!("Authentication failed {}, {:?}", received_id, response_type);
        }
    }

    pub fn send(&self, cmd: String) -> Result<()> {
        self.command_sender.try_send(cmd)?;
        
        Ok(())
    }

    pub fn output_stream(&self) -> impl Stream<Item = RconOutput> + 'static {
        let receiver = self.output_receiver.resubscribe(); // Each consumer gets a unique receiver
        let stream = tokio_stream::wrappers::BroadcastStream::new(receiver)
            .filter_map(|result| async {
                result.ok() // Filter out any errors from the stream
            });
        stream
    }
}