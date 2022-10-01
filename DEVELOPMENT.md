# Developer Tips

## Adding a contract

Use [`cw-template`](https://github.com/CosmWasm/cw-template) to create a new contract.

```shell
cd contracts
cargo generate --git https://github.com/CosmWasm/cw-template.git --name CONTRACT_NAME
cd CONTRACT_NAME
rm -rf LICENSE NOTICE *.md .git* .editorconfig Cargo.lock .circleci
sed -i 's/0.13.2/0.15.1/' Cargo.toml
git add .
```

After this, you will want to add it to the top level `Cargo.toml`, by adding a section
like named `[profile.release.package.CONTRACT_NAME]` that looks like other such sections.
