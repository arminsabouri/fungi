// First stage is to send the bulletin board id to all my receivers (M)
// For now these will be all the receivers in the mp pj session. In the future these peers can invite their recievers to join as well.
// For simplicity we will just work with M = N

// Then I can send my inputs and wait for at least N-1 other participants to send their inputs
// After that I send my outputs and wait for at least N-1 other participants to send their outputs
// Then we signal we are ready to sign. Signing is ommited from this protocol.
// the mppj session intiator will sign the tx and broadcast it to the network.

use crate::{
    bulletin_board::{BroadcastMessageType, BulletinBoardId},
    transaction::{Outpoint, Output, TxData, TxId},
    wallet::PaymentObligationId,
    Simulation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MultiPartyPayjoinSession {
    pub(crate) payment_obligation_ids: Vec<PaymentObligationId>,
    pub(crate) tx_template: TxData,
    pub(crate) state: TxConstructionState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TxConstructionState {
    SentBulletinBoardId,
    SentInputs,
    SentOutputs,
    SentReadyToSign,
    Success(TxId),
}

#[derive(Debug)]
pub(crate) struct SentBulletinBoardId<'a> {
    pub(crate) bulletin_board_id: BulletinBoardId,
    pub(crate) tx_template: TxData,
    pub(crate) sim: &'a mut Simulation,
    pub(crate) n: u16,
}

impl<'a> SentBulletinBoardId<'a> {
    pub(crate) fn new(
        sim: &'a mut Simulation,
        bulletin_board_id: BulletinBoardId,
        tx_template: TxData,
        n: u16,
    ) -> Self {
        Self {
            bulletin_board_id,
            tx_template,
            sim,
            n,
        }
    }

    pub(crate) fn send_inputs(self) -> SentInputs<'a> {
        for input in self.tx_template.inputs.iter() {
            self.sim.add_message_to_bulletin_board(
                self.bulletin_board_id,
                BroadcastMessageType::ContributeInputs(input.outpoint.clone()),
            );
        }
        SentInputs::new(
            self.sim,
            self.bulletin_board_id,
            self.tx_template.clone(),
            self.n,
        )
    }
}

pub(crate) struct SentInputs<'a> {
    pub(crate) bulletin_board_id: BulletinBoardId,
    pub(crate) tx_template: TxData,
    pub(crate) sim: &'a mut Simulation,
    pub(crate) n: u16,
}

impl<'a> SentInputs<'a> {
    pub(crate) fn new(
        sim: &'a mut Simulation,
        bulletin_board_id: BulletinBoardId,
        tx_template: TxData,
        n: u16,
    ) -> Self {
        Self {
            bulletin_board_id,
            tx_template,
            sim,
            n,
        }
    }

    fn read_txin_messages(&self) -> Vec<Outpoint> {
        let messages = self.sim.bulletin_boards[self.bulletin_board_id.0]
            .messages
            .iter()
            .filter_map(|message| match message {
                BroadcastMessageType::ContributeInputs(outpoint) => Some(outpoint.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        messages
    }

    pub(crate) fn have_enough_inputs(self) -> Option<SentOutputs<'a>> {
        let inputs = self.read_txin_messages();
        if inputs.len() < self.n as usize {
            return None;
        }

        // Broadcast my outputs
        for output in self.tx_template.outputs.iter() {
            self.sim.add_message_to_bulletin_board(
                self.bulletin_board_id,
                BroadcastMessageType::ContributeOutputs(output.clone()),
            );
        }

        Some(SentOutputs::new(
            self.sim,
            self.bulletin_board_id,
            self.tx_template.clone(),
            self.n,
        ))
    }
}

#[derive(Debug)]
pub(crate) struct SentOutputs<'a> {
    pub(crate) bulletin_board_id: BulletinBoardId,
    pub(crate) tx_template: TxData,
    pub(crate) sim: &'a mut Simulation,
    pub(crate) n: u16,
}

impl<'a> SentOutputs<'a> {
    pub(crate) fn new(
        sim: &'a mut Simulation,
        bulletin_board_id: BulletinBoardId,
        tx_template: TxData,
        n: u16,
    ) -> Self {
        Self {
            bulletin_board_id,
            tx_template,
            sim,
            n,
        }
    }

