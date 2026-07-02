use anchor_lang::prelude::*;

// Program ID of the devnet deployment (set by `anchor keys sync`). For a new
// cluster/keypair, run `anchor keys sync` again after `anchor build`.
declare_id!("Aqvk9Br2PPoTzGZbnYVxnwgpGTzPZTdcowpN9gdkRXGP");

/// Fractionax on-chain program.
///
/// M1 scaffold: a singleton `Registry` PDA (seeds `["registry"]`) that an admin
/// authority initializes and that tracks how many deals have been registered.
///
/// M3 compliance slice: a per-investor `InvestorCredential` PDA (seeds
/// `["investor", wallet]`) that the compliance authority writes after off-chain
/// KYC/AML screening, plus `assert_compliant` — the gated enforcement seam that a
/// real invest/transfer instruction requires before moving value. Later milestones
/// extend this with SPL token minting, vaults, and yield distribution.
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

    /// Write or update an investor's compliance credential (upsert), keyed by the
    /// investor's wallet. Gated to the registry authority (the compliance signer):
    /// the Compliance Agent decides eligibility off-chain and mirrors the result
    /// here so on-chain instructions can enforce it. Re-screening overwrites the
    /// prior credential.
    pub fn set_investor_credential(
        ctx: Context<SetInvestorCredential>,
        jurisdiction: [u8; 2],
        kyc_verified: bool,
        accredited: bool,
        sanctions_clear: bool,
    ) -> Result<()> {
        let credential = &mut ctx.accounts.credential;
        credential.wallet = ctx.accounts.investor.key();
        credential.jurisdiction = jurisdiction;
        credential.kyc_verified = kyc_verified;
        credential.accredited = accredited;
        credential.sanctions_clear = sanctions_clear;
        credential.revoked = false;
        credential.bump = ctx.bumps.credential;
        emit!(CredentialSet {
            wallet: credential.wallet,
            kyc_verified,
            accredited,
        });
        Ok(())
    }

    /// Revoke a credential (e.g. failed re-screening or sanctions match). Gated to
    /// the authority; a revoked credential fails `assert_compliant`.
    pub fn revoke_investor_credential(ctx: Context<RevokeInvestorCredential>) -> Result<()> {
        ctx.accounts.credential.revoked = true;
        emit!(CredentialRevoked {
            wallet: ctx.accounts.credential.wallet,
        });
        Ok(())
    }

    /// The gated enforcement seam: succeeds only if the signing investor holds a
    /// compliant credential for the requested action, otherwise errors. A real
    /// invest or secondary-transfer instruction requires this (directly or via CPI)
    /// before moving value — this is how off-chain compliance becomes enforceable
    /// on-chain. `require_accredited` demands accreditation (e.g. Reg D / high-risk).
    pub fn assert_compliant(ctx: Context<AssertCompliant>, require_accredited: bool) -> Result<()> {
        check_credential(&ctx.accounts.credential, require_accredited)?;
        emit!(ComplianceAsserted {
            wallet: ctx.accounts.credential.wallet,
            require_accredited,
        });
        Ok(())
    }
}

