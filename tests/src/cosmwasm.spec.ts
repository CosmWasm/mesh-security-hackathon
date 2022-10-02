import { CosmWasmSigner, Link, testutils } from "@confio/relayer";
import { toBinary } from "@cosmjs/cosmwasm-stargate";
import { assert } from "@cosmjs/utils";
import test from "ava";
import { Order } from "cosmjs-types/ibc/core/channel/v1/channel";

const { osmosis: oldOsmo, setup, wasmd } = testutils;
const osmosis = { ...oldOsmo, minFee: "0.025uosmo" };

import { IbcVersion, setupContracts, setupOsmosisClient, setupWasmClient } from "./utils";

let wasmIds: Record<string, number> = {};
let osmosisIds: Record<string, number> = {};

test.before(async (t) => {
  console.debug("Upload contracts to wasmd...");
  const wasmContracts = {
    mesh_consumer: "./internal/mesh_consumer.wasm",
    meta_staking: "./internal/meta_staking.wasm",
  };
  const wasmSign = await setupWasmClient();
  wasmIds = await setupContracts(wasmSign, wasmContracts);

  console.debug("Upload contracts to osmosis...");
  const osmosisContracts = {
    mesh_ilp: "./internal/mesh_ilp.wasm",
    mesh_provider: "./internal/mesh_provider.wasm",
    mesh_slasher: "./internal/mesh_slasher.wasm",
  };
  const osmosisSign = await setupOsmosisClient();
  osmosisIds = await setupContracts(osmosisSign, osmosisContracts);

  t.pass();
});

interface SetupInfo {
  wasmClient: CosmWasmSigner;
  osmoClient: CosmWasmSigner;
  wasmMeshConsumer: string;
  wasmMetaStaking: string;
  osmoMeshProvider: string;
  osmoMeshSlasher: string;
  osmoMeshIlp: string;
  meshConsumerPort: string;
  meshProviderPort: string;
  link: Link;
  ics20: {
    wasm: string;
    osmo: string;
  };
}

async function demoSetup(): Promise<SetupInfo> {
  // create a connection and channel
  const [src, dest] = await setup(wasmd, osmosis);
  const link = await Link.createWithNewConnections(src, dest);
  const osmoClient = await setupOsmosisClient();

  // instantiate mesh_ilp on osmosis
  const initMeshIlp = { denom: osmosis.denomStaking };
  const { contractAddress: osmoMeshIlp } = await osmoClient.sign.instantiate(
    osmoClient.senderAddress,
    osmosisIds.mesh_ilp,
    initMeshIlp,
    "mesh_ilp contract",
    "auto"
  );

  // instantiate mesh_provider on osmosis
  const initMeshProvider = {
    consumer: {
      connection_id: link.endB.connectionID,
    },
    slasher: {
      code_id: osmosisIds.mesh_slasher,
      msg: toBinary({
        owner: osmoClient.senderAddress,
      }),
    },
    ilp: osmoMeshIlp,
    // TODO: get real number somehow... look at tendermint client queries
    unbonding_period: 86400 * 7,
  };
  const { contractAddress: osmoMeshProvider } = await osmoClient.sign.instantiate(
    osmoClient.senderAddress,
    osmosisIds.mesh_provider,
    initMeshProvider,
    "mesh_provider contract",
    "auto"
  );
  const { ibcPortId: meshProviderPort } = await osmoClient.sign.getContract(osmoMeshProvider);
  assert(meshProviderPort);

  // query the newly created slasher
  const { slasher: osmoMeshSlasher } = await osmoClient.sign.queryContractSmart(osmoMeshProvider, { config: {} });

  // instantiate mesh_consumer on wasmd
  const wasmClient = await setupWasmClient();
  const initMeshConsumer = {
    provider: {
      port_id: meshProviderPort,
      connection_id: link.endA.connectionID,
    },
  };
  const { contractAddress: wasmMeshConsumer } = await wasmClient.sign.instantiate(
    wasmClient.senderAddress,
    wasmIds.mesh_consumer,
    initMeshConsumer,
    "mesh_consumer contract",
    "auto"
  );
  const { ibcPortId: meshConsumerPort } = await wasmClient.sign.getContract(wasmMeshConsumer);
  assert(meshConsumerPort);


  // instantiate meta_staking on wasmd
  const initMetaStaking = {};
  const { contractAddress: wasmMetaStaking } = await wasmClient.sign.instantiate(
    wasmClient.senderAddress,
    wasmIds.meta_staking,
    initMetaStaking,
    "meta_staking contract",
    "auto"
  );

  // Create connection between mesh_consumer and mesh_provider
  await link.createChannel("A", meshConsumerPort, meshProviderPort, Order.ORDER_UNORDERED, IbcVersion);

  // also create a ics20 channel on this connection
  const ics20Info = await link.createChannel("A", wasmd.ics20Port, osmosis.ics20Port, Order.ORDER_UNORDERED, "ics20-1");
  const ics20 = {
    wasm: ics20Info.src.channelId,
    osmo: ics20Info.dest.channelId,
  };

  return {
    wasmClient,
    osmoClient,
    wasmMeshConsumer,
    osmoMeshProvider,
    osmoMeshIlp,
    osmoMeshSlasher,
    wasmMetaStaking,
    meshConsumerPort,
    meshProviderPort,
    link,
    ics20,
  };
}

