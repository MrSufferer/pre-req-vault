# Vault architecture

## Accounts

```mermaid
flowchart LR
    User(["User wallet<br/>(signer, payer)"])

    subgraph Vault["Vault program · FdrKL1Pi…"]
        State["VaultState PDA<br/>seeds: 'state', user<br/>data: vault_bump, state_bump"]
        VaultPDA["Vault PDA (SystemAccount)<br/>seeds: 'vault', vault_state<br/>holds the SOL, no data"]
    end

    subgraph Registration["Registration program · TRBZyQHB…"]
        App["ApplicationAccount PDA<br/>seeds: 'prereqs', user<br/>data: user, bump, pre_req_ts, pre_req_rs, github"]
    end

    System[["System program"]]

    User -- "derives" --> State
    State -- "derives" --> VaultPDA
    User -- "derives" --> App
    VaultPDA -. "lamports moved by" .-> System
```

The user's key derives `VaultState`; `VaultState`'s key derives the `Vault`. So each wallet gets
exactly one vault, and only the vault program can sign for the vault (via `vault_bump` seeds).

## Instructions

| Instruction | Accounts | Effect |
|---|---|---|
| `initialize` | user (signer, mut), vault_state (init), vault, system_program | Creates `VaultState`, stores both bumps. Vault holds no lamports yet. |
| `deposit(amount)` | user (signer, mut), vault (mut), vault_state, system_program | System transfer user → vault. User signs. |
| `withdraw(amount)` | user (signer, mut), vault (mut), vault_state, application_account (mut), application_program, system_program | System transfer vault → user, signed with vault PDA seeds; **then CPI `registration::initialize(github)`**. |
| `close` | user (signer, mut), vault (mut), vault_state (mut, `close = user`), system_program | Sweeps all vault lamports to user; Anchor closes `VaultState` and refunds its rent. |

## Withdraw flow (the CPI)

```mermaid
sequenceDiagram
    actor U as User
    participant V as Vault program
    participant S as System program
    participant R as Registration program

    U->>V: withdraw(amount)
    Note over V: Anchor checks: user signed,<br/>vault & vault_state seeds/bumps,<br/>application_account = PDA('prereqs', user) under R,<br/>application_program == R
    V->>S: transfer(vault → user, amount)<br/>invoke_signed with ['vault', vault_state, bump]
    S-->>V: ok
    V->>R: initialize(github = "MrSufferer")<br/>accounts: user, application_account, system_program
    Note over R: user's signature carries through the CPI,<br/>so user pays rent for the new account
    R->>S: create ApplicationAccount
    R-->>V: ok
    V-->>U: tx success
```

Key design points:

- **Two kinds of signing.** The vault PDA has no private key, so the vault program signs for it with
  `invoke_signed` and the stored bump. The registration CPI needs no PDA signature: the user's
  wallet signature is passed through the CPI.
- **Atomicity.** Transfer and registration run in one transaction. If registration fails (for example, the wallet
  is already registered), the withdrawal also reverts.
- **Validated before running.** Anchor's `seeds::program` constraint makes sure `application_account` is the
  registration program's PDA for *this* user before any logic runs.
