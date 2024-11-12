import {Worker, NEAR, NearAccount} from "near-workspaces";
import anyTest, {TestFn} from "ava";

const test = anyTest as TestFn<{
    worker: Worker;
    accounts: Record<string, NearAccount>;
}>;

test.beforeEach(async t => {
    // Init the worker and start a Sandbox server
    const worker = await Worker.init();

    // Prepare sandbox for tests, create accounts, deploy contracts, etx.
    const root = worker.rootAccount;
    // Deploy the main registry contract
    const contract = await root.devDeploy(
        "target/wasm32-unknown-unknown/release/governor_near.wasm",
        {initialBalance: NEAR.parse("20 N").toJSON()},
    );
//    // Deploy the test token contract
//    const token = await root.devDeploy(
//        "artifacts/test_token.wasm",
//        {initialBalance: NEAR.parse("10 N").toJSON()},
//    );

    // Allocate accounts
    const deployer = await root.createSubAccount("deployer", {initialBalance: NEAR.parse("100 N").toJSON()});

//    // Initialize token contract
//    await root.call(token, "new", {attachedDeposit: NEAR.parse("1 N")});
//
//    // Mint tokens
//    await root.call(token, "mint", {
//        account_id: root.accountId,
//        amount: NEAR.parse("100 N")
//    }, {attachedDeposit: NEAR.parse("1 N")});

    // Save state for test runs, it is unique for each test
    t.context.worker = worker;
    t.context.accounts = {root, contract, deployer};
    //t.context.accounts = {root, contract, token, deployer};
});

test.afterEach.always(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});

test("Get payload", async t => {
    const {root, contract, deployer} = t.context.accounts;

    // This corresponds to Sepolia timelock address 000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d or 0x471b3f60f08c50dd0ecba1bcd113b66fcc02b63d
    const timelockBuffer = new Uint8Array([
        0,   0,  0,   0,   0,   0,   0,   0,   0,
        0,   0,  0,  71,  27,  63,  96, 240, 140,
        80, 221, 14, 203, 161, 188, 209,  19, 182,
        111, 204,  2, 182,  61
    ]);

    // Initialize the contract
    await root.call(contract, "new", {
        owner_id: deployer,
        wormhole_core: deployer,
        foreign_governor_address: Array.from(timelockBuffer)
    });

//    // Get call bytes
//    const attachedDeposit = "5 N";
//    await root.call(contract, "to_bytes", {
//        calls: vec
//    });
});