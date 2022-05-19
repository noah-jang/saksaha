use super::{System, SystemArgs};
use crate::blockchain::Blockchain;
use crate::config::Config;
use crate::db::DB;
use crate::p2p::host::Host;
use crate::p2p::host::HostArgs;
use crate::rpc::RPC;
use colored::Colorize;
use logger::{terr, tinfo};
use p2p_peer::PeerTable;
use std::sync::Arc;

impl System {
    pub(super) async fn start_routine(
        &self,
        sys_args: SystemArgs,
    ) -> Result<(), String> {
        tinfo!("saksaha", "system", "System is starting");

        let config = Config::new_from_sys_args(&sys_args);

        tinfo!("saksaha", "system", "Resolved config: {:?}", config);

        let p2p_peer_table = {
            let ps =
                PeerTable::init(config.p2p.p2p_peer_table_capacity).await?;

            Arc::new(ps)
        };

        let (rpc_socket, rpc_socket_addr) =
            match utils_net::bind_tcp_socket(config.rpc.rpc_port).await {
                Ok((socket, socket_addr)) => {
                    tinfo!(
                        "saksaha",
                        "system",
                        "Bound tcp socket for RPC, addr: {}",
                        socket_addr.to_string().yellow(),
                    );

                    (socket, socket_addr)
                }
                Err(err) => {
                    terr!(
                        "saksaha",
                        "system",
                        "Could not bind a tcp socket for RPC, err: {}",
                        err
                    );
                    return Err(err);
                }
            };

        let (p2p_socket, p2p_port) =
            match utils_net::bind_tcp_socket(config.p2p.p2p_port).await {
                Ok((socket, socket_addr)) => {
                    tinfo!(
                        "saksaha",
                        "system",
                        "Bound tcp socket for P2P host, addr: {}",
                        socket_addr.to_string().yellow(),
                    );

                    (socket, socket_addr.port())
                }
                Err(err) => {
                    terr!(
                        "saksaha",
                        "system",
                        "Could not bind a tcp socket for P2P Host, err: {}",
                        err
                    );
                    return Err(err);
                }
            };

        let p2p_host_args = HostArgs {
            disc_port: config.p2p.disc_port,
            disc_dial_interval: config.p2p.disc_dial_interval,
            disc_table_capacity: config.p2p.disc_table_capacity,
            disc_task_interval: config.p2p.disc_task_interval,
            disc_task_queue_capacity: config.p2p.disc_task_queue_capacity,
            p2p_task_interval: config.p2p.p2p_task_interval,
            p2p_task_queue_capacity: config.p2p.p2p_task_queue_capacity,
            p2p_dial_interval: config.p2p.p2p_dial_interval,
            p2p_socket,
            p2p_max_conn_count: config.p2p.p2p_max_conn_count,
            p2p_port,
            bootstrap_addrs: config.p2p.bootstrap_addrs,
            rpc_port: rpc_socket_addr.port(),
            secret: config.p2p.secret,
            public_key_str: config.p2p.public_key_str,
            p2p_peer_table,
        };

        let p2p_host = Host::init(p2p_host_args).await?;

        let rpc = RPC::init()?;

        let db = DB::init("db_ledger".to_string()).await?;

        let blockchain = Blockchain::init(db.ledger_db).await?;

        let system_thread = tokio::spawn(async move {
            tokio::join!(
                rpc.run(rpc_socket, rpc_socket_addr),
                p2p_host.run(),
                blockchain.run()
            );
        });

        tokio::select!(
            c = tokio::signal::ctrl_c() => {
                match c {
                    Ok(_) => {
                        tinfo!(
                            "sahsaha",
                            "system",
                            "ctrl+k is pressed.",
                        );

                        System::shutdown();
                    },
                    Err(err) => {
                        terr!(
                            "saksaha",
                            "system",
                            "Unexpected error while waiting for \
                                ctrl+p, err: {}",
                            err,
                        );

                        System::shutdown();
                    }
                }
            },
            _ = system_thread => {
            }
        );

        tinfo!(
            "saksaha",
            "system",
            "System main routine terminated. This is likely not what you \
            have expected."
        );

        Ok(())
    }
}
