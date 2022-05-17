use super::node::Node;
use crate::{NodeStatus, NodeValue};
use logger::{terr, tinfo};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex,
};

const PEER_TABLE_CAPACITY: usize = 50;

pub struct PeerTable {
    peers: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
    peers_map: Arc<Mutex<HashMap<String, Arc<Mutex<Node>>>>>,
    node_retreival_tx: Arc<UnboundedSender<Arc<Mutex<Node>>>>,
}

impl PeerTable {
    pub async fn init(
        peer_table_capacity: Option<u16>,
    ) -> Result<PeerTable, String> {
        let capacity = match peer_table_capacity {
            Some(c) => c.into(),
            None => PEER_TABLE_CAPACITY,
        };

        let node_retreival_tx = {
            let (tx, rx) = mpsc::unbounded_channel();

            let retrival_routine = RetrievalRoutine {};
            tokio::spawn(async move {
                retrival_routine.run(rx).await;
            });

            Arc::new(tx)
        };

        let peers = {
            let mut v = Vec::with_capacity(capacity);

            for _ in 0..capacity {
                let n = Node {
                    value: NodeValue::Empty,
                    status: NodeStatus::Available,
                };

                v.push(Arc::new(Mutex::new(n)));
            }

            Arc::new(Mutex::new(v))
        };

        let peers_map = {
            let m = HashMap::new();

            Arc::new(Mutex::new(m))
        };

        tinfo!(
            "peer",
            "",
            "Initializing peer table, capacity: {}",
            capacity
        );

        let ps = PeerTable {
            peers_map,
            peers,
            node_retreival_tx,
        };

        Ok(ps)
    }

    pub async fn get(
        &self,
        public_key: &String,
    ) -> Option<Result<NodeGuard, String>> {
        let peers_map = self.peers_map.clone();
        let peers_map_lock = peers_map.lock().await;

        match peers_map_lock.get(public_key) {
            Some(n) => {
                let node_lock = n.lock().await;
                if !node_lock.is_used() {
                    let node_guard = NodeGuard {
                        node: n.clone(),
                        node_retrieval_tx: self.node_retreival_tx.clone(),
                    };

                    if let NodeValue::Valued(v) = &node_lock.value {
                        if let Some(v) = &v.transport.addr_guard {
                            let known_addr = v.get_known_addr().await;

                            println!(
                                "peer get(): known_at: {}, x: {}",
                                known_addr.known_at, v.x
                            );
                        }
                    }

                    return Some(Ok(node_guard));
                } else {
                    return Some(Err(format!(
                        "Peer node is already being used"
                    )));
                }
            }
            None => {
                println!("peer get(): None");
                return None;
            }
        };
    }

    pub async fn reserve(
        &self,
        public_key: &String,
    ) -> Result<NodeGuard, String> {
        println!("peer reserve()");

        let peers = self.peers.lock().await;
        for node in peers.iter() {
            let mut node_lock = match node.try_lock() {
                Ok(n) => n,
                Err(_) => {
                    continue;
                }
            };

            if node_lock.is_empty() && !node_lock.is_used() {
                let node_guard = NodeGuard {
                    node: node.clone(),
                    node_retrieval_tx: self.node_retreival_tx.clone(),
                };

                let mut peers_map = self.peers_map.lock().await;
                peers_map.insert(public_key.clone(), node.clone());

                node_lock.status = NodeStatus::Used;

                return Ok(node_guard);
            }
        }

        Err(format!("Could not reserve a peer node"))
    }

    pub async fn print_all_nodes(&self) -> u16 {
        let peers = self.peers.lock().await;

        for (idx, node) in peers.iter().enumerate() {
            if let Ok(node_lock) = node.try_lock() {
                let a = &node_lock.value;
                match a {
                    NodeValue::Valued(p) => {
                        println!(
                            "peer table [{}] - p2p_port: {}",
                            idx, p.transport.p2p_port
                        );
                        return p.transport.p2p_port;
                    }
                    _ => {
                        println!("peer table [{}] - empty", idx);
                    }
                };
            } else {
                println!("peer table [{}] - locked", idx,);
            }
        }
        return 0;
    }

    pub async fn print_all_mapped_nodes(&self) {
        let peers_map = self.peers_map.lock().await;

        let len = peers_map.len();
        println!("Peer map length: {}", len);

        for (idx, node) in peers_map.values().into_iter().enumerate() {
            if let Ok(node_lock) = node.try_lock() {
                let a = &node_lock.value;
                match a {
                    NodeValue::Valued(p) => {
                        println!(
                            "peer table [{}] - p2p_port: {}",
                            idx, p.transport.p2p_port
                        );
                    }
                    _ => (),
                };
            }
        }
    }
}

pub struct NodeGuard {
    pub node: Arc<Mutex<Node>>,
    pub node_retrieval_tx: Arc<UnboundedSender<Arc<Mutex<Node>>>>,
}

impl Drop for NodeGuard {
    fn drop(&mut self) {
        match self.node_retrieval_tx.send(self.node.clone()) {
            Ok(_) => (),
            Err(err) => {
                terr!(
                    "p2p_peer",
                    "",
                    "Cannot retrieve peer node after use, err: {}",
                    err
                );
            }
        }
    }
}

pub struct RetrievalRoutine;

impl RetrievalRoutine {
    pub async fn run(&self, mut node_rx: UnboundedReceiver<Arc<Mutex<Node>>>) {
        loop {
            let node = match node_rx.recv().await {
                Some(n) => n,
                None => {
                    terr!(
                        "p2p_peer",
                        "table",
                        "All node guard senders have been closed. \
                        Something is critically wrong",
                    );

                    return;
                }
            };

            let mut n = node.lock().await;
            n.status = NodeStatus::Available;
        }
    }
}