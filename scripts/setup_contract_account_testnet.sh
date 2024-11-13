near account delete-account gov_000.sub_olas.olas_000.testnet beneficiary sub_olas.olas_000.testnet network-config testnet sign-with-keychain send

rm -rf ../../.near-credentials/testnet/gov*

near account create-account fund-myself gov_000.sub_olas.olas_000.testnet '10 NEAR' autogenerate-new-keypair save-to-legacy-keychain sign-as sub_olas.olas_000.testnet network-config testnet sign-with-keychain send

#near send-near sub_olas.olas_000.testnet gov_000.sub_olas.olas_000.testnet 1 --networkId testnet

#near send-near olas_000.testnet sub_olas.olas_000.testnet 8 --networkId testnet

near deploy gov_000.sub_olas.olas_000.testnet target/wasm32-unknown-unknown/release/governor_near.wasm --initFunction new --initArgs '{"owner_id":"sub_olas.olas_000.testnet", "wormhole_core": "wormhole.wormhole.testnet", "foreign_governor_address": [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 71, 27, 63, 96, 240, 140, 80, 221, 14, 203, 161, 188, 209, 19, 182, 111, 204, 2, 182, 61], "chain_id": 10002}' --networkId testnet

cp ../../.near-credentials/testnet/gov_000.sub_olas.olas_000.testnet.json .near-credentials/workspaces/testnet/.