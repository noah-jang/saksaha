use super::*;
use crate::db::columns::ledger_columns;

#[cfg(test)]
mod test {
    use super::dummy::Transaction;
    use crate::db::DB;
    use crate::{blockchain::Blockchain, db::columns::ledger_columns};

    fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_send_transaction() {
        init();

        let db = DB::init().await.unwrap();

        // let blockchain = Blockchain::init(db.ledger_db).await.unwrap();

        let tx =
            Transaction::new("0x0000", "0x123", "0x0000", "1346546123", "None");
        let db = db.ledger_db.db;

        let cf_val = vec![
            (ledger_columns::TX_HASH, tx.tx_hash),
            (ledger_columns::PI, tx.pi),
            (ledger_columns::SIG_VEC, tx.sig_vec),
            (ledger_columns::CREATED_AT, tx.created_at),
            (ledger_columns::DATA, tx.data),
        ];

        let cf_val_exe = cf_val
            .into_iter()
            .map(|(cf, val)| {
                println!("key: {}, val: {}", cf, val);
                db.put_cf(db.cf_handle(cf).unwrap(), "0", val).unwrap();
            })
            .collect();

        cf_val_exe
    }
}

#[cfg(test)]
mod dummy {
    pub(crate) struct Transaction<'a> {
        pub tx_hash: &'a str,
        pub pi: &'a str,
        pub sig_vec: &'a str,
        pub created_at: &'a str,
        pub data: &'a str,
    }

    impl<'a> Transaction<'a> {
        pub(crate) fn new(
            tx_hash: &'a str,
            pi: &'a str,
            sig_vec: &'a str,
            created_at: &'a str,
            data: &'a str,
        ) -> Transaction<'a> {
            Transaction {
                tx_hash,
                pi,
                sig_vec,
                created_at,
                data,
            }
        }
    }
}
