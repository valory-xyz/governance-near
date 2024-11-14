import process from "process";
import {Worker, NEAR, NearAccount} from "near-workspaces";
import {keyStores} from "near-api-js";
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

    const obj = {
        deposit: NEAR.parse("100 nN"),
        account_id: govContractName
    };

    console.log(obj.deposit);

    const jsonString = JSON.stringify(obj);
    const encoder = new TextEncoder();
    const byteArray = encoder.encode(jsonString);

    const calls: Call[] = [
        {
            contract_id: registriesContractName,
            deposit: NEAR.parse("0"),
            gas: 5,
            method_name: "is_paused",
            args: Array.from(new Uint8Array([]))
        },
        {
            contract_id: registriesContractName,
            deposit: NEAR.parse("0"),
            gas: 5,
            method_name: "version",
            args: Array.from(new Uint8Array([]))
        },
        {
            contract_id: govContractName,
            deposit: obj.deposit,
            gas: 5,
            method_name: "test_payable",
            args: Array.from(byteArray)
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
    //const vaa = "01000000000100313e1c858f4f4790a0b172e69420ec195de667209b28a1db20ad1cedbab153c0666fa151482625feb2839eada1bffe6da63919bc010f360d5b816b68c0d07721016734f52c000000002712000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d0000000000000011005b7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a302c226d6574686f645f6e616d65223a2269735f706175736564222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a302c226d6574686f645f6e616d65223a2276657273696f6e222c2261726773223a5b5d7d5d";

    // https://wormholescan.io/#/tx/0xebc96800331e6c6b309c8612dbddb14bbf078eb3fa80afdfeaf66ad43969399f?network=Testnet&view=overview
    //const vaa = "01000000000100794c1337fbd30a41877fd460f7a7ec3e4d8d68a6dd542f8dc6f5cf292af3301414ef74a2a80afccfcd6d3bf795fe5aea80c8357f960a06a48738a40b49668ab00067362024000000002712000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d0000000000000013005b7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a2230222c226d6574686f645f6e616d65223a2269735f706175736564222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a2230222c226d6574686f645f6e616d65223a2276657273696f6e222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22676f765f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a22313030303030303030303030303030303030303030303030222c226d6574686f645f6e616d65223a22746573745f70617961626c65222c2261726773223a5b3132332c33342c3130302c3130312c3131322c3131312c3131352c3130352c3131362c33342c35382c33342c34392c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c33342c34342c33342c39372c39392c39392c3131312c3131372c3131302c3131362c39352c3130352c3130302c33342c35382c33342c3131362c3131362c39352c35312c34382c34362c3131362c3130312c3131352c3131362c3131302c3130312c3131362c33342c3132355d7d5d";
    // relevant NEAR tx: https://testnet.nearblocks.io/txns/JCtZ5HiRGvJrxF2zF2GvVx9ef7SeeAsYhdFbE1KM8BdJ#

    // https://wormholescan.io/#/tx/0x45c4585ba16c3c4e147e9bbc690954d7455dc6939b220ac0ea5e9a241facbd28?network=Testnet&view=overview
    const vaa = "01000000000100846d56f00dba70ff82c0959bf3558b755c6852e476b4d3e7c05797094923f66e04a4bea07bc78d016ecda25cf129b376dddc66b2c9361f7b20d051964b109d350067364d78000000002712000000000000000000000000471b3f60f08c50dd0ecba1bcd113b66fcc02b63d0000000000000015005b7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a2230222c22676173223a352c226d6574686f645f6e616d65223a2269735f706175736564222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22636f6e74726163745f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a2230222c22676173223a352c226d6574686f645f6e616d65223a2276657273696f6e222c2261726773223a5b5d7d2c7b22636f6e74726163745f6964223a22676f765f3030302e7375625f6f6c61732e6f6c61735f3030302e746573746e6574222c226465706f736974223a22313030303030303030303030303030303030222c22676173223a352c226d6574686f645f6e616d65223a22746573745f70617961626c65222c2261726773223a5b3132332c33342c3130302c3130312c3131322c3131312c3131352c3130352c3131362c33342c35382c33342c34392c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c34382c33342c34342c33342c39372c39392c39392c3131312c3131372c3131302c3131362c39352c3130352c3130302c33342c35382c33342c3130332c3131312c3131382c39352c34382c34382c34382c34362c3131352c3131372c39382c39352c3131312c3130382c39372c3131352c34362c3131312c3130382c39372c3131352c39352c34382c34382c34382c34362c3131362c3130312c3131352c3131362c3131302c3130312c3131362c33342c3132355d7d5d";
    // relevant NEAR tx: https://testnet.nearblocks.io/txns/8GXPQGqGfwZDeGrqHTafYMx7xJKYBhspujjfm3FFg67g

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