import process from "process";
import {Worker, NEAR, NearAccount} from "near-workspaces";
import {keyStores} from "near-api-js";
import anyTest, {TestFn} from "ava";

interface Call {
    contract_id: string;
    deposit: number;
    method_name: string;
    args: number[];
}

interface CallResult {
    success: boolean;
    result: number[] | null;
}

const govContractName = "gov_000.sub_olas.olas_000.testnet";
const registriesContractName = "contract_000.sub_olas.olas_000.testnet";

const test = anyTest as TestFn<{
    worker: Worker;
}>;

test.before(async t => {
    t.context.worker = await Worker.init({
        homeDir: "/Users/kupermind/.near-credentials",
        network: "testnet",
        rootAccountId: "sub_olas"
    });
});

test.after.always(async t => {
    await t.context.worker.tearDown().catch(error => {
        console.log('Failed to tear down the worker:', error);
    });
});

test("Get payload", async t => {
    const root = t.context.worker.rootAccount;
    const contract = root.getAccount(govContractName);

    const calls: Call[] = [
        {
            contract_id: registriesContractName,
            deposit: 0,
            method_name: "is_paused",
            args: Array.from(new Uint8Array([]))
        },
        {
            contract_id: registriesContractName,
            deposit: 0,
            method_name: "version",
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

test.only("Delivery", async t => {
    const root = t.context.worker.rootAccount;
    const contract = root.getAccount(govContractName);
    // https://wormholescan.io/#/tx/0xfa83667d2226033cfe562ac216875de625fe55b2744fd801a63a8b52645edc64?network=Testnet&view=overview
    const vaa = "01000000000100313e1c858f4f4790a0b172e69420ec195de667209b28a1db20ad1cedbab153c0666fa151482625feb2839eada1bffe6da63919bc010f360d5b816b68c0d07721016734f52c000000002712000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d0000000000000011005b7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a302c226d6574686f645f6e616d65223a2269735f706175736564222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a302c226d6574686f645f6e616d65223a2276657273696f6e222c2261726773223a5b5d7d5d";

    const attachedDeposit = "1 N";
    const res: CallResult = await root.call(contract, "delivery", {
        vaa
    }, {attachedDeposit, gas: "300 Tgas"});

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