pub mod address;
mod calls;
pub mod dial_scheduler;
pub mod error;
pub mod listener;
pub mod msg;
mod ops;
pub mod queue;
mod table;

use self::{
    calls::ConnectionPool, dial_scheduler::DialScheduler, listener::Listener,
    queue::TaskQueue, table::Table,
};
use crate::{
    identity::Identity,
    v1::{address::Address, queue::Task},
};
use log::{info, warn};
use std::sync::Arc;

pub struct Disc {
    task_queue: Arc<TaskQueue>,
    calls: Arc<ConnectionPool>,
    table: Arc<Table>,
    id: Arc<Box<dyn Identity>>,
}

impl Disc {
    pub fn new(id: Arc<Box<dyn Identity>>) -> Disc {
        let table = Arc::new(Table::new());
        let task_queue = TaskQueue::new(table.clone(), id.clone());
        let calls = ConnectionPool::new();

        Disc {
            task_queue: Arc::new(task_queue),
            calls: Arc::new(calls),
            table,
            id,
        }
    }

    pub async fn start(
        &self,
        port: Option<u16>,
        p2p_listener_port: u16,
        bootstrap_urls: Option<Vec<String>>,
        default_bootstrap_urls: &str,
    ) -> Result<(), String> {
        let listener = Listener::new();
        let listener_port = match listener
            .start(port, p2p_listener_port, self.calls.clone())
            .await
        {
            Ok(port) => port,
            Err(err) => return Err(err),
        };

        self.task_queue.set();
        self.enqueue_initial_tasks(bootstrap_urls, default_bootstrap_urls)
            .await;

        let dial_scheduler = DialScheduler::new();
        let _ = dial_scheduler.start(
            self.id.clone(),
            listener_port,
            p2p_listener_port,
            self.table.clone(),
            self.task_queue.clone(),
        );

        self.task_queue.run_loop();

        Ok(())
    }

    pub async fn enqueue_initial_tasks(
        &self,
        bootstrap_urls: Option<Vec<String>>,
        default_bootstrap_urls: &str,
    ) {
        let bootstrap_urls = match bootstrap_urls {
            Some(u) => u,
            None => Vec::new(),
        };

        let default_bootstrap_urls: Vec<String> = default_bootstrap_urls
            .lines()
            .map(|l| l.to_string())
            .collect();

        let urls = [bootstrap_urls, default_bootstrap_urls].concat();

        info!("*********************************************************");
        info!("* Discovery table bootstrapped");

        let mut count = 0;
        {
            for url in urls {
                let addr = match Address::parse(url.clone()) {
                    Ok(n) => {
                        count += 1;
                        n
                    }
                    Err(err) => {
                        warn!(
                            "Discarding url failed to parse, url: {}, \
                            err: {:?}",
                            url.clone(),
                            err,
                        );

                        continue;
                    }
                };

                info!("* [{}] {}", count, addr.short_url());

                let task = Task::InitiateWhoAreYou(self.table.clone(), addr);
                match self.task_queue.push(task).await {
                    Ok(_) => (),
                    Err(err) => {
                        warn!("Couldn't enque new task, err: {}", err);
                    }
                };
            }
        };

        info!("* bootstrapped node count: {}", count);
        info!("*********************************************************");
    }
}