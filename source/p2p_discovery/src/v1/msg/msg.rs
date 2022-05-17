#[derive(Debug)]
pub(crate) enum MsgType {
    WhoAreYouSyn,
    WhoAreYouAck,
}

pub(crate) struct Msg {
    pub(crate) msg_type: MsgType,
    pub(crate) content: Vec<u8>,
}

impl Msg {}