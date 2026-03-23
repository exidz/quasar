use quasar_lang::prelude::*;

#[account(discriminator = 1)]
// Basket config: one manager-defined allowlist of mints.
pub struct BasketConfig {
    pub manager: Address,
    pub mint_a: Address,
    pub mint_b: Address,
    pub mint_c: Address,
    pub bump: u8,
}

impl BasketConfig {
    #[inline(always)]
    pub fn allows_mint(&self, mint: Address) -> bool {
        self.mint_a == mint || self.mint_b == mint || self.mint_c == mint
    }
}

#[account(discriminator = 2)]
// Per-user, per-mint vault state inside a basket.
pub struct BasketVault {
    pub basket: Address,
    pub user: Address,
    pub mint: Address,
    pub bump: u8,
}
