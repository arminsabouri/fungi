use crate::{
    cospend::CospendId,
    transaction::{Input, Output},
    wallet::WalletId,
    TimeStep,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct RegisterInput {
    pub(crate) wallet_id: WalletId,
    pub(crate) input: Input,
    /// If None, the input is valid forever, otherwise it is valid until the timestep in the option
    pub(crate) valid_till: Option<TimeStep>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct InitiateCospend {
    pub(crate) cospend_id: CospendId,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct RegisterOutputs {
    pub(crate) cospend_id: CospendId,
    pub(crate) outputs: Vec<Output>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum MessageType {
    RegisterInput(RegisterInput),
    RegisterCospend(InitiateCospend),
    RegisterOutputs(RegisterOutputs),
}

define_entity!(
    Message,
    {
        pub(crate) id: MessageId,
        pub(crate) message: MessageType,
        pub(crate) from: WalletId,
        // None if meant as a broadcast message
        pub(crate) to: Option<WalletId>,
    },
    {
    }
);
define_entity_handle_mut!(Message);

impl<'a> MessageHandle<'a> {
    pub(crate) fn data(&self) -> &'a MessageData {
        &self.sim.messages[self.id.0]
    }
}

impl<'a> MessageHandleMut<'a> {
    pub(crate) fn post(&mut self, message: MessageData) {
        self.sim.messages.push(message);
    }
}
