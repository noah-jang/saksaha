use crate::{get_mrs_data_from_host, put_mrs_data_to_host, ContractError, RET_LEN_SIZE};
use std::{collections::HashMap, convert::TryInto};

#[derive(Debug)]
pub struct List {
    _name: String,
    receipt: HashMap<String, Vec<u8>>,
}

impl List {
    pub fn new(_name: String) -> List {
        List {
            _name,
            receipt: HashMap::new(),
        }
    }

    pub fn get(&self, key: &String) -> Vec<u8> {
        let key: String = format!("{}_{}", self._name, key);

        let data = get_mrs_data_from_host(&key);

        data
    }

    pub fn put(&self, value: &String) {
        let key: String = format!("{}", self._name);

        put_mrs_data_to_host(&key, value);
    }

    pub fn push(&mut self, value: Vec<u8>) {
        //TO-DO: get latest idx of the stored List and update index
        let latest_idx_key = String::from("latest_idx");
        let latest_idx = get_mrs_data_from_host(&latest_idx_key);

        let latest_idx = 0;

        let key: String = format!("{}_{}", self._name, latest_idx);

        self.receipt.insert(key, value);
    }

    pub fn receipt(&self) -> HashMap<String, Vec<u8>> {
        self.receipt.clone()
    }
}