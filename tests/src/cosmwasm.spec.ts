import { CosmWasmSigner, Link, testutils } from "@confio/relayer";
import { toBinary } from "@cosmjs/cosmwasm-stargate";
import { assert, sleep } from "@cosmjs/utils";
import test from "ava";
import { Order } from "cosmjs-types/ibc/core/channel/v1/channel";

const { osmosis: oldOsmo, setup, wasmd } = testutils;
const osmosis = { ...oldOsmo, minFee: "0.025uosmo" };

import { assertPacketsFromB, IbcVersion, setupContracts, setupOsmosisClient, setupWasmClient } from "./utils";

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
    mesh_lockup: "./internal/mesh_lockup.wasm",
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
  osmoMeshLockup: string;
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
  const wasmClient = await setupWasmClient();

  // instantiate mesh_lockup on osmosis
  const initMeshLockup = { denom: osmosis.denomStaking };
  const { contractAddress: osmoMeshLockup } = await osmoClient.sign.instantiate(
    osmoClient.senderAddress,
    osmosisIds.mesh_lockup,
    initMeshLockup,
    "mesh_lockup contract",
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
    lockup: osmoMeshLockup,
    // 2 second unbonding here so we can test it
    unbonding_period: 2,
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

  // instantiate meta_staking on wasmd
  const initMetaStaking = {};
  const { contractAddress: wasmMetaStaking } = await wasmClient.sign.instantiate(
    wasmClient.senderAddress,
    wasmIds.meta_staking,
    initMetaStaking,
    "meta_staking contract",
    "auto"
  );

  // instantiate mesh_consumer on wasmd
  const initMeshConsumer = {
    provider: {
      port_id: meshProviderPort,
      connection_id: link.endA.connectionID,
    },
    remote_to_local_exchange_rate: "0.1",
    meta_staking_contract_address: wasmMetaStaking,
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
    osmoMeshLockup,
    osmoMeshSlasher,
    wasmMetaStaking,
    meshConsumerPort,
    meshProviderPort,
    link,
    ics20,
  };
}

// test.serial("Properly registerd", async (t) => {
//   // Setup should complete without error
//   await demoSetup();
//   t.assert(true);
// });

// test.serial("Fails to connect a second time", async (t) => {
//   const { link, meshConsumerPort, meshProviderPort } = await demoSetup();
//   // Create second connection between mesh_consumer and mesh_provider
//   try {
//     await link.createChannel("A", meshConsumerPort, meshProviderPort, Order.ORDER_UNORDERED, IbcVersion);
//   } catch (e) {
//     return t.assert((e as Error).message.includes("Contract already has a bound channel"));
//   }
//   throw Error("Should fail to connect a second time");
// });

// test.serial("fail if connect from different connect or port", async (t) => {
//   // create a connection and channel
//   const [src, dest] = await setup(wasmd, osmosis);
//   const link = await Link.createWithNewConnections(src, dest);
//   const osmoClient = await setupOsmosisClient();
//   const wasmClient = await setupWasmClient();

//   // instantiate mesh_provider on osmosis
//   const initMeshProvider = {
//     consumer: {
//       connection_id: link.endB.connectionID,
//     },
//     slasher: {
//       code_id: osmosisIds.mesh_slasher,
//       msg: toBinary({
//         owner: osmoClient.senderAddress,
//       }),
//     },
//     lockup: osmoClient.senderAddress,
//     unbonding_period: 86400 * 7,
//   };
//   const { contractAddress: osmoMeshProvider } = await osmoClient.sign.instantiate(
//     osmoClient.senderAddress,
//     osmosisIds.mesh_provider,
//     initMeshProvider,
//     "mesh_provider contract",
//     "auto"
//   );
//   const { ibcPortId: meshProviderPort } = await osmoClient.sign.getContract(osmoMeshProvider);
//   assert(meshProviderPort);

//   // instantiate meta_staking on wasmd
//   const initMetaStaking = {};
//   const { contractAddress: wasmMetaStaking } = await wasmClient.sign.instantiate(
//     wasmClient.senderAddress,
//     wasmIds.meta_staking,
//     initMetaStaking,
//     "meta_staking contract",
//     "auto"
//   );

