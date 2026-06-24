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
| `fractionax` | `programs/fractionax` | Singleton `Registry` PDA (seeds `["registry"]`) tracking the admin authority and an on-chain deal count, with `initialize` and `register_deal` instructions. Extended in M2 with RWA vaults, SPL minting, an investor registry, and yield distribution. |

The TypeScript client (`@fractionax/solana` in the meta-repo) derives the
`Registry` PDA and reads it; the web app surfaces it at `/onchain`.

## Deployed (devnet)

| | Address |
| --- | --- |
| Program | [`Aqvk9Br2PPoTzGZbnYVxnwgpGTzPZTdcowpN9gdkRXGP`](https://explorer.solana.com/address/Aqvk9Br2PPoTzGZbnYVxnwgpGTzPZTdcowpN9gdkRXGP?cluster=devnet) |
| Registry PDA | [`BKXd6X1Mg2Ab26bk4RLnNTqps4r2fjcbjj5FaKou7CSe`](https://explorer.solana.com/address/BKXd6X1Mg2Ab26bk4RLnNTqps4r2fjcbjj5FaKou7CSe?cluster=devnet) |

`@fractionax/solana` defaults to this program ID, so the web `/onchain` view reads
it live. Override with `FRACTIONAX_PROGRAM_ID` for a different deployment.

## Deploy to devnet

```bash
# Prerequisites: Rust, the Solana CLI, and Anchor 0.31.1 (via avm).
solana-keygen new                       # once, if you have no wallet
solana config set --url devnet
solana airdrop 2                         # fund the deployer

anchor build                             # compile + generate the IDL
anchor keys sync                         # write the real program id into
                                         # declare_id! and Anchor.toml
anchor build                             # rebuild with the synced id
anchor deploy --provider.cluster devnet  # deploy the program

# Initialize the singleton Registry PDA (one-time). With the IDL from
# `anchor build`, call the `initialize` instruction — e.g. via `anchor test`,
# an `anchor run` script, or @coral-xyz/anchor:
#   program.methods.initialize().rpc()
```

Then point the meta-repo at the deployed program so the web `/onchain` view and
`@fractionax/solana` read it live:

```bash
# in the meta-repo (e.g. apps/web/.env.local)
FRACTIONAX_PROGRAM_ID=<the id printed by `anchor keys sync`>
SOLANA_RPC_URL=https://api.devnet.solana.com   # or a private devnet RPC
```

Until then the program id stays the placeholder (System Program address) and the
`/onchain` page shows a "not deployed" state.
