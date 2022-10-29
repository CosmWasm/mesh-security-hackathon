import { CosmWasmSigner, Link, testutils } from "@confio/relayer";
import { toBinary } from "@cosmjs/cosmwasm-stargate";
import { StargateClient } from "@cosmjs/stargate";
import { assert } from "@cosmjs/utils";
import test from "ava";
import { Order } from "cosmjs-types/ibc/core/channel/v1/channel";

// eslint-disable-next-line import/order
import { MeshProviderClient } from "./bindings/MeshProvider.client";

const pprint = (x: unknown) => console.log(JSON.stringify(x, undefined, 2));

const { osmosis: oldOsmo, setup, wasmd } = testutils;
const osmosis = { ...oldOsmo, minFee: "0.025uosmo" };

import { MetaStakingClient } from "./bindings/MetaStaking.client";
import {
  assertPacketsFromA,
  assertPacketsFromB,
  hash,
  IbcVersion,
  setupContracts,
  setupOsmosisClient,
  setupOsmoStargateClient,
  setupWasmClient,
  setupWasmStargateClient,
  sleepBlocks,
  subCoins,
} from "./utils";

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
  osmoStargateClient: StargateClient;
  wasmStargateClient: StargateClient;
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
    wasmPort: string;
    osmo: string;
    osmoPort: string;
  };
}

