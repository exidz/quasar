# Basket Vault

Minimal SPL token index-fund example tested through the generated Rust and TypeScript client facades.

## What It Shows

- a manager creates a basket with an allowlist of up to three SPL mints
- users deposit only allowlisted SPL tokens
- each vault is unique per `(basket, user, mint)`
- one basket can hold multiple token vaults
- multiple users can each have their own vault for the same basket and mint
- Rust tests call the generated client in `target/client/rust/quasar-basket-vault-client`
- TypeScript tests call the generated kit client in `target/client/typescript/quasar_basket_vault/kit.ts`

## Program Model

- `create_basket`: creates a basket config PDA and stores the allowlisted mints
- `deposit`: creates or reuses the user vault for one basket/mint pair and transfers SPL tokens into the vault token account
- `withdraw`: transfers SPL tokens back out of the PDA-owned vault token account

Important rule:

- deposits fail if the mint is not in the basket allowlist

## How To Run

From the repo root:

```bash
cd /home/exidz/workspaces/quasar
```

### 1. Generate the client

```bash
target/debug/quasar idl examples/basket-vault
```

This creates the Rust client used by the tests at:

```text
target/client/rust/quasar-basket-vault-client
```

### 2. Compile-check the example tests

```bash
cargo test --manifest-path examples/basket-vault/Cargo.toml --no-run
```

### 3. Run the TypeScript kit tests

From the example directory:

```bash
cd examples/basket-vault
npm install
npx vitest run basket-vault.test.ts
cd /home/exidz/workspaces/quasar
```

### 4. Build the SBF program artifact

```bash
cargo build-sbf --manifest-path examples/basket-vault/Cargo.toml
```

This should produce:

```text
target/deploy/quasar_basket_vault.so
```

### 5. Run the Rust runtime tests

```bash
cargo test --manifest-path examples/basket-vault/Cargo.toml
```

## Common Failure

If you see:

```text
Program file not found: ../../target/deploy/quasar_basket_vault.so
```

run the SBF build step first:

```bash
cargo build-sbf --manifest-path examples/basket-vault/Cargo.toml
```

If the TypeScript test dependencies are missing:

```bash
cd examples/basket-vault
npm install
npx vitest run basket-vault.test.ts
```

## Useful Reset

If editor diagnostics look stale:

```bash
cargo clean -p quasar-basket-vault
cargo test --manifest-path examples/basket-vault/Cargo.toml --no-run
```

