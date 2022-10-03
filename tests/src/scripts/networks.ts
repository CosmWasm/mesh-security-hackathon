// data from https://github.com/cosmos/chain-registry/tree/master/testnets
import { GasPrice } from "cosmwasm";

export interface Network {
  chainId: string;
  rpcEndpoint: string;
  prefix: string;
  gasPrice: GasPrice;
  feeToken: string;
}

export const junoTestConfig: Network = {
  chainId: "uni-5",
  rpcEndpoint: "https://rpc.uni.junomint.com",
  prefix: "juno",
  gasPrice: GasPrice.fromString("0.05ujunox"),
  feeToken: "ujunox",
};

export const osmoTestConfig: Network = {
  chainId: "osmo-test-4",
  rpcEndpoint: "https://osmosistest-rpc.quickapi.com/",
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
    [osmoTestConfig.chainId]: "",
  },
  [osmoTestConfig.chainId]: {
    [junoTestConfig.chainId]: "",
  },
};
