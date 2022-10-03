// data from https://github.com/cosmos/chain-registry/tree/master/testnets
import { GasPrice } from "@cosmjs/stargate";

export interface Network {
  chainId: string;
  rpcEndpoint: string;
  prefix: string;
  gasPrice: GasPrice;
  feeToken: string;
}

export const junoTestConfig: Network = {
  chainId: "uni-5",
  rpcEndpoint: "https://juno-testnet-rpc.polkachu.com:443",
  prefix: "juno",
  gasPrice: GasPrice.fromString("0.05ujunox"),
  feeToken: "ujunox",
};

export const osmoTestConfig: Network = {
  chainId: "osmo-test-4",
  rpcEndpoint: "https://osmosis-testnet-rpc.allthatnode.com:26657",
  prefix: "osmo",
  gasPrice: GasPrice.fromString("0.025uosmo"),
  feeToken: "uosmo",
};

export const starTestConfig: Network = {
  chainId: "elfagar-1",
  rpcEndpoint: "https://rpc.elgafar-1.stargaze-apis.com",
  prefix: "stars",
  gasPrice: GasPrice.fromString("0.04ustars"),
  feeToken: "ustars",
};

// map from (chainId, chainId) to live existing connection
export const connections = {
  [junoTestConfig.chainId]: {
    // TODO: create this connection
    [osmoTestConfig.chainId]: "channel-1110",
  },
  [osmoTestConfig.chainId]: {
    [junoTestConfig.chainId]: "channel-28",
  },
};
