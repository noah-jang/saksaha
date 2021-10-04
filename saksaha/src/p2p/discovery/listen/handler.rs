use crate::{common::{Error, Result}, err, p2p::{address::AddressBook, credential::Credential, discovery::whoareyou::{self, WhoAreYou, WhoAreYouAck}, peer::{Peer, PeerStatus}}};
use k256::ecdsa::{signature::Signer, Signature, SigningKey};
use logger::log;
use std::sync::Arc;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::Mutex};

pub enum HandleStatus<E> {
    WhoAreYouReceiveFail(E),

    WhoAreYouAckInitiateFail(E),

    PeerUpdateFail(E),

    Success,
}

pub struct Handler {
    address_book: Arc<AddressBook>,
    stream: TcpStream,
    peer: Arc<Mutex<Peer>>,
    credential: Arc<Credential>,
    peer_op_port: u16,
}

impl Handler {
    pub fn new(
        address_book: Arc<AddressBook>,
        stream: TcpStream,
        peer: Arc<Mutex<Peer>>,
        credential: Arc<Credential>,
        peer_op_port: u16,
    ) -> Handler {
        Handler {
            address_book,
            stream,
            peer,
            credential,
            peer_op_port,
        }
    }

    pub async fn run(&mut self) -> HandleStatus<Error> {
        let way = match self.receive_who_are_you().await {
            Ok(w) => w,
            Err(err) => return HandleStatus::WhoAreYouReceiveFail(err),
        };

        match self.initate_who_are_you_ack().await {
            Ok(_) => (),
            Err(err) => return HandleStatus::WhoAreYouAckInitiateFail(err),
        };

        match self.handle_succeed_who_are_you(way).await {
            Ok(_) => (),
            Err(err) => return HandleStatus::PeerUpdateFail(err),
        };

        HandleStatus::Success
    }

    pub async fn receive_who_are_you(&mut self) -> Result<WhoAreYou> {
        match WhoAreYou::parse(&mut self.stream).await {
            Ok(w) => Ok(w),
            Err(err) => {
                return err!("Error parsing who are you request, err: {}", err);
            }
        }
    }

    pub async fn initate_who_are_you_ack(&mut self) -> Result<()> {
        let secret_key = &self.credential.secret_key;
        let signing_key = SigningKey::from(secret_key);
        let sig: Signature = signing_key.sign(whoareyou::MESSAGE);

        let way_ack = WhoAreYouAck::new(
            sig,
            self.peer_op_port,
            self.credential.public_key_bytes,
        );

        let buf = match way_ack.to_bytes() {
            Ok(b) => b,
            Err(err) => {
                return err!(
                    "Error converting WhoAreYouAck to bytes, err: {}",
                    err
                );
            }
        };

        match &self.stream.write_all(&buf).await {
            Ok(_) => (),
            Err(err) => {
                return err!(
                    "Error sending the whoAreYou buffer, err: {}",
                    err
                );
            }
        }

        Ok(())
    }

    pub async fn handle_succeed_who_are_you(
        &self,
        way: WhoAreYou,
    ) -> Result<()> {


        // let mut peer = self.peer.lock().await;
        // peer.status = PeerStatus::Discovered;
        // peer.endpoint = addr.endpoint.to_owned();
        // peer.peer_id = way_ack.way.get_peer_id();
        // addr.status = Status::HandshakeSucceeded;

        // log!(DEBUG, "Successfully discovered a peer: {:?}", peer);

        // tokio::spawn(async move {
        //     println!("Start synchroize");
        // });

        Ok(())
    }
}
