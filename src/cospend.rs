use crate::{
    transaction::{Input, Output},
    TimeStep,
};

define_entity!(
    Cospend,
    {
        pub(crate) id: CospendId,
        pub(crate) inputs: Vec<Input>,
        pub(crate) outputs: Vec<Output>,
        pub(crate) valid_till: TimeStep,
        // TODO: acceptable fee rate
        // TODO: other transaction properties (nsequence, nlocktime)
    },
    {
    }
);
define_entity_handle_mut!(Cospend);

impl<'a> CospendHandle<'a> {
    pub(crate) fn data(&self) -> &'a CospendData {
        &self.sim.cospends[self.id.0]
    }
}

impl<'a> CospendHandleMut<'a> {
    pub(crate) fn post(&mut self, cospend: CospendData) {
        self.sim.cospends.push(cospend);
    }
}
