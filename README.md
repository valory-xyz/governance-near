# Registries Near
Set of Autonolas registries contracts on NEAR.

## Pre-requisites
The program requires that the following environment is satisfied:
```
rustc --version
rustc 1.81.0 (eeb90cda1 2024-09-04)
```

Advise the script `setup-env.sh` to correctly install the required environment.

## Development
Install the dependencies:
```
yarn
```

If you need to remove / check dependencies, run:
```
cargo clean
cargo tree
```

You might also want to completely remove the `Cargo.lock` file.

Build the code with:
```
anchor build
```

### Create NEAR accounts
Documentation (subject to change): https://docs.near.org/concepts/protocol/account-id

Current version:
```bash
near account create-account fund-later `ACCOUNT_NAME` autogenerate-new-keypair save-to-legacy-keychain network-config testnet create
```

### Testing
Sandbox:
```bash
npx ava test/WormholeMessenger.ts
```

Testnet:
```bash
npx ava --config ava.testnet.config.cjs test/testnet_WormholeMessenger.ts
```

Testing with debug:
```bash
NEAR_WORKSPACES_DEBUG=true npx ava test/WormholeMessenger.ts
```

### Localnet
The local validator in this case is the project `near-sandbox`
https://github.com/near/near-sandbox
RPC: http://0.0.0.0:3030

Install sandbox:
```bash
npm i -g near-sandbox
```

Init sandbox:
```bash
# home of sandbox must be outside of repo, in /tmp
near-sandbox --home /tmp/near-sandbox init
# in another shell-windows
near-sandbox --home /tmp/near-sandbox run
```

Deploy the contract in the testnet:
```bash
    UPDATE: near deploy contract_000.sub_olas.olas_000.testnet target/wasm32-unknown-unknown/release/governor_near.wasm --initFunction new_default_meta --initArgs '{"owner_id":"sub_olas.olas_000.testnet", "multisig_factory": "multisignature2.testnet"}' --networkId testnet
```

Update:
```bash
near contract call-function as-transaction gov_002.sub_olas.olas_000.testnet update_contract_work file-args artifacts/governor_near.wasm prepaid-gas '300.0 Tgas' attached-deposit '0 NEAR' sign-as sub_olas.olas_000.testnet network-config testnet sign-with-keychain send
```

Update check (macOS):
```bash
sha256 ./artifacts/governor_near.wasm 
near state gov_002.sub_olas.olas_000.testnet
```
Update check (Linux):
```bash
sha256sum ./artifacts/governor_near.wasm 
near state gov_002.sub_olas.olas_000.testnet
```

### Testnet
- RPC: https://rpc.testnet.near.org
- Faucet: https://near-faucet.io/
- Explorer: https://nearblocks.io/
