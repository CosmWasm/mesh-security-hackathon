import { SigningCosmWasmClient, toBinary } from "@cosmjs/cosmwasm-stargate";
import { assert } from "@cosmjs/utils";

import { InstantiateMsg as ConsumerInitMsg } from "../bindings/MeshConsumer.types";
import { InstantiateMsg as LockupInitMsg } from "../bindings/MeshLockup.types";
import { MeshProviderClient } from "../bindings/MeshProvider.client";
import { InstantiateMsg as ProviderInitMsg } from "../bindings/MeshProvider.types";
import { Coin, InstantiateMsg as StakingInitMsg } from "../bindings/MetaStaking.types";

import { junoConfig } from "./config";
import { connect, getMnemonic, pprint, setupContracts } from "./helpers";
import { connections, junoTestConfig, osmoTestConfig } from "./networks";

interface ProviderInfo {
  meshLockupAddr: string;
  meshProviderAddr: string;
  meshProviderPort: string;
  meshSlasherAddr: string;
}
interface ConsumerInfo {
  metaStakingAddr: string;
  meshConsumerAddr: string;
  meshConsumerPort: string;
}

async function installProvider(
  client: SigningCosmWasmClient,
  signer: string,
  {
    connectionId,
    denom,
  }: {
    connectionId: string;
    denom: string;
  }
): Promise<ProviderInfo> {
  console.debug("Upload contracts to provider...");
  const providerContracts = {
    mesh_lockup: "./src/contracts/mesh_lockup.wasm",
    mesh_provider: "./src/contracts/mesh_provider.wasm",
    mesh_slasher: "./src/contracts/mesh_slasher.wasm",
  };
  const wasmIds = await setupContracts(client, signer, providerContracts);

  console.log("Instantiate mesh_lockup on provider");
  const initMeshLockup: LockupInitMsg = { denom };
  const { contractAddress: meshLockupAddr } = await client.instantiate(
    signer,
    wasmIds.mesh_lockup,
    initMeshLockup as any,
    "mesh_lockup contract",
    "auto"
  );

  console.log("Instantiate provider contract");
  const initMeshProvider: ProviderInitMsg = {
    consumer: {
      connection_id: connectionId,
    },
    slasher: {
      code_id: wasmIds.mesh_slasher,
      msg: toBinary({ owner: signer }),
    },
    lockup: meshLockupAddr,
    // TODO: get real number somehow... look at tendermint client queries or staking?
    unbonding_period: 86400 * 14,
  };
  const { contractAddress: meshProviderAddr } = await client.instantiate(
    signer,
    wasmIds.mesh_provider,
    initMeshProvider as any,
    "mesh_provider contract",
    "auto"
  );
  const { ibcPortId: meshProviderPort } = await client.getContract(meshProviderAddr);
  assert(meshProviderPort);

  console.log("query the newly created slasher");
  const providerClient = new MeshProviderClient(client, signer, meshProviderAddr);
  const { slasher: meshSlasherAddr } = await providerClient.config();
  if (!meshSlasherAddr) {
    throw new Error("Can't find slasher");
  }

  return { meshLockupAddr, meshProviderAddr, meshProviderPort, meshSlasherAddr };
}

