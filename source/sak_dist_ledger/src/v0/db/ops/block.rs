use crate::{LedgerDB, LedgerError};
use sak_kv_db::WriteBatch;
use sak_types::{Block, Tx};

impl LedgerDB {
    pub(crate) fn get_block(
        &self,
        block_hash: &String,
    ) -> Result<Option<Block>, LedgerError> {
        let db = &self.kv_db.db_instance;

        let validator_sig = self.schema.get_validator_sig(db, &block_hash)?;

        let tx_hashes = self.schema.get_tx_hashes(db, &block_hash)?;

        let witness_sigs = self.schema.get_witness_sigs(db, &block_hash)?;

        let created_at = self.schema.get_created_at(db, &block_hash)?;

        let block_height = self.schema.get_block_height(db, &block_hash)?;

        match (
            validator_sig,
            tx_hashes,
            witness_sigs,
            created_at,
            block_height,
        ) {
            (Some(vs), Some(th), Some(ws), Some(ca), Some(bh)) => {
                let b = Block::new(vs, th, ws, ca, bh);
                return Ok(Some(b));
            }
            (None, None, None, None, None) => {
                return Ok(None);
            }
            _ => {
                return Err(format!(
                    "Block is corrupted. Some data is missing, block_hash: {}",
                    block_hash,
                )
                .into());
            }
        }
    }

    pub(crate) fn get_block_hash_by_height(
        &self,
        block_height: &String,
    ) -> Result<Option<String>, LedgerError> {
        let db = &self.kv_db.db_instance;

        self.schema.get_block_hash(db, block_height)
    }

    pub(crate) async fn write_block(
        &self,
        block: &Block,
        txs: &Vec<&Tx>,
    ) -> Result<String, LedgerError> {
        let db = &self.kv_db.db_instance;

        let mut batch = WriteBatch::default();

        let block_hash = block.get_hash();

        println!(
            "write block, hash: {}, height: {}",
            block_hash,
            block.get_height()
        );

        self.schema.batch_put_validator_sig(
            db,
            &mut batch,
            block_hash,
            block.get_validator_sig(),
        )?;

        self.schema.batch_put_witness_sigs(
            db,
            &mut batch,
            block_hash,
            block.get_witness_sigs(),
        )?;

        self.schema.batch_put_tx_hashes(
            db,
            &mut batch,
            block_hash,
            block.get_tx_hashes(),
        )?;

        self.schema.batch_put_created_at(
            db,
            &mut batch,
            block_hash,
            block.get_created_at(),
        )?;

        self.schema.batch_put_block_hash(
            db,
            &mut batch,
            block.get_height(),
            block_hash,
        )?;

        self.schema.batch_put_block_height(
            db,
            &mut batch,
            block_hash,
            block.get_height(),
        )?;

        for tx in txs {
            self._batch_put_tx(db, &mut batch, tx)?;
        }

        db.write(batch)?;

        return Ok(block_hash.clone());
    }
}
