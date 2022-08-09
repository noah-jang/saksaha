use crate::{db::DBSchema, WalletError};
use log::{info, warn};
use sak_kv_db::{KeyValueDatabase, Options};

pub(crate) struct WalletDB {
    pub(crate) schema: DBSchema,
}

impl WalletDB {
    pub(crate) async fn init(
        app_prefix: &String,
    ) -> Result<WalletDB, WalletError> {
        let envelope_db_path = {
            let app_path = sak_fs::create_or_get_app_path_evl(app_prefix)?;
            let db_path = { app_path.join("db") };

            db_path
        };

        info!("Envelope db path: {:?}", envelope_db_path);

        let options = {
            let mut o = Options::default();
            o.create_missing_column_families(true);
            o.create_if_missing(true);

            o
        };

        let kv_db = match KeyValueDatabase::new(
            envelope_db_path,
            options,
            DBSchema::make_cf_descriptors(),
        ) {
            Ok(d) => d,
            Err(err) => {
                return Err(format!(
                    "Error initializing key value database, err: {}",
                    err
                )
                .into());
            }
        };

        let schema = DBSchema::new(kv_db.db_instance);

        let wallet_db = WalletDB { schema };

        Ok(wallet_db)
    }
}