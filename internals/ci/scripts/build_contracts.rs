use crate::log;
use crate::script::Script;
use crate::scripts::BoxedError;
use clap::ArgMatches;
use lazy_static::lazy_static;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::process::Command as Cmd;

const NETWORK_CONTRACTS_PATH: &str = "source/saksaha/src/ncontracts";

lazy_static! {
    static ref CONTRACTS: Vec<DirEntry> = {
        let project_root = match std::env::var("PROJECT_ROOT") {
            Ok(r) => PathBuf::from(r),
            Err(err) => {
                log!(
                    "Env (PROJECT_ROOT) is not given. Did you run in \
                    from 'ci'?, err: {}",
                    err
                );

                std::process::exit(1);
            }
        };

        let contracts_dir = {
            let p = project_root.join(NETWORK_CONTRACTS_PATH);

            if p.exists() {
                match std::fs::read_dir(p) {
                    Ok(d) => d,
                    Err(err) => {
                        log!(
                            "Contract directory needs to be loaded, err: {}",
                            err
                        );

                        std::process::exit(1);
                    }
                }
            } else {
                log!("Contract directory does not exists");

                std::process::exit(1);
            }
        };

        let contracts: Vec<DirEntry> = contracts_dir
            .map(|f| f.expect("Contents in the directory should be read"))
            .collect();

        contracts
    };
    static ref EXAMPLE: u8 = 42;
}

pub(crate) struct BuildContracts;

impl Script for BuildContracts {
    fn handle_matches(_matches: &ArgMatches) -> Result<(), BoxedError> {
        for c in CONTRACTS.iter() {
            let path = match c.path().into_os_string().into_string() {
                Ok(p) => p,
                Err(err) => {
                    log!(
                        "path of a contract should be resolved, err: {}",
                        err.to_string_lossy()
                    );

                    std::process::exit(1);
                }
            };

            // let build_path = {
            //     let p =
            //         PathBuf::from(std::env::var("PROJECT_ROOT")?).join("build");

            //     match p.into_os_string().into_string() {
            //         Ok(p) => p,
            //         Err(err) => {
            //             log!("Build path should be resolved");

            //             std::process::exit(1);
            //         }
            //     }
            // };

            let module_wat = r#"
            (module
            (type $t0 (func (param i32) (result i32)))
            (func $add_one (export "add_one") (type $t0) (param $p0 i32) (result i32)
                get_local $p0
                i32.const 1
                i32.add))
            "#;

            let args = ["build", &path];

            let cmd = Cmd::new("wasm-pack")
                .args(args)
                .spawn()
                .expect("failed to run");

            cmd.wait_with_output().unwrap();
        }

        Ok(())
    }
}