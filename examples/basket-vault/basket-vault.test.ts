import {
  AccountRole,
  address,
  getAddressCodec,
  getProgramDerivedAddress,
} from "@solana/kit";
import { getU64Codec } from "@solana/codecs";
import { describe, expect, it } from "vitest";

import {
  BASKET_CONFIG_DISCRIMINATOR,
  BasketConfigCodec,
  BASKET_VAULT_DISCRIMINATOR,
  BasketVaultCodec,
  CREATE_BASKET_INSTRUCTION_DISCRIMINATOR,
  DEPOSIT_INSTRUCTION_DISCRIMINATOR,
  ProgramInstruction,
  PROGRAM_ADDRESS,
  QuasarBasketVaultClient,
  WITHDRAW_INSTRUCTION_DISCRIMINATOR,
  accounts,
  args,
  instructions,
} from "../../target/client/typescript/quasar_basket_vault/kit";

describe("quasar_basket_vault TypeScript kit client", () => {
  const client = new QuasarBasketVaultClient();
  const amountCodec = getU64Codec();
  const user = address("11111111111111111111111111111111");
  const basket = address("22222222222222222222222222222222222222222222");
  const mint = address("33333333333333333333333333333333333333333333");
  const mintB = address("55555555555555555555555555555555555555555555");
  const mintC = address("77777777777777777777777777777777777777777777");
  const userTa = address("44444444444444444444444444444444444444444444");
  const vaultTa = address("66666666666666666666666666666666666666666666");
  const manager = address("88888888888888888888888888888888888888888888");
  const basketSeed = address("99999999999999999999999999999999999999999999");

  async function deriveBasketAddress() {
    return (
      await getProgramDerivedAddress({
        programAddress: PROGRAM_ADDRESS,
        seeds: [
          new Uint8Array([98, 97, 115, 107, 101, 116]),
          getAddressCodec().encode(manager),
          getAddressCodec().encode(basketSeed),
        ],
      })
    )[0];
  }

  async function deriveVaultAddress() {
    return (
      await getProgramDerivedAddress({
        programAddress: PROGRAM_ADDRESS,
        seeds: [
          new Uint8Array([98, 97, 115, 107, 101, 116, 45, 118, 97, 117, 108, 116]),
          getAddressCodec().encode(basket),
          getAddressCodec().encode(user),
          getAddressCodec().encode(mint),
        ],
      })
    )[0];
  }

  it("builds a createBasket instruction with the derived basket PDA", async () => {
    const createAccounts = { manager, basketSeed } satisfies accounts.CreateBasket;
    const createArgs = { mintA: mint, mintB, mintC } satisfies args.CreateBasket;
    const [ix, directIx, expectedBasket] = await Promise.all([
      instructions.createBasket(createAccounts, createArgs),
      client.createCreateBasketInstruction({ ...createAccounts, ...createArgs }),
      deriveBasketAddress(),
    ]);

    expect(ix.programAddress).toBe(PROGRAM_ADDRESS);
    expect(ix).toEqual(directIx);
    expect(ix.accounts).toEqual([
      { address: manager, role: AccountRole.WRITABLE_SIGNER },
      { address: basketSeed, role: AccountRole.READONLY },
      { address: expectedBasket, role: AccountRole.WRITABLE },
      { address: address("11111111111111111111111111111111"), role: AccountRole.READONLY },
    ]);
    expect(Array.from(ix.data)).toEqual([
      ...CREATE_BASKET_INSTRUCTION_DISCRIMINATOR,
      ...getAddressCodec().encode(mint),
      ...getAddressCodec().encode(mintB),
      ...getAddressCodec().encode(mintC),
    ]);

    const decoded = client.decodeInstruction(ix.data);
    expect(decoded?.type).toBe(ProgramInstruction.CreateBasket);
    expect(decoded && "args" in decoded ? decoded.args : null).toEqual(createArgs);
  });

  it("builds a deposit instruction with the facade and exact fixed accounts", async () => {
    const depositAccounts = {
      user,
      basket,
      mint,
      userTa,
      vaultTa,
    } satisfies accounts.Deposit;
    const depositArgs = { amount: 500_000n satisfies args.Deposit["amount"] };
    const [ix, directIx, expectedVault] = await Promise.all([
      instructions.deposit(depositAccounts, depositArgs),
      client.createDepositInstruction({ ...depositAccounts, ...depositArgs }),
      deriveVaultAddress(),
    ]);

    expect(ix.programAddress).toBe(PROGRAM_ADDRESS);
    expect(ix).toEqual(directIx);
    expect(ix.accounts).toEqual([
      { address: user, role: AccountRole.WRITABLE_SIGNER },
      { address: basket, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: userTa, role: AccountRole.WRITABLE },
      { address: expectedVault, role: AccountRole.WRITABLE },
      { address: vaultTa, role: AccountRole.WRITABLE },
      { address: address("SysvarRent111111111111111111111111111111111"), role: AccountRole.READONLY },
      { address: address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), role: AccountRole.READONLY },
      { address: address("11111111111111111111111111111111"), role: AccountRole.READONLY },
    ]);
    expect(Array.from(ix.data)).toEqual([
      ...DEPOSIT_INSTRUCTION_DISCRIMINATOR,
      ...amountCodec.encode(depositArgs.amount),
    ]);

    const decoded = client.decodeInstruction(ix.data);
    expect(decoded?.type).toBe(ProgramInstruction.Deposit);
    expect(decoded && "args" in decoded ? decoded.args : null).toEqual(depositArgs);
  });

  it("builds a withdraw instruction with the facade and exact fixed accounts", async () => {
    const withdrawAccounts = {
      user,
      basket,
      mint,
      userTa,
      vaultTa,
    } satisfies accounts.Withdraw;
    const withdrawArgs = { amount: 300_000n satisfies args.Withdraw["amount"] };
    const [ix, directIx, expectedVault] = await Promise.all([
      instructions.withdraw(withdrawAccounts, withdrawArgs),
      client.createWithdrawInstruction({ ...withdrawAccounts, ...withdrawArgs }),
      deriveVaultAddress(),
    ]);

    expect(ix.programAddress).toBe(PROGRAM_ADDRESS);
    expect(ix).toEqual(directIx);
    expect(ix.accounts).toEqual([
      { address: user, role: AccountRole.WRITABLE_SIGNER },
      { address: basket, role: AccountRole.READONLY },
      { address: mint, role: AccountRole.READONLY },
      { address: userTa, role: AccountRole.WRITABLE },
      { address: expectedVault, role: AccountRole.WRITABLE },
      { address: vaultTa, role: AccountRole.WRITABLE },
      { address: address("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"), role: AccountRole.READONLY },
    ]);
    expect(Array.from(ix.data)).toEqual([
      ...WITHDRAW_INSTRUCTION_DISCRIMINATOR,
      ...amountCodec.encode(withdrawArgs.amount),
    ]);

    const decoded = client.decodeInstruction(ix.data);
    expect(decoded?.type).toBe(ProgramInstruction.Withdraw);
    expect(decoded && "args" in decoded ? decoded.args : null).toEqual(withdrawArgs);
  });

  it("decodes the basket vault account layout", async () => {
    const vault = await deriveVaultAddress();
    const accountData = Uint8Array.from([
      ...BASKET_VAULT_DISCRIMINATOR,
      ...BasketVaultCodec.encode({ basket, user, mint, bump: 254 }),
    ]);

    expect(client.decodeBasketVault(accountData)).toEqual({
      basket,
      user,
      mint,
      bump: 254,
    });
    expect(vault).toBeTypeOf("string");
  });

  it("decodes the basket config allowlist layout", async () => {
    const basketAddress = await deriveBasketAddress();
    const accountData = Uint8Array.from([
      ...BASKET_CONFIG_DISCRIMINATOR,
      ...BasketConfigCodec.encode({ manager, mintA: mint, mintB, mintC, bump: 17 }),
    ]);

    expect(client.decodeBasketConfig(accountData)).toEqual({
      manager,
      mintA: mint,
      mintB,
      mintC,
      bump: 17,
    });
    expect(basketAddress).toBeTypeOf("string");
  });

  it("derives a different vault for each user in the same basket fund", async () => {
    const secondUser = address("77777777777777777777777777777777777777777777");
    const firstVault = await deriveVaultAddress();
    const secondVault = (
      await getProgramDerivedAddress({
        programAddress: PROGRAM_ADDRESS,
        seeds: [
          new Uint8Array([98, 97, 115, 107, 101, 116, 45, 118, 97, 117, 108, 116]),
          getAddressCodec().encode(basket),
          getAddressCodec().encode(secondUser),
          getAddressCodec().encode(mint),
        ],
      })
    )[0];

    expect(firstVault).not.toBe(secondVault);

    const firstInstruction = await instructions.deposit(
      { user, basket, mint, userTa, vaultTa },
      { amount: 1n },
    );
    const secondInstruction = await instructions.deposit(
      {
        user: secondUser,
        basket,
        mint,
        userTa: address("88888888888888888888888888888888888888888888"),
        vaultTa: address("99999999999999999999999999999999999999999999"),
      },
      { amount: 1n },
    );

    expect(firstInstruction.accounts[4]?.address).toBe(firstVault);
    expect(secondInstruction.accounts[4]?.address).toBe(secondVault);
  });

  it("derives a different vault for multiple tokens in the same basket", async () => {
    const baseVault = await deriveVaultAddress();
    const otherMint = address("BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB");

    const mintVault = (
      await getProgramDerivedAddress({
        programAddress: PROGRAM_ADDRESS,
        seeds: [
          new Uint8Array([98, 97, 115, 107, 101, 116, 45, 118, 97, 117, 108, 116]),
          getAddressCodec().encode(basket),
          getAddressCodec().encode(user),
          getAddressCodec().encode(otherMint),
        ],
      })
    )[0];

    expect(baseVault).not.toBe(mintVault);
  });

  it("derives a different vault when the basket changes", async () => {
    const baseVault = await deriveVaultAddress();
    const otherBasket = address("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");

    const basketVault = (
      await getProgramDerivedAddress({
        programAddress: PROGRAM_ADDRESS,
        seeds: [
          new Uint8Array([98, 97, 115, 107, 101, 116, 45, 118, 97, 117, 108, 116]),
          getAddressCodec().encode(otherBasket),
          getAddressCodec().encode(user),
          getAddressCodec().encode(mint),
        ],
      })
    )[0];

    expect(baseVault).not.toBe(basketVault);
  });

  it("returns null for unknown instruction discriminators", () => {
    expect(client.decodeInstruction(new Uint8Array([255, 1, 2, 3]))).toBeNull();
  });
});
