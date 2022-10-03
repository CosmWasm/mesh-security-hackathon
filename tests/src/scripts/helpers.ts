import { readFileSync } from "fs";
import { env } from "process";

import { makeCosmoshubPath } from "@cosmjs/amino";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

import { Network } from "./networks";

export const pprint = (x: unknown) => console.log(JSON.stringify(x, undefined, 2));

// Check "MNEMONIC" env variable and ensure it is set to a reasonable value
export function getMnemonic(): string {
  const mnemonic = env["MNEMONIC"];
  if (!mnemonic || mnemonic.length < 48) {
    throw new Error("Must set MNEMONIC to a 12 word phrase");
  }
  return mnemonic;
}

export async function connect(mnemonic: string, network: Network) {
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

export async function setupContracts(
  client: SigningCosmWasmClient,
  signer: string,
  contracts: Record<string, string>
): Promise<Record<string, number>> {
  const results: Record<string, number> = {};

  for (const name in contracts) {
    const path = contracts[name];
    console.info(`Storing ${name} from ${path}...`);
    const wasm = await readFileSync(path);
    const receipt = await client.upload(signer, wasm, "auto", `Upload ${name}`);
    console.debug(`Upload ${name} with CodeID: ${receipt.codeId}`);
    results[name] = receipt.codeId;
  }

  return results;
}
