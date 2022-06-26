use crate::{LedgerDB, LedgerError};
use sak_kv_db::WriteBatch;

impl LedgerDB {
    pub(crate) async fn get_ctr_data_by_ctr_addr(
        &self,
        ctr_addr: &String,
    ) -> Result<Option<Vec<u8>>, LedgerError> {
        let db = &self.kv_db.db_instance;

        let tx_hash = self
            .schema
            .get_tx_hash(db, ctr_addr)?
            .ok_or("ctr data does not exist")?;

        let ctr_data = self
            .schema
            .get_data(db, &tx_hash)?
            .ok_or("data does not exist")?;

        Ok(Some(ctr_data))
    }

    pub(crate) async fn get_ctr_state(
        &self,
        ctr_addr: &String,
        field_name: &String,
    ) -> Result<Option<Vec<u8>>, LedgerError> {
        let db = &self.kv_db.db_instance;

        let state_key = format!("{}:{}", ctr_addr, field_name);

        let ctr_state = self
            .schema
            .get_ctr_state(db, &state_key)?
            .ok_or("ctr state does not exist")?;

        Ok(Some(ctr_state))
    }

    pub(crate) async fn put_ctr_state(
        &self,
        contract_addr: &String,
        field_name: &String,
        field_value: &String,
    ) -> Result<String, LedgerError> {
        let db = &self.kv_db.db_instance;

        let mut batch = WriteBatch::default();

        let state_key = format!("{}:{}", contract_addr, field_name);

        self.schema.batch_put_ctr_state(
            db,
            &mut batch,
            &state_key,
            field_value,
        )?;

        db.write(batch)?;

        return Ok("".to_string().clone());
    }
}