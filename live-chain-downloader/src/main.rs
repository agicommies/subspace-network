use frame_remote_externalities::OnlineConfig;
use sp_runtime::{
    generic::{Block, Header},
    traits::BlakeTwo256,
    DeserializeOwned, OpaqueExtrinsic,
};

#[tokio::main]
async fn main() {
    env_logger::init();

    // <Block<Header<u32, BlakeTwo256>, OpaqueExtrinsic>, sp_io::SubstrateHostFunctions>
    foo::<Block<Header<u32, BlakeTwo256>, OpaqueExtrinsic>>().await;
}

async fn foo<Block: sp_runtime::traits::Block + DeserializeOwned>() {
    let ext = frame_remote_externalities::Builder::<Block>::new()
        .mode(frame_remote_externalities::Mode::Online(OnlineConfig {
            at: None,
            state_snapshot: None,
            pallets: vec![],
            transport: "wss://commune-api-node-1.communeai.net:443".to_string().into(),
            child_trie: true,
            hashed_prefixes: vec![],
            hashed_keys: vec![],
        }))
        .build()
        .await
        .unwrap();

    ext.insert(k, v)
}
