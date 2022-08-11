use crate::{Msg, TrptError, UpgradedP2PCodec};
use futures::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

pub struct SendReceipt {
    __created_by_sending: bool,
}

pub struct UpgradedConn {
    socket_addr: SocketAddr,
    conn_id: String,
    socket: Framed<TcpStream, UpgradedP2PCodec>,
}

impl UpgradedConn {
    pub async fn init(
        socket_addr: SocketAddr,
        socket: Framed<TcpStream, UpgradedP2PCodec>,
        conn_id: String,
        _is_initiator: bool,
    ) -> UpgradedConn {
        let upgraded_conn = UpgradedConn {
            socket_addr,
            socket,
            conn_id,
        };

        upgraded_conn
    }

    pub async fn send(&mut self, msg: Msg) -> Result<SendReceipt, TrptError> {
        println!("send msg!, msg: {}, conn id: {}", msg, self.conn_id);

        self.socket.send(msg).await?;

        let receipt = SendReceipt {
            __created_by_sending: true,
        };

        Ok(receipt)
    }

    pub async fn next_msg(&mut self) -> Option<Result<Msg, TrptError>> {
        println!("recv msg!, conn id: {}", self.conn_id);

        let msg = self.socket.next().await;

        msg
    }
}
