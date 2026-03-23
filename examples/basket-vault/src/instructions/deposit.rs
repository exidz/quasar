use {
    crate::state::{BasketConfig, BasketVault},
    quasar_lang::prelude::*,
    quasar_spl::{Mint, Token, TokenCpi},
};

#[derive(Accounts)]
pub struct Deposit<'info> {
    pub user: &'info mut Signer,
    pub basket: &'info Account<BasketConfig>,
    pub mint: &'info Account<Mint>,
    pub user_ta: &'info mut Account<Token>,
    // One vault PDA per (basket, user, mint).
    #[account(init_if_needed, payer = user, seeds = [b"basket-vault", basket, user, mint], bump)]
    pub vault: &'info mut Account<BasketVault>,
    #[account(init_if_needed, payer = user, token::mint = mint, token::authority = vault)]
    pub vault_ta: &'info mut Account<Token>,
    pub rent: &'info Sysvar<Rent>,
    pub token_program: &'info Program<Token>,
    pub system_program: &'info Program<System>,
}

impl<'info> Deposit<'info> {
    #[inline(always)]
    pub fn initialize_vault(&mut self, bumps: &DepositBumps) -> Result<(), ProgramError> {
        self.vault.set_inner(
            *self.basket.address(),
            *self.user.address(),
            *self.mint.address(),
            bumps.vault,
        );
        Ok(())
    }

    #[inline(always)]
    pub fn deposit(&self, amount: u64) -> Result<(), ProgramError> {
        // Reject deposits into mints the basket manager did not allowlist.
        if !self.basket.allows_mint(*self.mint.address()) {
            return Err(ProgramError::InvalidArgument);
        }

        self.token_program
            .transfer(self.user_ta, self.vault_ta, self.user, amount)
            .invoke()
    }
}
