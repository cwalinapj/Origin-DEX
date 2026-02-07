use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

declare_id!("Orig1nDex111111111111111111111111111111111");

#[program]
pub mod origin_dex {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let config = &mut ctx.accounts.config;
        if config.initialized {
            return err!(DexError::AlreadyInitialized);
        }
        config.admin = ctx.accounts.admin.key();
        config.bump = *ctx.bumps.get("config").ok_or(DexError::MissingBump)?;
        config.initialized = true;
        Ok(())
    }

    pub fn set_admin(ctx: Context<SetAdmin>, new_admin: Pubkey) -> Result<()> {
        let config = &mut ctx.accounts.config;
        require_keys_eq!(config.admin, ctx.accounts.admin.key(), DexError::Unauthorized);
        config.admin = new_admin;
        Ok(())
    }

    pub fn init_registry(ctx: Context<InitRegistry>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        if registry.initialized {
            return err!(DexError::AlreadyInitialized);
        }
        registry.admin = ctx.accounts.admin.key();
        registry.bump = *ctx.bumps.get("registry").ok_or(DexError::MissingBump)?;
        registry.next_pool_id = 0;
        registry.initialized = true;
        Ok(())
    }

    pub fn create_pool(
        ctx: Context<CreatePool>,
        fee_bps: u16,
        token_a_price_cents: u64,
        token_b_price_cents: u64,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        require_keys_eq!(registry.admin, ctx.accounts.admin.key(), DexError::Unauthorized);

        if fee_bps > 10_000 {
            return err!(DexError::InvalidFee);
        }

        if ctx.accounts.token_a_mint.freeze_authority.is_none()
            || ctx.accounts.token_b_mint.freeze_authority.is_none()
        {
            return err!(DexError::MintNotFrozen);
        }

        let bin_spacing_milli_cents =
            compute_bin_spacing_milli_cents(token_a_price_cents, token_b_price_cents)?;

        let pool = &mut ctx.accounts.pool;
        pool.pool_id = registry.next_pool_id;
        pool.creator = ctx.accounts.admin.key();
        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.token_a_frozen = ctx.accounts.token_a_mint.freeze_authority.is_some();
        pool.token_b_frozen = ctx.accounts.token_b_mint.freeze_authority.is_some();
        pool.fee_bps = fee_bps;
        pool.house_fee_bps = fee_bps.saturating_mul(5) / 100;
        pool.lp_fee_bps = pool.fee_bps.saturating_sub(pool.house_fee_bps);
        pool.bin_spacing_milli_cents = bin_spacing_milli_cents;
        pool.bump = *ctx.bumps.get("pool").ok_or(DexError::MissingBump)?;
        registry.next_pool_id = registry
            .next_pool_id
            .checked_add(1)
            .ok_or(DexError::Overflow)?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Config::SIZE,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    pub admin: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitRegistry<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + Registry::SIZE,
        seeds = [b"registry"],
        bump
    )]
    pub registry: Account<'info, Registry>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(
        mut,
        seeds = [b"registry"],
        bump = registry.bump,
    )]
    pub registry: Account<'info, Registry>,

    #[account(
        init,
        payer = admin,
        space = 8 + Pool::SIZE,
        seeds = [b"pool", &registry.next_pool_id.to_le_bytes()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    pub token_a_mint: Account<'info, Mint>,
    pub token_b_mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct Config {
    pub admin: Pubkey,
    pub bump: u8,
    pub initialized: bool,
}

impl Config {
    pub const SIZE: usize = 32 + 1 + 1;
}

#[account]
pub struct Registry {
    pub admin: Pubkey,
    pub bump: u8,
    pub next_pool_id: u64,
    pub initialized: bool,
}

impl Registry {
    pub const SIZE: usize = 32 + 1 + 8 + 1;
}

#[account]
pub struct Pool {
    pub pool_id: u64,
    pub creator: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_a_frozen: bool,
    pub token_b_frozen: bool,
    pub fee_bps: u16,
    pub lp_fee_bps: u16,
    pub house_fee_bps: u16,
    pub bin_spacing_milli_cents: u64,
    pub bump: u8,
}

impl Pool {
    pub const SIZE: usize = 8 + 32 + 32 + 32 + 1 + 1 + 2 + 2 + 2 + 8 + 1;
}

#[error_code]
pub enum DexError {
    #[msg("Config already initialized")]
    AlreadyInitialized,
    #[msg("Missing PDA bump")]
    MissingBump,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid fee parameters")]
    InvalidFee,
    #[msg("Token mint must be frozen")]
    MintNotFrozen,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Invalid price inputs")]
    InvalidPrice,
}

fn compute_bin_spacing_milli_cents(
    token_a_price_cents: u64,
    token_b_price_cents: u64,
) -> Result<u64> {
    if token_a_price_cents == 0 || token_b_price_cents == 0 {
        return err!(DexError::InvalidPrice);
    }
    // Bin spacing is the average price (in cents) expressed in milli-cents.
    // Examples:
    // - $1.00 (100 cents) => 1000 milli-cents (1 cent)
    // - $0.50 (50 cents) => 500 milli-cents (0.5 cents)
    // - $10.00 (1000 cents) => 10000 milli-cents (10 cents)
    let sum = token_a_price_cents
        .checked_add(token_b_price_cents)
        .ok_or(DexError::Overflow)?;
    let avg_cents = sum / 2;
    avg_cents
        .checked_mul(10)
        .ok_or(DexError::Overflow)
}