/// Pure compliance predicate over a credential — the single source of truth for
/// the on-chain gate, factored out so it is unit-testable without a validator.
fn check_credential(credential: &InvestorCredential, require_accredited: bool) -> Result<()> {
    require!(!credential.revoked, FractionaxError::CredentialRevoked);
    require!(credential.kyc_verified, FractionaxError::KycNotVerified);
    require!(credential.sanctions_clear, FractionaxError::SanctionsHit);
    if require_accredited {
        require!(
            credential.accredited,
            FractionaxError::AccreditationRequired
        );
    }
    Ok(())
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

#[derive(Accounts)]
pub struct SetInvestorCredential<'info> {
    #[account(seeds = [b"registry"], bump = registry.bump, has_one = authority)]
    pub registry: Account<'info, Registry>,
    #[account(
        init_if_needed,
        payer = authority,
        space = 8 + InvestorCredential::INIT_SPACE,
        seeds = [b"investor", investor.key().as_ref()],
        bump
    )]
    pub credential: Account<'info, InvestorCredential>,
    /// CHECK: the investor the credential is issued for; used only as a PDA seed
    /// and stored as `credential.wallet`. Not required to sign — the authority
    /// writes the credential on the investor's behalf after off-chain KYC.
    pub investor: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RevokeInvestorCredential<'info> {
    #[account(seeds = [b"registry"], bump = registry.bump, has_one = authority)]
    pub registry: Account<'info, Registry>,
    /// CHECK: seed reference for the credential PDA being revoked.
    pub investor: UncheckedAccount<'info>,
    #[account(
        mut,
        seeds = [b"investor", investor.key().as_ref()],
        bump = credential.bump
    )]
    pub credential: Account<'info, InvestorCredential>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AssertCompliant<'info> {
    // The credential PDA is bound to the signer's key, so the investor proves they
    // own the wallet the credential was issued for.
    #[account(
        seeds = [b"investor", investor.key().as_ref()],
        bump = credential.bump
    )]
    pub credential: Account<'info, InvestorCredential>,
    pub investor: Signer<'info>,
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

/// Per-investor compliance credential written by the authority after off-chain
/// KYC/AML. `jurisdiction` is an ISO 3166-1 alpha-2 code (2 bytes).
#[account]
#[derive(InitSpace)]
pub struct InvestorCredential {
    pub wallet: Pubkey,
    pub jurisdiction: [u8; 2],
    pub kyc_verified: bool,
    pub accredited: bool,
    pub sanctions_clear: bool,
    pub revoked: bool,
    pub bump: u8,
}

#[event]
pub struct DealRegistered {
    pub deal_id: String,
    pub count: u64,
}

#[event]
pub struct CredentialSet {
    pub wallet: Pubkey,
    pub kyc_verified: bool,
    pub accredited: bool,
}

#[event]
pub struct CredentialRevoked {
    pub wallet: Pubkey,
}

#[event]
pub struct ComplianceAsserted {
    pub wallet: Pubkey,
    pub require_accredited: bool,
}

#[error_code]
pub enum FractionaxError {
    #[msg("deal_id must be at most 64 characters")]
    DealIdTooLong,
    #[msg("deal_count overflow")]
    Overflow,
    #[msg("investor credential has been revoked")]
    CredentialRevoked,
    #[msg("investor KYC is not verified")]
    KycNotVerified,
    #[msg("investor failed sanctions screening")]
    SanctionsHit,
    #[msg("this action requires an accredited investor")]
    AccreditationRequired,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn credential(
        kyc_verified: bool,
        sanctions_clear: bool,
        accredited: bool,
        revoked: bool,
    ) -> InvestorCredential {
        InvestorCredential {
            wallet: Pubkey::default(),
            jurisdiction: *b"MY",
            kyc_verified,
            accredited,
            sanctions_clear,
            revoked,
            bump: 0,
        }
    }

    #[test]
    fn compliant_credential_passes() {
        assert!(check_credential(&credential(true, true, false, false), false).is_ok());
    }

    #[test]
    fn unverified_kyc_is_rejected() {
        assert!(check_credential(&credential(false, true, false, false), false).is_err());
    }

    #[test]
    fn sanctions_hit_is_rejected() {
        assert!(check_credential(&credential(true, false, false, false), false).is_err());
    }

    #[test]
    fn revoked_credential_is_rejected() {
        assert!(check_credential(&credential(true, true, true, true), false).is_err());
    }

    #[test]
    fn accreditation_gate_enforced() {
        // Retail investor fails when accreditation is required...
        assert!(check_credential(&credential(true, true, false, false), true).is_err());
        // ...but an accredited investor clears the same gate.
        assert!(check_credential(&credential(true, true, true, false), true).is_ok());
    }
}
