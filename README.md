# Registries Near
Set of Autonolas registries contracts on NEAR.

## Pre-requisites
The program requires that the following environment is satisfied:
```
rustc --version
rustc 1.81.0 (eeb90cda1 2024-09-04)

near --version
near-cli-rs 0.16.0
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
./scripts/build.sh
```


### Manage NEAR accounts
Create / delete accounts, transfer funds and deploy contracts on testnet:
```bash
./scripts/setup_contract_account_testnet.sh
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

Update (only after the relevant upgrade_hash has been set via the governance):
```bash
./scripts/upgrade_contract.sh)
```

Update check (macOS):
```bash
sha256 ./artifacts/governance_near.wasm 
near state gov_002.sub_olas.olas_000.testnet
```

Update check (Linux):
```bash
sha256sum ./artifacts/governance_near.wasm 
near state gov_002.sub_olas.olas_000.testnet
```

### Testnet
- RPC: https://rpc.testnet.near.org
- Faucet: https://near-faucet.io/
- Explorer: https://nearblocks.io/

## Acknowledgements
The tokenomics contracts were inspired and based on the following sources:
- [Wormhole Docs](https://docs.wormhole.com/);
- [Wormhole Near](https://github.com/wormhole-foundation/wormhole/blob/main/near/contracts/wormhole).
