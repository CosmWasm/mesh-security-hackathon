# Mesh Security IBC tests

These tests leverage `ts-relayer` to test mesh-security contract interactions between two real blockchain binaries.

## Setup

Ensure you have node 14+ (16+ recommended):

```
node --version
```

Then install via yarn or npm as typical:

```
yarn
```

## Testing

### Build optimized contract WASM

Compile the contracts for uploading.

```sh
./scripts/build_integration_wasm.sh
```

**NOTE: you need to run this each time your contract changes.**

### Run two chains in docker

Start `wasmd` and `osmosisd` chains using docker compose:

```bash
docker-compose up
```

### Run tests

In a separate terminal:

```
yarn test
```

## Development

Clean up test code with prettier and eslint:

```
yarn fix
```