    fn read_txout_messages(&self) -> Vec<Output> {
        let messages = self.sim.bulletin_boards[self.bulletin_board_id.0]
            .messages
            .iter()
            .filter_map(|message| match message {
                BroadcastMessageType::ContributeOutputs(output) => Some(output.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        messages
    }

    pub(crate) fn have_enough_outputs(self) -> Option<SentReadyToSign<'a>> {
        let outputs = self.read_txout_messages();
        if outputs.len() < self.n as usize {
            return None;
        }
        // Broadcast my ready to sign message
        self.sim.add_message_to_bulletin_board(
            self.bulletin_board_id,
            BroadcastMessageType::ReadyToSign(),
        );

        Some(SentReadyToSign::new(
            self.sim,
            self.bulletin_board_id,
            self.tx_template.clone(),
            self.n,
        ))
    }
}

#[derive(Debug)]
pub(crate) struct SentReadyToSign<'a> {
    pub(crate) bulletin_board_id: BulletinBoardId,
    pub(crate) tx_template: TxData,
    pub(crate) sim: &'a mut Simulation,
    pub(crate) n: u16,
}

impl<'a> SentReadyToSign<'a> {
    pub(crate) fn new(
        sim: &'a mut Simulation,
        bulletin_board_id: BulletinBoardId,
        tx_template: TxData,
        n: u16,
    ) -> Self {
        Self {
            bulletin_board_id,
            tx_template,
            sim,
            n,
        }
    }

    fn read_ready_to_sign_messages(&self) -> usize {
        self.sim.bulletin_boards[self.bulletin_board_id.0]
            .messages
            .iter()
            .filter(|message| matches!(message, BroadcastMessageType::ReadyToSign()))
            .count()
    }

