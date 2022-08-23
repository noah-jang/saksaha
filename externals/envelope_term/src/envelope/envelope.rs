use std::sync::Arc;

use super::actions::Actions;
use super::{state::AppState, ChannelState};
use crate::credential::Credential;
use crate::db::EnvelopeDB;
use crate::io::IoEvent;
use crate::{app, EnvelopeError};
use crate::{envelope::actions::Action, ENVELOPE_CTR_ADDR};
use chrono::Local;
use envelope_contract::{
    request_type::{GET_CH_LIST, GET_MSG, OPEN_CH, SEND_MSG},
    Channel, ChatMessage, EncryptedChatMessage, GetChListParams, GetMsgParams,
    OpenChParams, SendMsgParams,
};
use log::{error, warn};
use sak_contract_std::{CtrCallType, CtrRequest};
use sak_crypto::{
    aes_decrypt, derive_aes_key, PublicKey, SakKey, SecretKey, ToEncodedPoint,
};
use type_extension::{U8Arr32, U8Array};

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct Envelope {
    io_tx: tokio::sync::mpsc::Sender<IoEvent>,
    actions: Actions,
    state: AppState,
    db: EnvelopeDB,
    credential: Arc<Credential>,
    partner_credential: Credential,
}

impl Envelope {
    pub(crate) async fn init(
        io_tx: tokio::sync::mpsc::Sender<IoEvent>,
        // user_prefix: &String,
        credential: Arc<Credential>,
    ) -> Result<Self, EnvelopeError> {
        let actions = vec![Action::Quit].into();
        let state = AppState::default();

        let db = EnvelopeDB::init(&credential.acc_addr).await?;

        // for test, dummy
        let partner_credential = Credential::new(None, None)?;
        {
            db.register_user(&credential.clone()).await?;

            db.register_user(&partner_credential).await?;
        }

        Ok(Self {
            io_tx,
            actions,
            state,
            db,
            credential,
            partner_credential,
        })
    }

    /// We could update the app or dispatch event on tick
    pub async fn update_on_tick(&mut self) -> AppReturn {
        // here we just increment a counter
        self.state.incr_tick();
        AppReturn::Continue
    }