async function installConsumer(
  client: SigningCosmWasmClient,
  signer: string,
  {
    connectionId,
    providerPortId,
    fundsAvailableForStaking,
  }: {
    connectionId: string;
    providerPortId: string;
    fundsAvailableForStaking: Coin;
  }
): Promise<ConsumerInfo> {
  console.debug("Upload contracts to consumer...");
  const consumerContracts = {
    mesh_consumer: "./src/contracts/mesh_consumer.wasm",
    meta_staking: "./src/contracts/meta_staking.wasm",
  };
  const wasmIds = await setupContracts(client, signer, consumerContracts);

  console.log("instantiate meta_staking on wasmd");
  const initMetaStaking: StakingInitMsg = {};
  const { contractAddress: metaStakingAddr } = await client.instantiate(
    signer,
    wasmIds.meta_staking,
    initMetaStaking as any,
    "meta_staking contract",
    "auto"
  );

  console.log("instantiate mesh_consumer on wasmd");
  const initMeshConsumer: ConsumerInitMsg = {
    provider: {
      port_id: providerPortId,
      connection_id: connectionId,
    },
    remote_to_local_exchange_rate: "0.3",
    meta_staking_contract_address: metaStakingAddr,
  };
  const { contractAddress: meshConsumerAddr } = await client.instantiate(
    signer,
    wasmIds.mesh_consumer,
    initMeshConsumer as any,
    "mesh_consumer contract",
    "auto"
  );
  const { ibcPortId: meshConsumerPort } = await client.getContract(meshConsumerAddr);
  assert(meshConsumerPort);

  // Fund meta staking module
  console.log("Funding the meta-staking contract: ");
  await client.sendTokens(signer, metaStakingAddr, [fundsAvailableForStaking], "auto");

  // Add consumer to meta-staking contract
  console.log("Add consumer to wasmd meta-staking contract");
  await client.execute(
    signer,
    metaStakingAddr,
    {
      sudo: {
        add_consumer: {
          consumer_address: meshConsumerAddr,
          funds_available_for_staking: fundsAvailableForStaking,
        },
      },
    },
    "auto"
  );

  return { metaStakingAddr, meshConsumerAddr, meshConsumerPort };
}

async function fixConsumer(
  client: SigningCosmWasmClient,
  signer: string,
  {
    fundsAvailableForStaking,
    meshConsumerAddr,
    metaStakingAddr,
  }: {
    metaStakingAddr: string;
    meshConsumerAddr: string;
    fundsAvailableForStaking: Coin;
  }
): Promise<void> {
  // console.log("Remove consumer to wasmd meta-staking contract");
  // await client.execute(
  //   signer,
  //   metaStakingAddr,
  //   {
  //     sudo: {
  //       remove_consumer: {
  //         consumer_address: meshConsumerAddr,
  //       },
  //     },
  //   },
  //   "auto"
  // );

  console.log(`Add consumer to wasmd meta-staking contract: ${fundsAvailableForStaking}`);
  await client.execute(
    signer,
    metaStakingAddr,
    {
      sudo: {
        add_consumer: {
          consumer_address: meshConsumerAddr,
          funds_available_for_staking: fundsAvailableForStaking,
        },
      },
    },
    "auto"
  );
}

async function main() {
  const mnemonic = getMnemonic();
  const [providerConfig, consumerConfig] = [osmoTestConfig, junoTestConfig];
  const provider = await connect(mnemonic, providerConfig);
  const consumer = await connect(mnemonic, consumerConfig);

  const connectProvToCons = connections[providerConfig.chainId][consumerConfig.chainId];
  const connectConsToProv = connections[consumerConfig.chainId][providerConfig.chainId];
  if (connectProvToCons === undefined || connectConsToProv === undefined) {
    throw Error("Connection not found");
  }

  // const provInfo = await installProvider(provider.client, provider.address, {
  //   connectionId: connectProvToCons,
  //   denom: providerConfig.feeToken,
  // });
  // pprint(provInfo);
  // const consInfo = await installConsumer(consumer.client, consumer.address, {
  //   connectionId: connectConsToProv,
  //   providerPortId: provInfo.meshProviderPort,
  //   fundsAvailableForStaking: { denom: consumerConfig.feeToken, amount: "1000000" },
  // });
  // pprint(consInfo);

  await fixConsumer(consumer.client, consumer.address, {
    fundsAvailableForStaking: { amount: "6500000", denom: "ujunox" },
    meshConsumerAddr: junoConfig.meshConsumerAddr,
    metaStakingAddr: junoConfig.metaStakingAddr,
  });

  // TODO: ibc connection
  // // Create connection between mesh_consumer and mesh_provider
  // await link.createChannel("A", meshConsumerPort, meshProviderPort, Order.ORDER_UNORDERED, IbcVersion);
  // // also create a ics20 channel on this connection
  // const ics20Info = await link.createChannel("A", wasmd.ics20Port, osmosis.ics20Port, Order.ORDER_UNORDERED, "ics20-1");
}

main().then(
  () => {
    process.exit(0);
  },
  (error) => {
    console.error(error);
    process.exit(1);
  }
);
