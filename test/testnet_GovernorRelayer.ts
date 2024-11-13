import process from "process";
import {Worker, NEAR, NearAccount} from "near-workspaces";
import {keyStores} from "near-api-js";
import anyTest, {TestFn} from "ava";

interface Call {
    contract_id: string;
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
            method_name: "is_paused",
            args: Array.from(new Uint8Array([]))
        },
        {
            contract_id: registriesContractName,
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
    // https://wormholescan.io/#/tx/0x5d3ad033f79a576af5115c39607dcb49776df38ff4a02b0eeb69866b45d6df4a?network=Testnet&view=overview
    const vaa = "01000000000100c868d05d4b56ac72663f2b6a0cb77a2886b1d52d2fa4e20555c16f4f77f419e120d2d612a06eb2c10d1f39bdc708770b38df6948d8b25c1cdd67e4f514e5df85006734d768000000002712000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d0000000000000010005b7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226d6574686f645f6e616d65223a2269735f706175736564222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226d6574686f645f6e616d65223a2276657273696f6e222c2261726773223a5b5d7d5d";
    //const failedVaa = "01000000000100dda0db566a5112ae22e824fb4a4eafc56106a64f7862c7f1b926138f9a685636729be3047f17392e2d4040b05852282f009cf9f3d5c3cfaf4a1573f1b9522d14006734b77c000000002712000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d000000000000000f005b7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226d6574686f645f6e616d65223a22706175736564222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226d6574686f645f6e616d65223a2276657273696f6e222c2261726773223a5b5d7d5d";

    const res: CallResult = await root.call(contract, "delivery", {
        vaa
    }, {gas: "300 Tgas"});

    console.log(res);

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