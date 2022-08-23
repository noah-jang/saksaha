use crate::{
    credential::{self, Credential},
    db::EnvelopeDBSchema,
    EnvelopeError,
};
use log::{info, warn};
use sak_crypto::{
    PublicKey, SakKey, SecretKey, SigningKey, ToEncodedPoint, VerifyingKey,
};
use sak_kv_db::{KeyValueDatabase, Options};
use std::path::PathBuf;

pub(crate) const APP_NAME: &str = "envelope";

pub(crate) struct EnvelopeDB {
    pub(crate) schema: EnvelopeDBSchema,
}

impl EnvelopeDB {
    pub(crate) async fn init(
        // app_prefix: &String,
        acc_addr: &String,
    ) -> Result<EnvelopeDB, EnvelopeError> {
        let envelope_db_path = {
            let db_path = Self::get_db_path(acc_addr)?;

            if !db_path.exists() {
                std::fs::create_dir_all(db_path.clone())?;
            }

            db_path
        };

        let options = {
            let mut o = Options::default();
            o.create_missing_column_families(true);
            o.create_if_missing(true);

            o
        };

        let kv_db = match KeyValueDatabase::new(
            envelope_db_path,
            options,
            EnvelopeDBSchema::make_cf_descriptors(),
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

        let schema = EnvelopeDBSchema::new(kv_db.db_instance);

        let database = EnvelopeDB { schema };

        info!("Initialized Database");

        Ok(database)
    }

    pub(crate) async fn register_user(
        &self,
        credential: &Credential,
    ) -> Result<(), EnvelopeError> {
        log::info!("Register User: {:?}", credential.acc_addr);

        self.schema
            .put_user_data(
                &credential.secret,
                &credential.public_key,
                &credential.signature,
                &credential.acc_addr,
            )
            .await?;

        Ok(())
    }

    pub fn get_db_path(app_prefix: &str) -> Result<PathBuf, EnvelopeError> {
        let app_path = sak_fs::get_app_root_path(APP_NAME)?.join(app_prefix);

        let db_path = app_path.join("db");

        Ok(db_path)
    }
}