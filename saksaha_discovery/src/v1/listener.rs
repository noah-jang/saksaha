use super::{DiscState, active_calls::{ActiveCalls, Traffic}, ops::{whoareyou::receiver::WhoAreYouReceiver, Opcode}, table::Table, task_queue::TaskQueue};
use log::{debug, error, info, warn};
use std::{convert::TryInto, net::SocketAddr, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
};

#[derive(Error, Debug)]
pub enum ListenerError {
    #[error("Already has active call with endpoint, {0}")]
    CallAlreadyInProgress(String),
}

pub struct Listener {
    disc_state: Arc<DiscState>,
    task_queue: Arc<TaskQueue>,
    udp_socket: Arc<UdpSocket>,
    way_receiver: Arc<WhoAreYouReceiver>,
}

impl Listener {
    pub fn new(
        disc_state: Arc<DiscState>,
        udp_socket: Arc<UdpSocket>,
        way_receiver: Arc<WhoAreYouReceiver>,
        task_queue: Arc<TaskQueue>,
    ) -> Listener {
        Listener {
            disc_state,
            udp_socket,
            way_receiver,
            task_queue,
        }
    }

    pub async fn start(&self, my_p2p_port: u16) -> Result<(), String> {
        match self.run_loop() {
            Ok(_) => (),
            Err(err) => {
                return Err(format!("Couldn't start loop, err: {}", err));
            }
        };

        Ok(())
    }

    pub fn run_loop(&self) -> Result<(), String> {
        let disc_state = self.disc_state.clone();
        let udp_socket = self.udp_socket.clone();
        let task_queue = self.task_queue.clone();
        let way_receiver = self.way_receiver.clone();

        tokio::spawn(async move {
            loop {
                let mut buf = [0; 1024];
                let (len, addr) = match udp_socket.recv_from(&mut buf).await {
                    Ok(res) => {
                        debug!(
                            "Accepted incoming request, len: {}, addr: {}",
                            res.0, res.1
                        );
                        res
                    }
                    Err(err) => {
                        warn!("Error accepting request, err: {}", err);
                        continue;
                    }
                };

                match Handler::run(
                    disc_state.clone(),
                    task_queue.clone(),
                    way_receiver.clone(),
                    addr,
                    &buf,
                )
                .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        error!(
                            "Error processing request, addr: {}, err: {}",
                            addr, err
                        );
                    }
                }

                // let peer_ip = match stream.peer_addr() {
                //     Ok(a) => a.ip().to_string(),
                //     Err(err) => {
                //         warn!("Cannot retrieve peer addr, err: {}", err,);

                //         continue;
                //     }
                // };

                // if active_calls.contain(&peer_ip).await {
                //     debug!("Already on phone, dropping conn, {}", peer_ip);

                //     continue;
                // } else {
                //     active_calls
                //         .insert(peer_ip.clone(), Traffic::InBound)
                //         .await;
                // }

                // Routine::run_handler(
                //     stream,
                //     peer_ip.clone(),
                //     // credential.clone(),
                //     peer_op_port,
                //     // task_queue.clone(),
                //     active_calls.clone(),
                //     // peer_store.clone(),
                // );
            }
        });

        Ok(())
    }
}

struct Handler;

impl Handler {
    async fn run(
        disc_state: Arc<DiscState>,
        task_queue: Arc<TaskQueue>,
        way_receiver: Arc<WhoAreYouReceiver>,
        addr: SocketAddr,
        buf: &[u8],
    ) -> Result<(), String> {
        let endpoint = get_endpoint(addr);
        let len = buf.len();

        if len < 5 {
            return Err(format!("content too short, len: {}", len));
        }

        let len: usize = {
            let mut len_buf: [u8; 4] = [0; 4];
            len_buf.copy_from_slice(&buf[..4]);

            let len = u32::from_le_bytes(len_buf);
            let len: usize = match len.try_into() {
                Ok(l) => l,
                Err(err) => {
                    return Err(format!(
                        "Error converting size into usize, len: {}",
                        len
                    ));
                }
            };
            len
        };

        let opcode = {
            let c = Opcode::from(buf[4]);
            if c == Opcode::Undefined {
                return Err(format!("Undefined opcode, val: {}", buf[4]));
            }
            c
        };

        match opcode {
            Opcode::WhoAreYou => {}
            Opcode::WhoAreYouAck => {}
            Opcode::Undefined => {}
        };

        Ok(())
    }
}

