use {
    crate::state::BasketConfig,
    quasar_lang::prelude::*,
};

#[derive(Accounts)]
pub struct CreateBasket<'info> {
    pub manager: &'info mut Signer,
    pub basket_seed: &'info UncheckedAccount,
    // The seed account lets one manager derive many basket configs.
    #[account(init, payer = manager, seeds = [b"basket", manager, basket_seed], bump)]
    pub basket: &'info mut Account<BasketConfig>,
    pub system_program: &'info Program<System>,
}

impl<'info> CreateBasket<'info> {
    #[inline(always)]
    pub fn create(
        &mut self,
        mint_a: Address,
        mint_b: Address,
        mint_c: Address,
        bumps: &CreateBasketBumps,
    ) -> Result<(), ProgramError> {
        // Keep the example simple: up to three allowlisted mints, no duplicates.
        if !has_any_mint(mint_a, mint_b, mint_c) || has_duplicate_non_default_mints(mint_a, mint_b, mint_c) {
            return Err(ProgramError::InvalidArgument);
        }

        self.basket
            .set_inner(*self.manager.address(), mint_a, mint_b, mint_c, bumps.basket);
        Ok(())
    }
}

#[inline(always)]
fn has_any_mint(mint_a: Address, mint_b: Address, mint_c: Address) -> bool {
    mint_a != Address::default() || mint_b != Address::default() || mint_c != Address::default()
}

#[inline(always)]
fn has_duplicate_non_default_mints(mint_a: Address, mint_b: Address, mint_c: Address) -> bool {
    let values = [mint_a, mint_b, mint_c];
    for i in 0..values.len() {
        if values[i] == Address::default() {
            continue;
        }
        for j in (i + 1)..values.len() {
            if values[i] == values[j] {
                return true;
            }
        }
    }
    false
}
