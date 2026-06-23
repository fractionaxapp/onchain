use anchor_lang::prelude::*;

// Placeholder program ID (System Program address). Run `anchor keys sync`
// after the first build to replace it with the keypair-derived ID.
declare_id!("11111111111111111111111111111111");

/// Fractionax on-chain program.
///
/// M1 scaffold: a single program-owned `Registry` account. Later milestones
/// extend this program with RWA vaults, SPL token minting, an investor
/// registry, and yield distribution.
#[program]
pub mod fractionax {
    use super::*;

    /// Initialize the program registry, owned by an admin authority.
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        registry.authority = ctx.accounts.authority.key();
        registry.deal_count = 0;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + Registry::INIT_SPACE)]
    pub registry: Account<'info, Registry>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Top-level program state. Tracks the admin authority and how many deals have
/// been registered on-chain.
#[account]
#[derive(InitSpace)]
pub struct Registry {
    pub authority: Pubkey,
    pub deal_count: u64,
}
