use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

declare_id!("Orig1nDex111111111111111111111111111111111");

pub const TOKEN_KIND_ERC20_PROXY: u8 = 1;
pub const TOKEN_KIND_FIAT_GOLD_PROXY: u8 = 2;
pub const TOKEN_KIND_WRAPPED_SOL: u8 = 3;
pub const TOKEN_KIND_USDC: u8 = 4;
pub const TOKEN_KIND_EUR: u8 = 5;
pub const TOKEN_KIND_COMMODITY_PROXY: u8 = 6;
pub const TOKEN_KIND_NATIVE_TOKEN: u8 = 7;

pub const GUARANTEE_POLICY_FIXED_MINT: u8 = 0;
pub const GUARANTEE_POLICY_USER_CHOICE: u8 = 1;

pub const ASSET_MASK_WSOL: u16 = 1 << 0;
pub const ASSET_MASK_USDC: u16 = 1 << 1;
pub const ASSET_MASK_NATIVE_TOKEN: u16 = 1 << 2;
pub const ASSET_MASK_EUR_TOKEN: u16 = 1 << 3;
pub const ASSET_MASK_FIAT_GOLD_PROXY: u16 = 1 << 4;
pub const ASSET_MASK_COMMODITY_PROXY: u16 = 1 << 5;

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
        token_a_kind: u8,
        token_b_kind: u8,
        guarantee_policy: u8,
        allowed_assets_mask: u16,
        guarantee_mint: Pubkey,
    ) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        require_keys_eq!(registry.admin, ctx.accounts.admin.key(), DexError::Unauthorized);

        if fee_bps > 10_000 {
            return err!(DexError::InvalidFee);
        }

        validate_token_kind(
            token_a_kind,
            &ctx.accounts.token_a_mint,
            "token_a",
        )?;
        validate_token_kind(
            token_b_kind,
            &ctx.accounts.token_b_mint,
            "token_b",
        )?;

        validate_guarantee_policy(guarantee_policy, allowed_assets_mask, guarantee_mint)?;

        let bin_spacing_milli_cents =
            compute_bin_spacing_milli_cents(token_a_price_cents, token_b_price_cents)?;

        let pool = &mut ctx.accounts.pool;
        pool.pool_id = registry.next_pool_id;
        pool.creator = ctx.accounts.admin.key();
        pool.token_a_mint = ctx.accounts.token_a_mint.key();
        pool.token_b_mint = ctx.accounts.token_b_mint.key();
        pool.token_a_kind = token_a_kind;
        pool.token_b_kind = token_b_kind;
        pool.token_a_frozen = ctx.accounts.token_a_mint.freeze_authority.is_some();
        pool.token_b_frozen = ctx.accounts.token_b_mint.freeze_authority.is_some();
        pool.fee_bps = fee_bps;
        pool.house_fee_bps = fee_bps.saturating_mul(5) / 100;
        pool.lp_fee_bps = pool.fee_bps.saturating_sub(pool.house_fee_bps);
        pool.bin_spacing_milli_cents = bin_spacing_milli_cents;
        pool.guarantee_policy = guarantee_policy;
        pool.allowed_assets_mask = allowed_assets_mask;
        pool.guarantee_mint = guarantee_mint;
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
    pub token_a_kind: u8,
    pub token_b_kind: u8,
    pub token_a_frozen: bool,
    pub token_b_frozen: bool,
    pub fee_bps: u16,
    pub lp_fee_bps: u16,
    pub house_fee_bps: u16,
    pub bin_spacing_milli_cents: u64,
    pub guarantee_policy: u8,
    pub allowed_assets_mask: u16,
    pub guarantee_mint: Pubkey,
    pub bump: u8,
}

impl Pool {
    pub const SIZE: usize = 8
        + 32
        + 32
        + 32
        + 1
        + 1
        + 1
        + 1
        + 2
        + 2
        + 2
        + 8
        + 1
        + 2
        + 32
        + 1;
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
    #[msg("Token mint must not be frozen")]
    MintMustBeUnfrozen,
    #[msg("Invalid token kind")]
    InvalidTokenKind,
    #[msg("Invalid guarantee policy")]
    InvalidGuaranteePolicy,
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

fn validate_token_kind(kind: u8, mint: &Account<Mint>, label: &str) -> Result<()> {
    match kind {
        // 1 = ERC20 proxy (frozen required)
        1 => {
            if mint.freeze_authority.is_none() {
                msg!("{} mint must be frozen for ERC20 proxy", label);
                return err!(DexError::MintNotFrozen);
            }
        }
        // 2 = Fiat/Gold proxy (unfrozen required)
        2 => {
            if mint.freeze_authority.is_some() {
                msg!("{} mint must not be frozen for fiat/gold proxy", label);
                return err!(DexError::MintMustBeUnfrozen);
            }
        }
        // 3 = Wrapped SOL (no freeze requirement)
        3 => {}
        // 4 = USDC (frozen required)
        4 => {
            if mint.freeze_authority.is_none() {
                msg!("{} mint must be frozen for USDC", label);
                return err!(DexError::MintNotFrozen);
            }
        }
        // 5 = EUR token (frozen required)
        5 => {
            if mint.freeze_authority.is_none() {
                msg!("{} mint must be frozen for EUR token", label);
                return err!(DexError::MintNotFrozen);
            }
        }
        // 6 = Commodity proxy (frozen required)
        6 => {
            if mint.freeze_authority.is_none() {
                msg!("{} mint must be frozen for commodity proxy", label);
                return err!(DexError::MintNotFrozen);
            }
        }
        // 7 = Native token (frozen required)
        7 => {
            if mint.freeze_authority.is_none() {
                msg!("{} mint must be frozen for native token", label);
                return err!(DexError::MintNotFrozen);
            }
        }
        _ => return err!(DexError::InvalidTokenKind),
    }

    Ok(())
}

fn validate_guarantee_policy(
    policy: u8,
    allowed_assets_mask: u16,
    guarantee_mint: Pubkey,
) -> Result<()> {
    match policy {
        // 0 = fixed guarantee mint
        0 => {
            if guarantee_mint == Pubkey::default() || allowed_assets_mask != 0 {
                return err!(DexError::InvalidGuaranteePolicy);
            }
        }
        // 1 = user choice from allowed assets mask
        1 => {
            if allowed_assets_mask == 0 || guarantee_mint != Pubkey::default() {
                return err!(DexError::InvalidGuaranteePolicy);
            }
        }
        _ => return err!(DexError::InvalidGuaranteePolicy),
    }
    Ok(())
}