struct Routine {}

impl Routine {
    pub fn new() -> Routine {
        Routine {}
    }

    pub fn run(
        &self,
        tcp_listener: TcpListener,
        peer_op_port: u16,
        // peer_store: Arc<PeerStore>,
        // credential: Arc<Credential>,
        // task_queue: Arc<TaskQueue>,
        active_calls: Arc<ActiveCalls>,
    ) {
        tokio::spawn(async move {
            loop {

                // let (stream, addr) = match tcp_listener.accept().await {
                //     Ok(res) => {
                //         debug!("Accepted incoming request, addr: {}", res.1);
                //         res
                //     }
                //     Err(err) => {
                //         warn!("Error accepting request, err: {}", err);
                //         continue;
                //     }
                // };

                // println!("4, addr: {:?}", addr);

                // let peer_ip = match stream.peer_addr() {
                //     Ok(a) => a.ip().to_string(),
                //     Err(err) => {
                //         warn!("Cannot retrieve peer addr, err: {}", err,);

                //         continue;
                //     }
                // };

                // if active_calls.contain(&peer_ip).await {
                //     debug!("Already on phone, dropping conn, {}", peer_ip);

                //     continue;
                // } else {
                //     active_calls
                //         .insert(peer_ip.clone(), Traffic::InBound)
                //         .await;
                // }

                // Routine::run_handler(
                //     stream,
                //     peer_ip.clone(),
                //     // credential.clone(),
                //     peer_op_port,
                //     // task_queue.clone(),
                //     active_calls.clone(),
                //     // peer_store.clone(),
                // );
            }
        });
    }

    pub fn run_handler(
        stream: TcpStream,
        peer_ip: String,
        // credential: Arc<Credential>,
        peer_op_port: u16,
        // task_queue: Arc<TaskQueue>,
        active_calls: Arc<ActiveCalls>,
        // peer_store: Arc<PeerStore>,
    ) {
        // let mut handler = Handler::new(
        //     stream,
        //     // peer_store,
        //     // credential,
        //     peer_op_port,
        // );

        // tokio::spawn(async move {
        //     match handler.run().await {
        //         Ok(_) => (),
        //         Err(err) => match err {
        //             HandleError::NoAvailablePeerSlot => {
        //                 debug!("No available peer slot, sleeping");

        //                 tokio::time::sleep(Duration::from_millis(1000)).await;
        //             }
        //             HandleError::PeerAlreadyTalking(endpoint) => {
        //                 debug!(
        //                     "Peer might be in talk already, endpoint: {}",
        //                     endpoint,
        //                 );
        //             }
        //             HandleError::AddressAcquireFail(err) => {
        //                 warn!(
        //                     "Cannot acquire address of \
        //                             incoming connection, err: {}",
        //                     err
        //                 );
        //             }
        //             HandleError::Success => (),
        //             HandleError::WhoAreYouReceiveFail(err) => {
        //                 warn!(
        //                     "Disc listen failed receiving \
        //                             who are you, err: {}",
        //                     err
        //                 );
        //             }
        //             HandleError::WhoAreYouAckInitiateFail(err) => {
        //                 warn!(
        //                     "Disc listen failed initiating \
        //                             who are you ack, err: {}",
        //                     err
        //                 );
        //             }
        //             HandleError::PeerUpdateFail(err) => {
        //                 warn!("Disc listen failed updating peer, err: {}", err);
        //             }
        //         },
        //     };

        //     active_calls.remove(&peer_ip).await;
        // });
    }
}

fn get_endpoint(addr: SocketAddr) -> String {
    format!("{}:{}", addr.ip(), addr.port())
}
