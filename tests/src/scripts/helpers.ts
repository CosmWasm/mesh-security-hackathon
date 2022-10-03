import { env } from "process";

import { DirectSecp256k1HdWallet, makeCosmoshubPath, SigningCosmWasmClient } from "cosmwasm";

import { junoTestConfig, Network, osmoTestConfig } from "./networks";

const pprint = (x: unknown) => console.log(JSON.stringify(x, undefined, 2));

// Check "MNEMONIC" env variable and ensure it is set to a reasonable value
function getMnemonic(): string {
    const mnemonic = env["MNEMONIC"];
    if (!mnemonic || mnemonic.length < 48) {
        throw new Error("Must set MNEMONIC to a 12 word phrase");
    }
    return mnemonic;
}

async function connect(mnemonic: string, network: Network) {
    const { prefix, gasPrice, feeToken, rpcEndpoint } = network;
    const hdPath = makeCosmoshubPath(0);

    // Setup signer
    const offlineSigner = await DirectSecp256k1HdWallet.fromMnemonic(mnemonic, { prefix, hdPaths: [hdPath] });
    const { address } = (await offlineSigner.getAccounts())[0];
    console.log(`Connected to ${address}`);
    
    // Init SigningCosmWasmClient client
    const client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, offlineSigner, {
        prefix,
        gasPrice,
    });
    const balance = await client.getBalance(address, feeToken);
    console.log(`Balance: ${balance.amount} ${balance.denom}`);
  
    const chainId = await client.getChainId();
  
    if (chainId !== network.chainId) {
        throw Error("Given ChainId doesn't match the clients ChainID!");
    }
  
    return { client, address };
}

// // sees if it is time to call
// async function checkTrigger(client: SigningCosmWasmClient) {
//     const { config } = await client.queryContractSmart(distroAddr, {config:{}});
//     pprint(config);
//     const elapsed = Date.now() / 1000 - config.last_payment;
//     if (elapsed < config.epoch) {
//         console.log(`Next epoch comes in ${config.epoch - elapsed} seconds`);
//         return false;
//     } else {
//         return true;
//     }
// }

async function main() {
    const mnemonic = getMnemonic();
    const provider = await connect(mnemonic, osmoTestConfig);
    const consumer = await connect(mnemonic, junoTestConfig);
    // if (await checkTrigger(client)) {
    //     await pingTrigger(client, address);
    // }
}

main().then(
    () => {
        process.exit(0);
    },
    (error) => {
        console.error(error);
        process.exit(1);
    },
);