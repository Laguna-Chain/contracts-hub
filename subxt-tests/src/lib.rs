use subxt::{OnlineClient, PolkadotConfig};

pub mod eth_client;
pub mod generic_client;
pub mod utils;

// metadata file obtained from the latest substrate-contracts-node
#[subxt::subxt(runtime_metadata_path = "./laguna.scale")]
pub mod node {}

pub type API = OnlineClient<PolkadotConfig>;