    /// Send a network event to the IO thread
    pub async fn dispatch(&mut self, action: IoEvent) {
        // `is_loading` will be set to false again after the async
        // action has finished in io/handler.rs
        self.state.is_loading = true;

        if let Err(e) = self.io_tx.send(action).await {
            self.state.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }

    pub fn get_actions(&self) -> &Actions {
        &self.actions
    }

    pub(crate) fn get_db(&self) -> &EnvelopeDB {
        &self.db
    }

    pub(crate) fn get_state(&self) -> &AppState {
        &self.state
    }

    pub(crate) fn get_credential(&self) -> &Credential {
        &self.credential
    }

    pub(crate) fn get_state_mut(&mut self) -> &mut AppState {
        &mut self.state
    }

    pub fn initialized(&mut self) {
        self.actions = vec![
            Action::Quit,
            Action::SwitchEditMode,
            Action::SwitchNormalMode,
            Action::ShowOpenCh,
            Action::ShowChList,
            Action::ShowChat,
            Action::Down,
            Action::Up,
            Action::UpdateBalance,
            Action::Select,
            Action::RestoreChat,
        ]
        .into();

        self.state = AppState::initialized()
    }

    pub fn loaded(&mut self) {
        self.state.is_loading = false;
    }

    pub fn slept(&mut self) {
        self.state.incr_sleep();
    }

    pub async fn set_ch_list(
        &mut self,
        data: Vec<u8>,
    ) -> Result<(), EnvelopeError> {
        match serde_json::from_slice::<Vec<Channel>>(&data) {
            Ok(c) => {
                for i in c.into_iter() {
                    let mut new_ch = ChannelState::new(i, String::default());

                    // First, try to decrypt the `ch_id` with `my_sk`
                    let my_sk = {
                        let s = &self.credential.secret;

                        U8Array::from_hex_string(s.to_string())?
                    };

                    let ch_id_decrypted = {
                        let ch_id: Vec<u8> = serde_json::from_str(
                            &new_ch.channel.ch_id.clone().as_str(),
                        )?;

                        String::from_utf8(aes_decrypt(&my_sk, &ch_id)?)?
                    };

                    // Prefix of the encrypted `ch_id` is `MY_PK` rn
                    let my_pk = &self.credential.public_key;

                    if &ch_id_decrypted[0..my_pk.len()] == my_pk.as_str() {
                        let ch_id: String =
                            match ch_id_decrypted.split('_').nth(1) {
                                Some(ci) => ci.to_string(),
                                None => {
                                    return Err(format!(
                                        "\
                                        Error occured while \
                                        parsing encrypted `ch_id`\
                                    "
                                    )
                                    .into());
                                }
                            };

                        let sig_decrypted: String = {
                            let sig: Vec<u8> = serde_json::from_str(
                                &new_ch.channel.sig.clone().as_str(),
                            )?;

                            String::from_utf8(aes_decrypt(&my_sk, &sig)?)?
                        };

                        new_ch.channel.ch_id = ch_id;

                        new_ch.channel.sig = sig_decrypted;

                        self.state.set_ch_list(new_ch)?;
                    } else {
                        // If the decryption with `MY_SK` has failed,
                        // it should be decrypted with ECIES-scheme aes key
                        let aes_key = {
                            let my_sk = {
                                let s = &self.credential.secret;

                                SecretKey::from_bytes(s.as_bytes())?
                            };

                            let eph_pub_key = PublicKey::from_sec1_bytes(
                                new_ch.channel.eph_key.as_bytes(),
                            )?;

                            derive_aes_key(my_sk, eph_pub_key)?
                        };

                        let ch_id_decrypted = {
                            let ch_id: Vec<u8> = serde_json::from_str(
                                &new_ch.channel.ch_id.clone().as_str(),
                            )?;

                            String::from_utf8(aes_decrypt(&aes_key, &ch_id)?)?
                        };

                        let ch_id: String =
                            match ch_id_decrypted.split('_').nth(1) {
                                Some(ci) => ci.to_string(),
                                None => {
                                    return Err(format!(
                                        "\
                                        Error occured while \
                                        parsing encrypted `ch_id`\
                                    "
                                    )
                                    .into());
                                }
                            };

                        let sig_decrypted: String = {
                            let sig: Vec<u8> = serde_json::from_str(
                                &new_ch.channel.sig.clone().as_str(),
                            )?;

                            String::from_utf8(aes_decrypt(&aes_key, &sig)?)?
                        };

                        new_ch.channel.ch_id = ch_id;

                        new_ch.channel.sig = sig_decrypted;

                        self.state.set_ch_list(new_ch)?;
                    }
                }
            }
            Err(_) => {}
        }

        // self.state.set_ch_list(data)?;

        Ok(())
    }

    pub async fn set_chats(
        &mut self,
        data: Vec<u8>,
    ) -> Result<(), EnvelopeError> {
        let my_pk = self.credential.public_key.to_string();
        let my_sk = self.credential.secret.to_string();

        let encrypted_chat_msg_vec: Vec<EncryptedChatMessage> =
            match serde_json::from_slice::<Vec<EncryptedChatMessage>>(&data) {
                Ok(c) => c.into_iter().map(|m| m).collect(),
                Err(err) => {
                    return Err(format!(
                        "failed to deserialize vec<string>, err: {:?}",
                        err
                    )
                    .into());
                }
            };

        let eph_key: String = {
            let mut res: String = String::default();

            for ch_state in self.get_state().ch_list.iter() {
                if ch_state.channel.ch_id == self.get_state().selected_ch_id {
                    res = ch_state.channel.eph_key.clone();
                }
            }

            res
        };

        let aes_key = {
            if &eph_key[0..5] == "init_" {
                let eph_sk = &eph_key[5..];

                let eph_sk_encrypted: Vec<u8> = serde_json::from_str(eph_sk)?;

                let sk = {
                    let my_sk: U8Arr32 =
                        U8Array::from_hex_string(my_sk.to_string())?;

                    let eph_sk =
                        sak_crypto::aes_decrypt(&my_sk, &eph_sk_encrypted)?;

                    SecretKey::from_bytes(&eph_sk)?
                };

                let pk = {
                    // for dev, her_pk == `user_2_pk`
                    let her_pk =
                        self.get_pk(&self.partner_credential.acc_addr).await?;

                    let her_pk_vec: Vec<u8> = sak_crypto::decode_hex(&her_pk)?;

                    PublicKey::from_sec1_bytes(&her_pk_vec)?
                };

                derive_aes_key(sk, pk)?
            } else {
                let eph_pk = eph_key;

                let sk = SecretKey::from_bytes(&my_sk.as_bytes())?;

                let pk = {
                    let eph_pk_vec: Vec<u8> = sak_crypto::decode_hex(&eph_pk)?;

                    PublicKey::from_sec1_bytes(&eph_pk_vec)?
                };

                derive_aes_key(sk, pk)?
            }
        };

        let mut chat_msg: Vec<ChatMessage> = vec![];

        for encrypted_chat_msg in encrypted_chat_msg_vec.into_iter() {
            let encrypted_chat_msg: Vec<u8> =
                serde_json::from_str(&encrypted_chat_msg)?;

            let chat_msg_ser: String = {
                let chat_msg =
                    sak_crypto::aes_decrypt(&aes_key, &encrypted_chat_msg)?;

                String::from_utf8(chat_msg)?
            };

            let mut res: ChatMessage = serde_json::from_str(&chat_msg_ser)?;

            if res.user == my_pk {
                res.user = "me".to_string();
            } else {
                res.user = res.user[0..16].to_string();
            }

            chat_msg.push(res);
        }

        self.get_state_mut().set_chats(chat_msg, my_pk);

        log::info!("set_chats done");

        Ok(())
    }

    pub async fn open_ch(
        &mut self,
        her_pk: &String,
    ) -> Result<(), EnvelopeError> {
        log::info!("Trying to make a channel w/ partner: {:?}", her_pk);

        let (eph_sk, eph_pk) = SakKey::generate();

        let eph_pk: String =
            serde_json::to_string(eph_pk.to_encoded_point(false).as_bytes())?;
        let my_sk = self.credential.secret.clone();
        let my_pk = self.credential.public_key.clone();
        let my_sig = self.credential.signature.clone();
        let user_1_acc_addr = self.credential.acc_addr.clone();

        let ch_id_num = sak_crypto::rand();

        let ch_id = format!("{}_{}", my_pk, ch_id_num.to_string());

        {
            // =-=-=-=-=-= `open_ch` for initiator  =-=-=-=-=-=-=-=

            let my_sk: U8Arr32 = U8Array::from_hex_string(my_sk)?;

            let open_ch = {
                let ch_id_enc = {
                    let ch_id_enc =
                        sak_crypto::aes_encrypt(&my_sk, &ch_id.as_bytes())?;

                    serde_json::to_string(&ch_id_enc)?
                };

                let eph_sk_enc = {
                    let eph_sk_enc: Vec<u8> =
                        sak_crypto::aes_encrypt(&my_sk, &eph_sk.to_bytes())?;

                    // for dev, prefix is `init_`
                    format!("init_{}", serde_json::to_string(&eph_sk_enc)?)
                };

                let sig_enc = {
                    let sig_enc =
                        sak_crypto::aes_encrypt(&my_sk, &my_sig.as_bytes())?;

                    serde_json::to_string(&sig_enc)?
                };

                Channel::new(ch_id_enc, eph_sk_enc, sig_enc)?
            };

            let ctr_addr = ENVELOPE_CTR_ADDR.to_string();

            let open_ch_params = OpenChParams {
                dst_pk: my_pk,
                open_ch,
            };

            let req_type = OPEN_CH.to_string();

            let args = serde_json::to_vec(&open_ch_params)?;

            let ctr_request = CtrRequest {
                req_type,
                args,
                ctr_call_type: CtrCallType::Execute,
            };

            app::send_tx_pour(user_1_acc_addr, ctr_addr, ctr_request).await?;
        }

        {
            // =-=-=-=-=-=  `open_ch` for receiver =-=-=-=-=-=-=-=

            let aes_key = {
                let her_pk: Vec<u8> = sak_crypto::decode_hex(her_pk)?;

                let her_pk = PublicKey::from_sec1_bytes(&her_pk.as_slice())?;

                sak_crypto::derive_aes_key(eph_sk, her_pk)?
            };

            let open_ch = {
                let ch_id_enc = {
                    let ch_id_enc =
                        sak_crypto::aes_encrypt(&aes_key, &ch_id.as_bytes())?;

                    serde_json::to_string(&ch_id_enc)?
                };

                let eph_pk = eph_pk;

                let sig_enc = {
                    let sig_enc =
                        sak_crypto::aes_encrypt(&aes_key, &my_sig.as_bytes())?;

                    serde_json::to_string(&sig_enc)?
                };

                Channel::new(ch_id_enc, eph_pk, sig_enc)?
            };

            let ctr_addr = ENVELOPE_CTR_ADDR.to_string();

            let open_ch_params = OpenChParams {
                dst_pk: her_pk.clone(),
                open_ch,
            };

            let req_type = OPEN_CH.to_string();

            let args = serde_json::to_vec(&open_ch_params)?;

            let ctr_request = CtrRequest {
                req_type,
                args,
                ctr_call_type: CtrCallType::Execute,
            };

            app::send_tx_pour(
                self.partner_credential.acc_addr.clone(),
                ctr_addr,
                ctr_request,
            )
            .await?;
        }

        Ok(())
    }

    pub async fn get_ch_list(&mut self) -> Result<(), EnvelopeError> {
        let my_pk = &self.credential.public_key;

        let get_ch_list_params = GetChListParams {
            dst_pk: my_pk.clone(),
        };

        let args = serde_json::to_vec(&get_ch_list_params)?;

        if let Some(d) = saksaha::query_ctr(
            ENVELOPE_CTR_ADDR.to_string(),
            GET_CH_LIST.to_string(),
            args,
        )
        .await?
        .result
        {
            self.dispatch(IoEvent::GetChList(d.result)).await
        };

        Ok(())
    }

    pub async fn get_messages(
        &mut self,
        ch_id: String,
    ) -> Result<(), EnvelopeError> {
        let get_msg_params = GetMsgParams { ch_id };

        let args = serde_json::to_vec(&get_msg_params)?;

        if let Ok(r) = saksaha::query_ctr(
            ENVELOPE_CTR_ADDR.into(),
            GET_MSG.to_string(),
            args,
        )
        .await
        {
            if let Some(d) = r.result {
                self.dispatch(IoEvent::GetMessages(d.result)).await;
            }
        }

        Ok(())
    }

    pub async fn send_messages(
        &self,
        msg: &String,
    ) -> Result<(), EnvelopeError> {
        let ctr_addr = ENVELOPE_CTR_ADDR.to_string();

        let user_1_pk = self.credential.public_key.to_string();
        let user_1_sk = &self.credential.secret;
        let user_1_acc_addr = &self.credential.acc_addr;

        let user_1_sk: U8Arr32 =
            U8Array::from_hex_string(user_1_sk.to_string())?;

        let selected_ch_id = self.state.selected_ch_id.clone();

        let eph_key: String = {
            let mut res: String = String::default();

            for ch_state in self.get_state().ch_list.iter() {
                if ch_state.channel.ch_id == selected_ch_id {
                    res = ch_state.channel.eph_key.clone();
                }
            }

            res
        };

        let aes_key = {
            // In this channel, I am Initiator: `eph_key` == `eph_sk`
            // the `aes_key` will be `kdf(eph_sk, my_pk)`
            if &eph_key[0..5] == "init_" {
                let eph_sk = &eph_key[5..];

                let eph_sk_encrypted: Vec<u8> = serde_json::from_str(eph_sk)?;

                let sk = {
                    let eph_sk =
                        sak_crypto::aes_decrypt(&user_1_sk, &eph_sk_encrypted)?;

                    SecretKey::from_bytes(&eph_sk)?
                };

                let pk = {
                    // for dev, her_pk == `user_2_pk`
                    let her_pk =
                        self.get_pk(&self.partner_credential.acc_addr).await?;

                    let her_pk_vec: Vec<u8> = sak_crypto::decode_hex(&her_pk)?;

                    PublicKey::from_sec1_bytes(&her_pk_vec)?
                };

                derive_aes_key(sk, pk)?
            } else {
                // In this channel, I am Receiver: `eph_key` == `eph_pk`
                // The Initiator had opened channel with `my public key`,
                // so the `aes_key` will be `kdf(my_sk, eph_pk)`
                let eph_pk = eph_key;

                let sk = {
                    let my_sk = &self.credential.secret;

                    SecretKey::from_bytes(&my_sk.as_bytes())?
                };

                let pk = {
                    let eph_pk_vec: Vec<u8> = sak_crypto::decode_hex(&eph_pk)?;

                    PublicKey::from_sec1_bytes(&eph_pk_vec)?
                };

                derive_aes_key(sk, pk)?
            }
        };

        let chat_msg = ChatMessage {
            date: Local::now().format("%H:%M:%S ").to_string(),
            user: user_1_pk,
            msg: msg.clone(),
        };

        let chat_msg_serialized = serde_json::to_string(&chat_msg)?;
        // let chat_msg_serialized = serde_json::to_string(&msg)?;

        let encrypted_msg = {
            let encrypted_msg = &sak_crypto::aes_encrypt(
                &aes_key,
                chat_msg_serialized.as_bytes(),
            )?;

            serde_json::to_string(encrypted_msg)?
        };

        let send_msg_params = SendMsgParams {
            ch_id: selected_ch_id,
            msg: encrypted_msg,
        };

        let args = serde_json::to_vec(&send_msg_params)?;

        let req_type = SEND_MSG.to_string();

        let ctr_request = CtrRequest {
            req_type,
            args,
            ctr_call_type: CtrCallType::Execute,
        };

        app::send_tx_pour(user_1_acc_addr.to_string(), ctr_addr, ctr_request)
            .await?;

        Ok(())
    }

    // Now we do not do encryption nonetheless
    // async fn encrypt_open_ch(
    //     &mut self,
    //     her_pk: &String,
    //     ch_id: &String,
    // ) -> Result<Channel, EnvelopeError> {
    //     let my_sk = match self
    //         .db
    //         .schema
    //         .get_my_sk_by_acc_addr(&USER_1.to_string())
    //         .await?
    //     {
    //         Some(v) => v,
    //         None => {
    //             return Err(
    //                 format!("failed to get secret key from user id",).into()
    //             )
    //         }
    //     };

    //     let my_sig = match self.db.schema.get_my_sig_by_sk(&my_sk).await? {
    //         Some(v) => v,
    //         None => {
    //             return Err(
    //                 format!("failed to get my signature from sk",).into()
    //             )
    //         }
    //     };

    //     let (eph_sk, eph_pk) = SakKey::generate();

    //     let her_pk_vec: Vec<u8> = sak_crypto::decode_hex(her_pk)?;
    //     let her_pk_pub = PublicKey::from_sec1_bytes(&her_pk_vec.as_slice())?;

    //     let (a_pk_sig_encrypted, aes_key_from_a) = {
    //         let aes_key_from_a = sak_crypto::derive_aes_key(eph_sk, her_pk_pub);

    //         let a_credential_encrypted = {
    //             let ciphertext = sak_crypto::aes_encrypt(
    //                 &aes_key_from_a,
    //                 my_sig.as_bytes(),
    //             )?;

    //             serde_json::to_string(&ciphertext)?
    //         };

    //         (a_credential_encrypted, aes_key_from_a)
    //     };

    //     self.db
    //         .schema
    //         .put_ch_data(&ch_id, her_pk, &aes_key_from_a)
    //         .await?;

    //     // let open_ch_input: Vec<String> =
    //     //     vec![eph_pk_str, ch_id, a_pk_sig_encrypted, ];

    //     // serde_json::to_string(&open_ch_input)?

    //     let eph_pk_str =
    //         serde_json::to_string(eph_pk.to_encoded_point(false).as_bytes())?;

    //     let open_ch = Channel {
    //         ch_id,
    //         eph_key: eph_pk_str,
    //         sig: a_pk_sig_encrypted,
    //     };

    //     Ok(open_ch)
    // }

    async fn get_pk(&self, acc_addr: &String) -> Result<String, EnvelopeError> {
        let user_sk = self
            .db
            .schema
            .get_my_sk_by_acc_addr(acc_addr)
            .await?
            .ok_or("Cannot retrieve pk")?;

        let user_pk =
            self.db.schema.get_my_pk_by_sk(&user_sk).await?.ok_or("")?;

        Ok(user_pk)
    }

    async fn get_acc_addr(
        &self,
        user: &String,
    ) -> Result<String, EnvelopeError> {
        let acc_addr = self
            .db
            .schema
            .get_my_acc_addr_by_user_id(user)
            .await?
            .ok_or("")?;

        Ok(acc_addr)
    }

    async fn get_sig(&self, secret: &String) -> Result<String, EnvelopeError> {
        let user_sig =
            self.db.schema.get_my_sig_by_sk(secret).await?.ok_or("")?;

        Ok(user_sig)
    }

    pub fn get_partner_pk(&self) -> &String {
        &self.partner_credential.public_key
    }

    // async fn get_shared_secret(
    //     &self,
    //     ch_id: &String,
    // ) -> Result<String, EnvelopeError> {
    //     let shared_secret = self
    //         .db
    //         .schema
    //         .get_ch_shared_secret(ch_id)
    //         .await?
    //         .ok_or("")?;

    //     Ok(shared_secret)
    // }
}