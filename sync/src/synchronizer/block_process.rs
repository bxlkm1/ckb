use crate::{
    synchronizer::{BlockStatus, Synchronizer},
    Status,
};
use ckb_logger::debug;
use ckb_network::{CKBProtocolContext, PeerIndex};
use ckb_types::{packed, prelude::*};
use std::sync::Arc;

pub struct BlockProcess<'a> {
    message: packed::SendBlockReader<'a>,
    synchronizer: &'a Synchronizer,
    peer: PeerIndex,
    nc: Arc<dyn CKBProtocolContext + Sync>,
}

impl<'a> BlockProcess<'a> {
    pub fn new(
        message: packed::SendBlockReader<'a>,
        synchronizer: &'a Synchronizer,
        peer: PeerIndex,
        nc: Arc<dyn CKBProtocolContext + Sync>,
    ) -> Self {
        BlockProcess {
            message,
            synchronizer,
            peer,
            nc,
        }
    }

    pub fn execute(self) -> Status {
        let block = Arc::new(self.message.block().to_entity().into_view());
        debug!(
            "BlockProcess received block {} {}",
            block.number(),
            block.hash(),
        );
        let shared = self.synchronizer.shared();
        let state = shared.state();

        if state.new_block_received(&block) {
            self.synchronizer
                .receive_block(self.peer, block.clone(), Arc::clone(&self.nc));
        } else if shared
            .active_chain()
            .contains_block_status(&block.hash(), BlockStatus::BLOCK_STORED)
        {
            state
                .peers()
                .set_last_common_header(self.peer, block.header());
        }

        Status::ok()
    }
}