async function demoSetup(): Promise<SetupInfo> {
  // create a connection and channel
  const [src, dest] = await setup(wasmd, osmosis);
  const link = await Link.createWithNewConnections(src, dest);
  const osmoClient = await setupOsmosisClient();
  const wasmClient = await setupWasmClient();

  const osmoStargateClient = await setupOsmoStargateClient();
  const wasmStargateClient = await setupWasmStargateClient();

  // instantiate mesh_lockup on osmosis
  const initMeshLockup = { denom: osmosis.denomStaking };
  const { contractAddress: osmoMeshLockup } = await osmoClient.sign.instantiate(
    osmoClient.senderAddress,
    osmosisIds.mesh_lockup,
    initMeshLockup,
    "mesh_lockup contract",
    "auto"
  );

  // create a ics20 channel on this connection
  // We need to pass it to contracts
  const ics20Info = await link.createChannel("A", wasmd.ics20Port, osmosis.ics20Port, Order.ORDER_UNORDERED, "ics20-1");
  const ics20 = {
    wasm: ics20Info.src.channelId,
    wasmPort: ics20Info.src.portId,
    osmo: ics20Info.dest.channelId,
    osmoPort: ics20Info.dest.portId,
  };

  const ibcDenom = "ibc/" + hash(`${ics20.osmoPort}/${ics20.osmo}/ucosm`);

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
    // 0 second unbonding here so we can test it
    unbonding_period: 0,
    rewards_ibc_denom: ibcDenom,
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
    ics20_channel: ics20.osmo,
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

  return {
    wasmClient,
    osmoClient,
    osmoStargateClient,
    wasmStargateClient,
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
  const osmoClient = await setupOsmosisClient();
  const wasmClient = await setupWasmClient();

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
    lockup: osmoClient.senderAddress,
    unbonding_period: 86400 * 7,
    rewards_ibc_denom: "",
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
      // this is not the meshProviderPort, so authentication will reject it
      port_id: "connection-123456",
      connection_id: link.endA.connectionID,
    },
    remote_to_local_exchange_rate: "0.1",
    meta_staking_contract_address: wasmMetaStaking,
    ics20_channel: "channel-10",
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

test.serial("Happy Path (cross-stake / cross-unstake)", async (t) => {
  const {
    wasmClient,
    osmoClient,
    wasmStargateClient,
    osmoStargateClient,
    wasmMeshConsumer,
    osmoMeshProvider,
    osmoMeshLockup,
    wasmMetaStaking,
    link,
    ics20,
  } = await demoSetup();

  console.log("addresses: ", osmoMeshProvider, wasmMeshConsumer);

  const fundsAvailableForStaking = { amount: "1000000", denom: "ustake" };
  const ibcDenom = "ibc/" + hash(`${ics20.osmoPort}/${ics20.osmo}/ucosm`);

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
  const lockedTokens = { amount: "500000", denom: "uosmo" };
  const lockupRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshLockup,
    { bond: {} },
    "auto",
    "memo",
    [lockedTokens]
  );
  console.log("Alice locks up 500000uosmo: ", lockupRes);

  // Relay packets to get list of validators from provider
  const relay_info_1 = await link.relayAll();
  assertPacketsFromB(relay_info_1, 1, true);

  // Get list of validators
  const osmoValidators = await osmoClient.sign.queryContractSmart(osmoMeshProvider, { list_validators: {} });
  console.log("List of validators: ", osmoValidators);
  const validatorAddr = osmoValidators.validators[0].address;

  // Grant claim, cross stake 100 tokens to validator on wasmd
  const grantClaimRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshLockup,
    {
      grant_claim: { provider: osmoMeshProvider, amount: "500000", validator: validatorAddr },
    },
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

  // Query Staked tokens remote
  const metaStakingClient = new MetaStakingClient(wasmClient.sign, wasmClient.senderAddress, wasmMetaStaking);
  const remoteStakedTokens = await metaStakingClient.allDelegations({ consumer: wasmMeshConsumer });
  console.log("Remote staked tokens: ");
  pprint(remoteStakedTokens.delegations);

  // Unstake 100 tokens on wasmd
  const unstakeRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshProvider,
    { unstake: { amount: "100", validator: validatorAddr } },
    "auto"
  );
  console.log("Unstake remote tokens: ", unstakeRes);

  // Relay packets to cross-stake
  const relay_info_3 = await link.relayAll();
  assertPacketsFromB(relay_info_3, 1, true);

  const emptyStakedTokenResponse = await osmoClient.sign.queryContractSmart(osmoMeshProvider, {
    account: { address: osmoClient.senderAddress },
  });
  console.log("List of staked tokens on consumer chain: ", emptyStakedTokenResponse);

  /** Start withdraw rewards process */
  const preConsumerbalance = await wasmStargateClient.getBalance(wasmMeshConsumer, "ucosm");
  const preMetabalance = await wasmStargateClient.getBalance(wasmMetaStaking, "ucosm");
  const preProviderbalance = await osmoStargateClient.getBalance(osmoMeshProvider, `ucosm`);

  console.log("Meta:", preMetabalance, "Consumer:", preConsumerbalance, "Provider:", preProviderbalance);

  // Withdraw rewards from validator
  const resWithdrawReward = await metaStakingClient.withdrawDelegatorReward({
    validator: validatorAddr,
  });
  const preSendMetabalance = await wasmClient.sign.getBalance(wasmMetaStaking, "ucosm");

  console.log("Withdraw amount:", resWithdrawReward.logs[0].events[5].attributes[0].value);
  console.log("Pre-send meta balance:", preSendMetabalance);

  const rewardsTosend = subCoins(preSendMetabalance, preMetabalance);

  // Sleep 20 blocks
  await sleepBlocks(20, wasmStargateClient);

  // withdraw from meta-staking to consumer to provider
  const resWithdrawToConsumer = await metaStakingClient.withdrawToCostumer({
    validator: validatorAddr,
    consumer: wasmMeshConsumer,
  });

  console.log("Withdraw to consumer response: ", resWithdrawToConsumer);

  // Relay our packets to provider
  const relay_info_4 = await link.relayAll();
  assertPacketsFromA(relay_info_4, 1, true);
  console.log("Relay info: ", relay_info_4);

  // Try to relay again to see if something changed
  const relay_info_5 = await link.relayAll();
  assertPacketsFromB(relay_info_5, 0, true);

  // Log balances
  const metaStakingBalance = await wasmStargateClient.getBalance(wasmMetaStaking, "ucosm");
  const meshConsumerBalance = await wasmStargateClient.getBalance(wasmMeshConsumer, "ucosm");
  let meshProviderBalance = await osmoStargateClient.getBalance(osmoMeshProvider, ibcDenom);

  // Verify meta has same value as before send (we sent the right amount)
  t.is(metaStakingBalance.amount, preMetabalance.amount);
  // Verify consumer is empty (no funds are "stuck")
  t.is(meshConsumerBalance.amount, "0");
  // Verify provider got same balance we sent.
  t.is(meshProviderBalance.amount, rewardsTosend.amount);

  // Do rewards withdraw from provider to the sender
  const meshProviderClient = new MeshProviderClient(osmoClient.sign, osmoClient.senderAddress, osmoMeshProvider);
  const withdrawToUser = await meshProviderClient.claimRewards({ validator: validatorAddr });

  console.log("Withdraw to user:", withdrawToUser);

  const senderBalance = await osmoStargateClient.getBalance(osmoClient.senderAddress, ibcDenom);
  meshProviderBalance = await osmoStargateClient.getBalance(osmoMeshProvider, ibcDenom);

  // Verify provider is empty
  t.is(meshProviderBalance.amount, "0");
  // Verify sender got his funds
  t.is(senderBalance.amount, rewardsTosend.amount);

  // Make another tx to advanace the block
  await osmoClient.sign.execute(osmoClient.senderAddress, osmoMeshLockup, { bond: {} }, "auto", "memo", [lockedTokens]);

  // Unbond 100 tokens from wasmd now that a block has passed
  const unbondRes = await osmoClient.sign.execute(
    osmoClient.senderAddress,
    osmoMeshLockup,
    { unbond: { amount: "100" } },
    "auto"
  );
  console.log("Unbond tokens response: ", unbondRes);

  // Check balance
  const balances = await osmoStargateClient.getAllBalances(osmoClient.senderAddress);
  console.log("Alice balances: ", balances);

  // If we made it through everything, we win
  t.assert(true);
});
