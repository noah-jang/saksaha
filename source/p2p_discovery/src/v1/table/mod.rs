mod iter;
mod node;

pub(crate) use self::node::*;
pub use iter::*;
use p2p_identity::addr::Addr;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver, UnboundedSender},
    Mutex, MutexGuard,
};

// const DISC_TABLE_CAPACITY: usize = 100;
const DISC_TABLE_CAPACITY: usize = 5;

/// TODO Table shall have Kademlia flavored buckets
pub(crate) struct Table {
    addr_map: Arc<Mutex<HashMap<String, Arc<Mutex<Node>>>>>,
    addrs: Arc<Mutex<Vec<Arc<Mutex<Node>>>>>,
    known_addrs_tx: Arc<UnboundedSender<Arc<Mutex<Node>>>>,
    known_addrs_rx: Arc<Mutex<UnboundedReceiver<Arc<Mutex<Node>>>>>,
}

impl Table {
    pub(crate) async fn init(
        disc_table_capacity: Option<u16>,
    ) -> Result<Table, String> {
        let addr_map = {
            let m = HashMap::new();
            Arc::new(Mutex::new(m))
        };

        let disc_table_capacity = match disc_table_capacity {
            Some(c) => c.into(),
            None => DISC_TABLE_CAPACITY,
        };

        let addrs = {
            let mut v = Vec::with_capacity(disc_table_capacity);

            for _ in 0..disc_table_capacity {
                let n = Node {
                    value: NodeValue::Empty,
                };

                let n = Arc::new(Mutex::new(n));
                v.push(n);
            }

            Arc::new(Mutex::new(v))
        };

        let (known_addrs_tx, known_addrs_rx) = {
            let (tx, rx) = mpsc::unbounded_channel();
            (Arc::new(tx), Arc::new(Mutex::new(rx)))
        };

        let table = Table {
            addr_map,
            addrs,
            known_addrs_tx,
            known_addrs_rx,
        };

        Ok(table)
    }

    pub(crate) async fn upsert(
        &self,
        addr: Addr,
        node_status: NodeStatus,
    ) -> Result<Arc<Mutex<Node>>, String> {
        let endpoint = addr.disc_endpoint();

        let addr_map = self.addr_map.clone();
        let mut addr_map_guard = addr_map.lock().await;

        // if map already had the address node
        if let Some(n) = addr_map_guard.get(&endpoint) {
            let mut node_lock = n.lock().await;
            node_lock.value = NodeValue::Valued(NodeValueInner {
                addr: addr.clone(),
                status: node_status,
            });

            return Ok(n.clone());
        } else {
            // When the map doesn't have a node associated with the endpoint
            let addrs_lock = self.addrs.lock().await;

            match find_empty_node(&addrs_lock) {
                Some(n) => {
                    addr_map_guard.insert(endpoint, n.clone());

                    let mut node_lock = n.lock().await;
                    node_lock.value = NodeValue::Valued(NodeValueInner {
                        addr: addr.clone(),
                        status: node_status,
                    });

                    return Ok(n.clone());
                }
                None => {
                    return Err(format!("Every node is currently locked"));
                }
            };
        }
    }

    pub(crate) async fn add_known_node(
        &self,
        node: Arc<Mutex<Node>>,
    ) -> Result<(), String> {
        println!("adding known node");

        self.print_all_nodes().await;

        match self.known_addrs_tx.send(node) {
            Ok(_) => Ok(()),
            Err(err) => Err(format!(
                "Couldn't push known node into queue, err: {}",
                err
            )),
        }
    }

    pub(crate) async fn print_all_nodes(&self) {
        let addrs = self.addrs.lock().await;
        for (idx, node) in addrs.iter().enumerate() {
            let node_lock = node.lock().await;
            println!("addr table elements [{}] - {:?}", idx, node_lock);
        }
    }

    pub(crate) fn iter(&self) -> AddrsIterator {
        AddrsIterator::init(
            self.known_addrs_tx.clone(),
            self.known_addrs_rx.clone(),
        )
    }
}

fn find_empty_node(
    addrs_lock: &MutexGuard<Vec<Arc<Mutex<Node>>>>,
) -> Option<Arc<Mutex<Node>>> {
    for node in addrs_lock.iter() {
        match node.try_lock() {
            Ok(n) => match &n.value {
                NodeValue::Empty => {
                    return Some(node.clone());
                }
                _ => continue,
            },
            Err(_) => continue,
        };
    }

    return None;
}
