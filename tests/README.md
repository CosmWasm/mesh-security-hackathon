# Mesh Security IBC tests

These tests leverage `ts-relayer` to test mesh-security contract interactions between two real blockchain binaries.

## Setup

Ensure you have node 14+ (16+ recommended):

```
node --version
```

Then install via npm as typical:

```
npm install
```

## Development

Build the source:

```
npm run build
```

Clean it up with prettier and eslint:

```
npm run fix
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
npm run test
```
