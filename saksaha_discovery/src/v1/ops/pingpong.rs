use crate::{task::TaskResult, v1::table::TableNode};
use log::{debug};
use tokio::net::TcpStream;

pub struct PingPong;

impl PingPong {
    pub async fn ping(addr: Address) -> TaskResult {
        let endpoint = format!("{}:{}", addr.ip, addr.disc_port);

        let mut stream = match TcpStream::connect(endpoint.to_owned()).await {
            Ok(s) => {
                debug!(
                    "Successfully connected to endpoint, {}",
                    endpoint
                );
                s
            }
            Err(err) => {
                return TaskResult::Retriable;
                // let msg = format!(
                //     "Cannot connect to peer.ip: {}, port: {}, err: {}",
                //     peer.ip, peer.disc_port, err
                // );
                // let err = Error::new_default(msg);

                // peer.record_fail();

                // return HandleStatus::ConnectionFail(err);
            }
        };

        TaskResult::Success
    }
}