//   // instantiate mesh_consumer on wasmd
//   const initMeshConsumer = {
//     provider: {
//       // this is not the meshProviderPort, so authentication will reject it
//       port_id: "connection-123456",
//       connection_id: link.endA.connectionID,
//     },
//     remote_to_local_exchange_rate: "0.1",
//     meta_staking_contract_address: wasmMetaStaking,
//   };
//   const { contractAddress: wasmMeshConsumer } = await wasmClient.sign.instantiate(
//     wasmClient.senderAddress,
//     wasmIds.mesh_consumer,
//     initMeshConsumer,
//     "mesh_consumer contract",
//     "auto"
//   );
//   const { ibcPortId: meshConsumerPort } = await wasmClient.sign.getContract(wasmMeshConsumer);
//   assert(meshConsumerPort);

//   // Create connection with a different port
//   try {
//     await link.createChannel("A", meshConsumerPort, meshProviderPort, Order.ORDER_UNORDERED, IbcVersion);
//   } catch (e) {
//     return t.assert((e as Error).message.includes("Unauthorized"));
//   }
//   throw Error("Should fail to when connecting with wrong port");
// });

test.serial("happy path", async (t) => {
  const { wasmClient, osmoClient, wasmMeshConsumer, osmoMeshProvider, osmoMeshLockup, wasmMetaStaking, link } =
    await demoSetup();

  const fundsAvailableForStaking = { amount: "100000", denom: "ustake" };

  // Fund meta staking module
  const funding_res = await wasmClient.sign.sendTokens(
    wasmClient.senderAddress,
    wasmMetaStaking,
    [fundsAvailableForStaking],
    "auto"
  );
  console.log("Funding the meta-staking contract: ", funding_res);

  // Add consumer to meta-staking contract
  const add_consumer_res = await wasmClient.sign.execute(
    wasmClient.senderAddress,
    wasmMetaStaking,
    {
      sudo: {
        add_consumer: {
          consumer_address: wasmMeshConsumer,
          funds_available_for_staking: fundsAvailableForStaking,
        },
      },
    },
    "auto"
  );
  console.log("Add consumer to wasmd meta-staking contract: ", add_consumer_res);

  // Lockup 100 tokens on Osmosis
  const lockedTokens = { amount: "100", denom: "uosmo" };
  const lockupRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshLockup,
    { bond: {} },
    "auto",
    "memo",
    [lockedTokens]
  );
  console.log("Alice locks up 100uosmo: ", lockupRes);

  // Relay packets to get list of validators from provider
  const relay_info_1 = await link.relayAll();
  assertPacketsFromB(relay_info_1, 1, true);

  // Get list of validators
  const osmoValidators = await osmoClient.sign.queryContractSmart(osmoMeshProvider, { list_validators: {} });
  console.log("List of validators: ", osmoValidators);

  // Grant claim, cross stake 100 tokens to validator on wasmd
  const grantClaimRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshLockup,
    { grant_claim: { leinholder: osmoMeshProvider, amount: "100", validator: osmoValidators.validators[0].address } },
    "auto"
  );
  console.log("Grant a claim to provider contract (cross-stake): ", grantClaimRes);

  // Relay packets to cross-stake
  const relay_info_2 = await link.relayAll();
  assertPacketsFromB(relay_info_2, 1, true);

  // Query staked tokens
  const stakedTokenResponse = await osmoClient.sign.queryContractSmart(osmoMeshProvider, {
    account: { address: osmoClient.senderAddress },
  });
  console.log("Staked tokens response: ", stakedTokenResponse);

  // Unstake 100 tokens on wasmd
  const unstakeRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshProvider,
    { unstake: { amount: "100", validator: osmoValidators.validators[0].address } },
    "auto"
  );
  console.log("Unstake remote tokens: ", unstakeRes);

  // Relay packets to cross-stake
  const relay_info_3 = await link.relayAll();
  assertPacketsFromB(relay_info_3, 1, true);

  const emptyStakedTokenResponse = await osmoClient.sign.queryContractSmart(osmoMeshProvider, {
    account: { address: osmoClient.senderAddress },
  });
  console.log("Staked tokens response (now empty): ", emptyStakedTokenResponse);

  await t.throws(
    osmoClient.sign.execute(osmoClient.senderAddress, osmoMeshLockup, { unbond: { amount: "100" } }, "auto"),
    /No tokens/
  );

  // unbonding period is 2 seconds, let's wait 2.5 to ensure this works.
  await sleep(2500);

  // Unbond 100 tokens from wasmd now that a block has passed
  const unbondRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshLockup,
    { unbond: { amount: "100" } },
    "auto"
  );
  console.log("Unbond tokens response: ", unbondRes);

  // Check balance
  const balance = await osmoClient.sign.getBalance(osmoClient.senderAddress, "uosmo");
  console.log("Alice balance: ", balance);

  // If we made it through everything, we win
  t.assert(true);
});
