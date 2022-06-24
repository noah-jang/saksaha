use super::columns;
use crate::{columns::LedgerDBColumnFamily, BoxedError};
use log::info;
use sak_fs::FS;
use sak_kv_db::{ColumnFamilyDescriptor, KeyValueDatabase, Options};

pub(crate) struct Database {
    pub(crate) ledger_db: KeyValueDatabase<LedgerDBColumnFamily>,
}

impl Database {
    pub async fn init(app_prefix: &String) -> Result<Database, BoxedError> {
        let ledger_db = init_ledger_db(&app_prefix)?;

        let database = Database { ledger_db };

        info!("Initialized Database");

        Ok(database)
    }
}

pub(crate) fn init_ledger_db(
    app_prefix: &String,
) -> Result<KeyValueDatabase<LedgerDBColumnFamily>, BoxedError> {
    let ledger_db_path = {
        let app_path = FS::create_or_get_app_path(app_prefix)?;
        let db_path = { app_path.join("db").join("ledger") };

        db_path
    };

    let options = {
        let mut o = Options::default();
        o.create_missing_column_families(true);
        o.create_if_missing(true);

        o
    };

    // let cf_descriptors = make_ledger_db_cf_descriptors();

    let cf = LedgerDBColumnFamily::new();

    let kv_db = match KeyValueDatabase::new(ledger_db_path, options, cf) {
        Ok(d) => d,
        Err(err) => {
            return Err(format!(
                "Error initializing key value database, err: {}",
                err
            )
            .into());
        }
    };

    Ok(kv_db)
}

// fn make_ledger_db_cf_descriptors() -> Vec<ColumnFamilyDescriptor> {
//     let columns = vec![
//         (columns::TX_HASH, Options::default()),
//         (columns::PI, Options::default()),
//         (columns::SIG_VEC, Options::default()),
//         (columns::CREATED_AT, Options::default()),
//         (columns::DATA, Options::default()),
//         (columns::CONTRACT_ADDR, Options::default()),
//         (columns::VALIDATOR_SIG, Options::default()),
//         (columns::TX_HASHES, Options::default()),
//         (columns::WITNESS_SIGS, Options::default()),
//         (columns::HEIGHT, Options::default()),
//         (columns::BLOCK_HASH, Options::default()),
//         (columns::CONTRACT_STATE, Options::default()),
//     ];

//     let cf = columns
//         .into_iter()
//         .map(|(col_name, options)| {
//             ColumnFamilyDescriptor::new(col_name, options)
//         })
//         .collect();

//     cf
// }
