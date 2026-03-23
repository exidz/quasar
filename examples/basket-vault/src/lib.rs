#![no_std]

use quasar_lang::prelude::*;

mod instructions;
use instructions::*;
mod state;
#[cfg(test)]
mod tests;

declare_id!("55555555555555555555555555555555555555555555");

#[program]
mod quasar_basket_vault {
    use super::*;

    #[instruction(discriminator = 0)]
    // A basket is the allowlisted token set. Users deposit into per-mint vaults
    // beneath that basket, but only for mints configured here.
    pub fn create_basket(
        ctx: Ctx<CreateBasket>,
        mint_a: Address,
        mint_b: Address,
        mint_c: Address,
    ) -> Result<(), ProgramError> {
        ctx.accounts
            .create(mint_a, mint_b, mint_c, &ctx.bumps)
    }

    #[instruction(discriminator = 1)]
    // Deposits are scoped to a single (basket, user, mint) vault.
    pub fn deposit(ctx: Ctx<Deposit>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.initialize_vault(&ctx.bumps)?;
        ctx.accounts.deposit(amount)
    }

    #[instruction(discriminator = 2)]
    // Withdrawals move tokens out of the PDA-owned vault token account.
    pub fn withdraw(ctx: Ctx<Withdraw>, amount: u64) -> Result<(), ProgramError> {
        ctx.accounts.withdraw(amount, &ctx.bumps)
    }
}
