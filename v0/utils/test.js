const { connect, Contract, Account, Connection, keyStores } = require("near-api-js");
const { readFileSync } = require("fs");
const { join } = require("path");

const CONTRACT_NAME = readFileSync(join(__dirname, "../neardev/dev-account")).toString("utf-8");
const config = {
    networkId: "testnet",
    keyStore: new keyStores.UnencryptedFileSystemKeyStore(`${process.env['HOME']}/.near-credentials`), // optional if not signing transactions
    nodeUrl: "https://rpc.testnet.near.org",
    walletUrl: "https://wallet.testnet.near.org",
    helperUrl: "https://helper.testnet.near.org",
    explorerUrl: "https://explorer.testnet.near.org",
};

const order = {
    order_id: Math.random().toString(),
    description: "Description",
    status: "Pending"
};

const GAS = "300000000000000";
const DEPOSIT = "1";
let contract;
// Create order
async function createOrder(){
    return contract.create_order({
        args: { order },
        amount: DEPOSIT,
        gas: GAS
    })
}

// get order
async function getOrder(){
    return contract.get_order({
        order_id: order.order_id,
    });
}

// update order
async function updateOrder(){
    return contract.update_order({
        args: { order: {...order, description: "Changed" } },
        amount: DEPOSIT,
        gas: GAS
    })
}

// run
async function run(){
    const near = await connect(config);
    const account = await near.account(CONTRACT_NAME);
    contract = new Contract(account, CONTRACT_NAME, {
        changeMethods: ["create_order", "update_order"],
        viewMethods: ["get_order"]
    });
    
    const createOrderResult = await createOrder();
    console.log("createOrderResult", createOrderResult);
    let getOrderResult = await getOrder();
    console.log("getOrderResult", getOrderResult);
    const updateOrderResult = await updateOrder();
    console.log("updateOrderResult", updateOrderResult);
    getOrderResult = await getOrder();
    console.log("getOrderResult", getOrderResult);
}

run();