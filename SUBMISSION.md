# Turbin3 Q4 2026 prerequisite — Vault + registration CPI

`withdraw` now CPIs into the registration program (`TRBZyQHB3m68FGeVsqTK39Wm4xejadjVhP5MAZaKWDM`)
`initialize` instruction, creating the `["prereqs", user]` PDA that records the GitHub handle `MrSufferer`.

- Vault program (devnet): `FdrKL1Pi8Gt3xs7xiwDaqNWB6pQPvDmZHKkiXzVxwmSb`
- Wallet: `ECmGsGAAPJimTwLk3SzkQ39pUQbaBj7U5qgSRRgYSFy`
- Registration account: `EYCPkkuCyqp6Rm1tXxuMY4h1RGwt3tS7c64xQw3QLdDz`

Devnet transactions:

| Step | Signature |
|---|---|
| initialize | [4rYDj3uV…](https://explorer.solana.com/tx/4rYDj3uVfG4cWVr7zJdqroP9vJrtrfYJn4Qmg4YfcZTEJq6U1K8GYCeXSFmFx74Z6SsLtcJoJS4j9s5KydJCKXjo?cluster=devnet) |
| deposit 1 SOL | [28TGEb2Y…](https://explorer.solana.com/tx/28TGEb2YY4JNQgvmnmCtaWpzn9s7AU7D4icAYQ87x1JDwbQzGmsdV6NAtLgVzMDSo77S2uKPfwtELpjryDgg4jyp?cluster=devnet) |
| withdraw 0.5 SOL + registration CPI | [2DfSpRvE…](https://explorer.solana.com/tx/2DfSpRvEoXTSd9CY1USiDJPMx7QMobmrVfsFLgtcz8bVy8bM9bZuicexQqSr2yRAYU4JsrkHfpMELJ7fzWgiFinv?cluster=devnet) |
| close | [2ch9HoUQ…](https://explorer.solana.com/tx/2ch9HoUQXQbPN45vBrMn18gZGR55e8baYembEGSY4ogN2G64m6MUVT1FFWAzMgPTFkXpHEUYBEczgiqf5idTsgV8?cluster=devnet) |

## Run

```sh
cargo build-sbf && cargo test   # LiteSVM test, uses tests/fixtures/registration.so dumped from devnet
anchor deploy --provider.cluster devnet
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com ANCHOR_WALLET=~/.config/solana/id.json \
  pnpm exec ts-mocha -p ./tsconfig.json -t 1000000 "tests/**/*.ts"
```
