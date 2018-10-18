use bigint::H256;
use ckb_chain::chain::ChainProvider;
use ckb_protocol::{FlatbuffersVectorIterator, GetBlocks, SyncMessage};
use flatbuffers::FlatBufferBuilder;
use network::{NetworkContext, PeerId};
use synchronizer::Synchronizer;

pub struct GetBlocksProcess<'a, C: 'a> {
    message: &'a GetBlocks<'a>,
    synchronizer: &'a Synchronizer<C>,
    nc: &'a NetworkContext,
}

impl<'a, C> GetBlocksProcess<'a, C>
where
    C: ChainProvider + 'a,
{
    pub fn new(
        message: &'a GetBlocks,
        synchronizer: &'a Synchronizer<C>,
        _peer: PeerId,
        nc: &'a NetworkContext,
    ) -> Self {
        GetBlocksProcess {
            message,
            nc,
            synchronizer,
        }
    }

    pub fn execute(self) {
        FlatbuffersVectorIterator::new(self.message.block_hashes().unwrap()).for_each(|bytes| {
            let block_hash = H256::from_slice(bytes.seq().unwrap());
            debug!(target: "sync", "get_blocks {:?}", block_hash);
            if let Some(block) = self.synchronizer.get_block(&block_hash) {
                debug!(target: "sync", "respond_block {} {:?}", block.header().number(), block.header().hash());
                let fbb = &mut FlatBufferBuilder::new();
                let message = SyncMessage::build_block(fbb, &block);
                fbb.finish(message, None);
                self.nc.respond(0, fbb.finished_data().to_vec());
            } else {
                // TODO response not found
                // TODO add timeout check in synchronizer
            }
        })
    }
}