test.serial("Properly registerd", async (t) => {
  // Setup should complete without error
  await demoSetup();
  t.assert(true);
});

test.serial("Fails to connect a second time", async (t) => {
  const { link, meshConsumerPort, meshProviderPort } = await demoSetup();
  // Create second connection between mesh_consumer and mesh_provider
  try {
    await link.createChannel("A", meshConsumerPort, meshProviderPort, Order.ORDER_UNORDERED, IbcVersion);
  } catch (e) {
    return t.assert((e as Error).message.includes("Contract already has a bound channel"));
  }
  throw Error("Should fail to connect a second time");
});

test.serial("fail if connect from different connect or port", async (t) => {
  // create a connection and channel
  const [src, dest] = await setup(wasmd, osmosis);
  const link = await Link.createWithNewConnections(src, dest);

  // instantiate mesh_provider on osmosis
  const osmoClient = await setupOsmosisClient();
  const initMeshProvider = {
    consumer: {
      connection_id: link.endB.connectionID,
    },
    slasher: {
      code_id: osmosisIds.mesh_slasher,
      msg: toBinary({
        owner: osmoClient.senderAddress,
      }),
    },
  };
  const { contractAddress: osmoMeshProvider } = await osmoClient.sign.instantiate(
    osmoClient.senderAddress,
    osmosisIds.mesh_provider,
    initMeshProvider,
    "mesh_provider contract",
    "auto"
  );
  const { ibcPortId: meshProviderPort } = await osmoClient.sign.getContract(osmoMeshProvider);
  assert(meshProviderPort);

  // instantiate mesh_consumer on wasmd
  const wasmClient = await setupWasmClient();
  const initMeshConsumer = {
    provider: {
      port_id: "connection-123456",
      connection_id: link.endA.connectionID,
    },
  };
  const { contractAddress: wasmMeshConsumer } = await wasmClient.sign.instantiate(
    wasmClient.senderAddress,
    wasmIds.mesh_consumer,
    initMeshConsumer,
    "mesh_consumer contract",
    "auto"
  );
  const { ibcPortId: meshConsumerPort } = await wasmClient.sign.getContract(wasmMeshConsumer);
  assert(meshConsumerPort);

  // Create connection with a different port
  try {
    await link.createChannel("A", meshConsumerPort, meshProviderPort, Order.ORDER_UNORDERED, IbcVersion);
  } catch (e) {
    return t.assert((e as Error).message.includes("Unauthorized"));
  }
  throw Error("Should fail to when connecting with wrong port");
});
