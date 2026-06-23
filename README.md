# fractionaxapp/onchain

The **on-chain tier** of [FractionAX](https://github.com/fractionaxapp/fractionaxapp):
the Solana programs (written with [Anchor](https://www.anchor-lang.com)) that make
agent decisions enforceable on-chain.

> **This repo is a submodule** of the
> [`fractionaxapp`](https://github.com/fractionaxapp/fractionaxapp) meta-monorepo,
> mounted at `onchain/`. Unlike the JS/Python projects, it is **not** orchestrated
> by moon/pnpm — it builds on its own Rust + Solana + Anchor toolchain.

## What it is

| Program | Path | What it is |
| --- | --- | --- |
| `fractionax` | `programs/fractionax` | Program registry (M1 scaffold). Extended in M2 with RWA vaults, SPL minting, an investor registry, and yield distribution. |

The TypeScript client (`@fractionax/solana` in the meta-repo) consumes this
program's ID and IDL once deployed.

## Develop

```bash
# Prerequisites: Rust, the Solana CLI, and Anchor 0.31.1 (via avm).
anchor build           # compile the program + generate the IDL
anchor keys sync       # set the real program IDs before the first deploy
anchor deploy --provider.cluster devnet
```

> The program IDs in `Anchor.toml` and `declare_id!` are placeholders (the System
> Program address) until `anchor keys sync` is run.
