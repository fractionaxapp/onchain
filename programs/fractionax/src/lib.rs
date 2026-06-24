use anchor_lang::prelude::*;

// Program ID of the devnet deployment (set by `anchor keys sync`). For a new
// cluster/keypair, run `anchor keys sync` again after `anchor build`.
declare_id!("Aqvk9Br2PPoTzGZbnYVxnwgpGTzPZTdcowpN9gdkRXGP");

/// Fractionax on-chain program.
///
/// M1 scaffold: a singleton `Registry` PDA (seeds `["registry"]`) that an admin
/// authority initializes and that tracks how many deals have been registered.
/// Later milestones extend this program with RWA vaults, SPL token minting, an
/// investor registry, and yield distribution.
#[program]
pub mod fractionax {
    use super::*;

    /// Initialize the program registry PDA, owned by an admin authority.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.authority = ctx.accounts.authority.key();
        registry.deal_count = 0;
        registry.bump = ctx.bumps.registry;
        Ok(())
    }

    /// Register a sourced deal on-chain (admin-gated). Increments the registry's
    /// deal counter and emits an event; deal metadata stays off-chain until
    /// tokenization (M2). This is the write seam the Execution Agent will use.
    pub fn register_deal(ctx: Context<RegisterDeal>, deal_id: String) -> Result<()> {
        require!(deal_id.len() <= 64, FractionaxError::DealIdTooLong);
        let registry = &mut ctx.accounts.registry;
        registry.deal_count = registry
            .deal_count
            .checked_add(1)
            .ok_or(FractionaxError::Overflow)?;
        emit!(DealRegistered {
            deal_id,
            count: registry.deal_count,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + Registry::INIT_SPACE,
        seeds = [b"registry"],
        bump
    )]
    pub registry: Account<'info, Registry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterDeal<'info> {
    #[account(mut, seeds = [b"registry"], bump = registry.bump, has_one = authority)]
    pub registry: Account<'info, Registry>,
    pub authority: Signer<'info>,
}

/// Top-level program state: the admin authority, the on-chain deal counter, and
/// the PDA bump.
#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub authority: Pubkey,
    pub deal_count: u64,
    pub bump: u8,
}

#[event]
pub struct DealRegistered {
    pub deal_id: String,
    pub count: u64,
}

#[error_code]
pub enum FractionaxError {
    #[msg("deal_id must be at most 64 characters")]
    DealIdTooLong,
    #[msg("deal_count overflow")]
    Overflow,
}
