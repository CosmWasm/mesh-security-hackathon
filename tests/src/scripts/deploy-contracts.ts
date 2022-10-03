import { assert, SigningCosmWasmClient, toBinary } from "cosmwasm";

import { InstantiateMsg as ConsumerInitMsg } from "../bindings/MeshConsumer.types";
import { InstantiateMsg as LockupInitMsg } from "../bindings/MeshLockup.types";
import { MeshProviderClient } from "../bindings/MeshProvider.client";
import { InstantiateMsg as ProviderInitMsg } from "../bindings/MeshProvider.types";
import { InstantiateMsg as StakingInitMsg } from "../bindings/MetaStaking.types";

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
    mesh_lockup: "./internal/mesh_lockup.wasm",
    mesh_provider: "./internal/mesh_provider.wasm",
    mesh_slasher: "./internal/mesh_slasher.wasm",
  };
  const wasmIds = await setupContracts(client, signer, providerContracts);

  const initMeshLockup: LockupInitMsg = { denom };
  const { contractAddress: meshLockupAddr } = await client.instantiate(
    signer,
    wasmIds.mesh_lockup,
    initMeshLockup,
    "mesh_lockup contract",
    "auto"
  );

  const initMeshProvider: ProviderInitMsg = {
    consumer: {
      connection_id: connectionId,
    },
    slasher: {
      code_id: wasmIds.mesh_slasher,
      msg: toBinary({
        owner: signer,
      }),
    },
    lockup: meshLockupAddr,
    // TODO: get real number somehow... look at tendermint client queries or staking?
    unbonding_period: 86400 * 14,
  };
  const { contractAddress: meshProviderAddr } = await client.instantiate(
    signer,
    wasmIds.mesh_provider,
    initMeshProvider,
    "mesh_provider contract",
    "auto"
  );
  const { ibcPortId: meshProviderPort } = await client.getContract(meshProviderAddr);
  assert(meshProviderPort);

  // query the newly created slasher
  const providerClient = new MeshProviderClient(client, signer, meshProviderAddr);
  const { slasher } = await providerClient.config();
  const meshSlasherAddr = assert(slasher);

  return { meshLockupAddr, meshProviderAddr, meshProviderPort, meshSlasherAddr };
}

async function installConsumer(
  client: SigningCosmWasmClient,
  signer: string,
  {
    connectionId,
    providerPortId,
  }: {
    connectionId: string;
    providerPortId: string;
  }
): Promise<ConsumerInfo> {
  console.debug("Upload contracts to consumer...");
  const consumerContracts = {
    mesh_consumer: "./internal/mesh_consumer.wasm",
    meta_staking: "./internal/meta_staking.wasm",
  };
  const wasmIds = await setupContracts(client, signer, consumerContracts);

  // instantiate meta_staking on wasmd
  const initMetaStaking: StakingInitMsg = {};
  const { contractAddress: metaStakingAddr } = await client.instantiate(
    signer,
    wasmIds.meta_staking,
    initMetaStaking,
    "meta_staking contract",
    "auto"
  );

  // instantiate mesh_consumer on wasmd
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
    initMeshConsumer,
    "mesh_consumer contract",
    "auto"
  );
  const { ibcPortId: meshConsumerPort } = await client.getContract(meshConsumerAddr);
  assert(meshConsumerPort);

  return { metaStakingAddr, meshConsumerAddr, meshConsumerPort };
}

async function main() {
  const mnemonic = getMnemonic();
  const provider = await connect(mnemonic, osmoTestConfig);
  const consumer = await connect(mnemonic, junoTestConfig);

  const connectProvToCons = connections[osmoTestConfig.chainId][junoTestConfig.chainId];
  const connectConsToProv = connections[junoTestConfig.chainId][osmoTestConfig.chainId];
  if (connectProvToCons === undefined || connectConsToProv === undefined) {
    throw Error("Connection not found");
  }

  const provInfo = await installProvider(provider.client, provider.address, {
    connectionId: connectProvToCons,
    denom: osmoTestConfig.feeToken,
  });
  pprint(provInfo);
  const consInfo = await installConsumer(consumer.client, consumer.address, {
    connectionId: connectConsToProv,
    providerPortId: provInfo.meshProviderPort,
  });
  pprint(consInfo);
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
