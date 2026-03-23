use {
    crate::state::{BasketConfig, BasketVault},
    quasar_lang::prelude::*,
    quasar_spl::{Mint, Token, TokenCpi},
};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    pub user: &'info mut Signer,
    pub basket: &'info Account<BasketConfig>,
    pub mint: &'info Account<Mint>,
    pub user_ta: &'info mut Account<Token>,
    // The vault PDA signs token transfers out of its token account.
    #[account(
        mut,
        has_one = user,
        has_one = mint,
        seeds = [b"basket-vault", basket, user, mint],
        bump = vault.bump
    )]
    pub vault: &'info mut Account<BasketVault>,
    pub vault_ta: &'info mut Account<Token>,
    pub token_program: &'info Program<Token>,
}

impl<'info> Withdraw<'info> {
    #[inline(always)]
    pub fn withdraw(&self, amount: u64, bumps: &WithdrawBumps) -> Result<(), ProgramError> {
        let seeds = bumps.vault_seeds();
        self.token_program
            .transfer(self.vault_ta, self.user_ta, self.vault, amount)
            .invoke_signed(&seeds)
    }
}
