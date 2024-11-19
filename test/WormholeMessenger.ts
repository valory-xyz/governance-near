import {Worker, NEAR, NearAccount} from "near-workspaces";
import anyTest, {TestFn} from "ava";

interface Call {
    contract_id: string;
    deposit: NEAR;
    gas: number;
    method_name: string;
    args: number[];
}

interface CallResult {
    success: boolean;
    result: number[] | null;
}

// This corresponds to Sepolia timelock address 000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d or 0x471b3f60f08c50dd0ecba1bcd113b66fcc02b63d
const timelockBuffer = new Uint8Array([
    0,   0,  0,   0,   0,   0,   0,   0,   0,
    0,   0,  0,  71,  27,  63,  96, 240, 140,
    80, 221, 14, 203, 161, 188, 209,  19, 182,
    111, 204,  2, 182,  61
]);

const sepoliaChainId = 10002;

const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async t => {
    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    // Prepare sandbox for tests, create accounts, deploy contracts, etx.
    const root = worker.rootAccount;
    // Deploy the main governance contract
    const contract = await root.devDeploy(
        "target/wasm32-unknown-unknown/release/governor_near.wasm",
        {initialBalance: NEAR.parse("20 N").toJSON()},
    );

    // Deploy the mock wormhole
    const wormhole = await root.devDeploy(
        "artifacts/wormhole_near.wasm",
        {initialBalance: NEAR.parse("20 N").toJSON()},
    );

    // Allocate accounts
    const deployer = await root.createSubAccount("deployer", {initialBalance: NEAR.parse("100 N").toJSON()});

    // Save state for test runs, it is unique for each test
    t.context.worker = worker;
    t.context.accounts = {root, contract, wormhole, deployer};
});

test.afterEach.always(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});

test("Get payload", async t => {
    const {root, contract, wormhole} = t.context.accounts;

    // Initialize the contract
    await root.call(contract, "new", {
        wormhole_core: wormhole,
        foreign_governor_emitter: Array.from(timelockBuffer),
        foreign_chain_id: sepoliaChainId
    });

    const calls: Call[] = [
        {
            contract_id: "wormhole_000.sub_olas.olas_000.testnet",
            deposit: NEAR.parse("0"),
            gas: 5,
            method_name: "version",
            args: Array.from(new Uint8Array([]))
        },
        {
            contract_id: "wormhole_000.sub_olas.olas_000.testnet",
            deposit: NEAR.parse("0"),
            gas: 5,
            method_name: "get_storage_usage",
            args: Array.from(new Uint8Array([]))
        }
    ];

    console.log("Calls:", calls);

    // Get call bytes
    const data = await root.call(contract, "to_bytes", {
        calls
    }) as Uint8Array;

    console.log("Payload:", data);

    // Convert `data` to a hex string
    const hexString = Buffer.from(data).toString("hex");
    console.log("Hex string:", hexString);
});

test("View functions", async t => {
    const {root, contract, wormhole} = t.context.accounts;

    // Initialize the contract
    await root.call(contract, "new", {
        wormhole_core: wormhole,
        foreign_governor_emitter: Array.from(timelockBuffer),
        foreign_chain_id: sepoliaChainId
    });

    // Get foreign emitter
    const emitter = await contract.view("get_foreign_governor_emitter", {});

    // Get foreign chain Id
    const chainId = await contract.view("get_foreign_chain_id", {});
});

test("VAA processing", async t => {
    const {root, contract, wormhole, deployer} = t.context.accounts;

    // Corresponding calls above: wormhole.version() and wormhole.get_storage_usage()
    // NOTE: change generic wormhole_000.sub_olas.olas_000.testnet in calls to wormhole_core
    const vaa = "0100000000010034b1959a6ae5645e12d8f3f1c41aa721c762f4c913fa3013ad67acf20e42af180c4e0807dc1f7c1d0897066666d402cc306540f0f83f5518e757bbbd0f78651d01673c9698000000002712000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d0000000000000018005b7b22636f6e74726163745f6964223a22776f726d686f6c655f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a2230222c22676173223a352c226d6574686f645f6e616d65223a2276657273696f6e222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22776f726d686f6c655f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a2230222c22676173223a352c226d6574686f645f6e616d65223a226765745f73746f726167655f7573616765222c2261726773223a5b5d7d5d";
    // https://wormholescan.io/#/tx/0x26c1c08f7534e9de17401a5e28ce449c9bd525864f603ed4a247e3dd665f8d00?network=Testnet&view=overview

    // Initialize wormhole
    await root.call(wormhole, "new", {
        guardian_set_index: 0
    });

    // Initialize the contract
    await root.call(contract, "new", {
        wormhole_core: wormhole,
        foreign_governor_emitter: Array.from(timelockBuffer),
        foreign_chain_id: sepoliaChainId
    });

    let storage = await contract.view("get_storage_usage", {});
    console.log("Storage usage before delivery:", storage);

    const attachedDeposit = "1 N";
    const res: CallResult = await root.call(contract, "delivery", {
        vaa
    }, {attachedDeposit, gas: "300 Tgas"});

    storage = await contract.view("get_storage_usage", {});
    console.log("Storage usage after delivery:", storage);

    // Get the result field
    const result = res.result;

    // Ensure result is not null before converting
    if (result !== null) {
        // Convert `result` to `Uint8Array` and then decode it
        const strOut = new TextDecoder().decode(new Uint8Array(result));
        console.log("Decoded string:", strOut);
    } else {
        console.log("Result is null (Rust's None)");
    }
});