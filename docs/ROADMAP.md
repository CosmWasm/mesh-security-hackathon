# Development Roadmap

Created: Apr 17, 2023

## High Level Milestones

### MVP

Target Date: June/July 2023

The first milestone will be design containing all major features in a solid foundation to build on.
It is designed to be launched on two testnets along with a usable UI, so we can begin getting real
feedback from users and relayers, while building out the more complex features and polishing off some
tougher issues.

### V1

Target date: Early Q4 2023 ??

This will include a feature complete version of all code (eg including slashing), but may not have all
extensions (such as remote staking not providing gov voting power). The provider side should
be well reviewed and free from any security holes (safe to deploy on larger chains). The consumer side
(which includes a custom SDK module) will be as solid as possible, but not recommended to run on serious
chains. At this points, Osmosis could provide security to a small market cap chain.

### V2

Target date: Early Q1 2024 ??

This will include all optional features we consider feasible in a realistic time frame and most importantly
will have much deeper security model, and have received some external reviews (maybe audit if someone pays).
This should be stable enough such that chains with solid market caps could provide peer security (being both
provider and consumer).

## Plan for MVP

A higher-level backlog for what it takes to create the MVP.

**Part 1**

* Start new repo for production mesh-security (port ideas from prototype, but we don't need to build on it)
* Finalize the documentation for provider side
* Define all contract interfaces / APIs as Rust files
* Produce stub-contracts with proper APIs (all `unimplemented!()`)

**Part 2** 

* Produce mock contracts with proper APIs (all with dummy testing implementation)
  * Mocks should also be usable for UI testing with eg 1 minute unbonding
* Write and test vault contract (that calls mock local and remote staking via multi-test)
  * Ensure vault accepts both native and cw20 tokens
  * All configuration options should be implemented
* Finalize the documentation for the consumer side, including the custom SDK modules
* Create initial designs (wireframes) for the UI, focusing around vault and local/remote staking 

**Part 3**

* Implement local staking module (simple version - no optional features)
* Implement remote staking module, provider side (simple version - no optional features)
* Implement mock converter, consumer side (connects via IBC properly, but )
* Full stack IBC test from `token -> vault -> remote staking -> converter`, bonding and unbonding
* Usable UI for provider side (mocking out remote providers), with bonding and unbonding
