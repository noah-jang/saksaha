use super::{
    active_calls::{ActiveCalls, Traffic},
    address::Address,
    ops::{
        whoareyou::{
            receiver::{WhoAreYouReceiver, WhoAreYouRecvError},
            WhoAreYouOperator,
        },
        Opcode,
    },
    table::Table,
    task_queue::{Task, TaskQueue},
    DiscState,
};
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
    way_operator: Arc<WhoAreYouOperator>,
}

impl Listener {
    pub fn new(
        disc_state: Arc<DiscState>,
        udp_socket: Arc<UdpSocket>,
        way_operator: Arc<WhoAreYouOperator>,
        task_queue: Arc<TaskQueue>,
    ) -> Listener {
        Listener {
            disc_state,
            task_queue,
            udp_socket,
            way_operator,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
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
        let way_operator = self.way_operator.clone();
        let task_queue = self.task_queue.clone();

        tokio::spawn(async move {
            loop {
                let mut buf = [0; 512];
                let (_, socket_addr) =
                    match udp_socket.recv_from(&mut buf).await {
                        Ok(res) => {
                            debug!(
                                "Accepted incoming request, len: {}, addr: {}",
                                res.0, res.1,
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
                    way_operator.clone(),
                    task_queue.clone(),
                    socket_addr,
                    &buf,
                )
                .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        error!(
                            "Error processing request, addr: {}, err: {}",
                            socket_addr, err
                        );
                    }
                };
            }
        });

        Ok(())
    }
}

struct Handler;

impl Handler {
    async fn run(
        disc_state: Arc<DiscState>,
        way_operator: Arc<WhoAreYouOperator>,
        task_queue: Arc<TaskQueue>,
        addr: SocketAddr,
        buf: &[u8],
    ) -> Result<(), String> {
        let addr = Address::from_socket_addr(addr);
        let len = buf.len();

        if len < 5 {
            return Err(format!("content too short, len: {}", len));
        }

        let opcode = {
            let c = Opcode::from(buf[4]);
            if c == Opcode::Undefined {
                return Err(format!("Undefined opcode, val: {}", buf[4]));
            }
            c
        };

        match opcode {
            Opcode::WhoAreYouSyn => {
                match way_operator
                    .receiver
                    .handle_who_are_you(addr.clone(), buf)
                    .await
                {
                    Ok(_) => (),
                    Err(err) => {
                        // match err {
                        //     WhoAreYouRecvError::MessageParseFail(_) => {
                        //     }
                        // }
                        error!("Request handle fail, err: {}", err);
                    }
                }
            }
            Opcode::WhoAreYouAck => {}
            Opcode::Undefined => {}
        };

        Ok(())
    }
}