    pub(crate) fn have_enough_ready_to_sign(self) -> Option<TxData> {
        let ready_to_sign_messages = self.read_ready_to_sign_messages();
        if ready_to_sign_messages < self.n as usize {
            return None;
        }
        // Signatures are abstracted away, so the "leader" can just boradcast to the network
        let messages = self.sim.bulletin_boards[self.bulletin_board_id.0]
            .messages
            .clone();
        let mut tx = TxData::default();
        for message in messages {
            match message {
                BroadcastMessageType::ContributeInputs(outpoint) => {
                    tx.inputs.push(crate::transaction::Input { outpoint });
                }
                BroadcastMessageType::ContributeOutputs(output) => {
                    tx.outputs.push(output);
                }
                _ => continue,
            }
        }

        Some(tx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        actions::{create_strategy, CompositeScorer, CompositeStrategy},
        transaction::Input,
        SimulationBuilder,
    };
    use bitcoin::Amount;

    // Test harness helpers
    mod test_harness {
        use super::*;

        /// Creates a minimal simulation with a specified number of wallets
        pub fn create_minimal_simulation(num_wallets: usize) -> crate::Simulation {
            use crate::config::{ScorerConfig, WalletTypeConfig};
            let wallet_types = vec![WalletTypeConfig {
                name: "test".to_string(),
                count: num_wallets,
                strategies: vec!["UnilateralSpender".to_string()],
                scorer: ScorerConfig {
                    initiate_payjoin_utility_factor: 0.0,
                    respond_to_payjoin_utility_factor: 0.0,
                    payment_obligation_utility_factor: 0.0,
                    multi_party_payjoin_utility_factor: 0.0,
                },
            }];
            SimulationBuilder::new(42, wallet_types, 10, 1, 0).build()
        }

        /// Creates a mock transaction template with specified number of inputs and outputs
        pub fn create_mock_tx_template(
            sim: &mut crate::Simulation,
            num_inputs: usize,
            num_outputs: usize,
        ) -> TxData {
            // Create a wallet and address for outputs
            let default_scorer = CompositeScorer {
                initiate_payjoin_utility_factor: 0.0,
                payment_obligation_utility_factor: 0.0,
                respond_to_payjoin_utility_factor: 0.0,
                multi_party_payjoin_utility_factor: 0.0,
            };
            let wallet = sim.new_wallet(
                CompositeStrategy {
                    strategies: vec![create_strategy("UnilateralSpender").unwrap()],
                },
                default_scorer,
            );
            let address = wallet.with_mut(sim).new_address();

            // Create mock inputs (using dummy outpoints)
            let mut inputs = Vec::new();
            for i in 0..num_inputs {
                inputs.push(Input {
                    outpoint: Outpoint {
                        txid: TxId(i),
                        index: 0,
                    },
                });
            }

            // Create mock outputs
            let mut outputs = Vec::new();
            for _ in 0..num_outputs {
                outputs.push(Output {
                    amount: Amount::from_sat(1000),
                    address_id: address,
                });
            }

            TxData {
                inputs,
                outputs,
                wallet_acks: Vec::new(),
            }
        }

        /// Adds input contributions from other participants to the bulletin board
        pub fn add_other_inputs(
            sim: &mut crate::Simulation,
            bulletin_board_id: BulletinBoardId,
            num_inputs: usize,
        ) {
            for i in 0..num_inputs {
                sim.add_message_to_bulletin_board(
                    bulletin_board_id,
                    BroadcastMessageType::ContributeInputs(Outpoint {
                        txid: TxId(100 + i), // Use different txids to distinguish
                        index: 0,
                    }),
                );
            }
        }

        /// Adds output contributions from other participants to the bulletin board
        pub fn add_other_outputs(
            sim: &mut crate::Simulation,
            bulletin_board_id: BulletinBoardId,
            num_outputs: usize,
        ) {
            let default_scorer = CompositeScorer {
                initiate_payjoin_utility_factor: 0.0,
                payment_obligation_utility_factor: 0.0,
                respond_to_payjoin_utility_factor: 0.0,
                multi_party_payjoin_utility_factor: 0.0,
            };
            let wallet = sim.new_wallet(
                CompositeStrategy {
                    strategies: vec![create_strategy("UnilateralSpender").unwrap()],
                },
                default_scorer,
            );
            let address = wallet.with_mut(sim).new_address();

            for _ in 0..num_outputs {
                sim.add_message_to_bulletin_board(
                    bulletin_board_id,
                    BroadcastMessageType::ContributeOutputs(Output {
                        amount: Amount::from_sat(2000),
                        address_id: address,
                    }),
                );
            }
        }

        /// Adds ready-to-sign messages from other participants
        pub fn add_other_ready_to_sign(
            sim: &mut crate::Simulation,
            bulletin_board_id: BulletinBoardId,
            num_messages: usize,
        ) {
            for _ in 0..num_messages {
                sim.add_message_to_bulletin_board(
                    bulletin_board_id,
                    BroadcastMessageType::ReadyToSign(),
                );
            }
        }
    }

    #[test]
    fn test_state_machine() {
        let n = 3;
        let mut sim = test_harness::create_minimal_simulation(3);

        let tx_template_1 = test_harness::create_mock_tx_template(&mut sim, 2, 1);

        let bulletin_board_id = sim.create_bulletin_board();
        test_harness::add_other_inputs(&mut sim, bulletin_board_id, 2);
        test_harness::add_other_outputs(&mut sim, bulletin_board_id, 2);
        test_harness::add_other_ready_to_sign(&mut sim, bulletin_board_id, 2);

        let session_1 = SentBulletinBoardId::new(&mut sim, bulletin_board_id, tx_template_1, n);
        let session_1 = session_1.send_inputs();

        // Send other inputs
        let sent_outputs = session_1
            .have_enough_inputs()
            .expect("should have enough inputs");
        let sent_ready = sent_outputs
            .have_enough_outputs()
            .expect("should have enough outputs");
        let txdata = sent_ready
            .have_enough_ready_to_sign()
            .expect("should have enough ready to sign");
        // TODO: assert the inputs and outputs are correct
        assert_eq!(txdata.inputs.len(), 4);
        assert_eq!(txdata.outputs.len(), 3);
    }
}
