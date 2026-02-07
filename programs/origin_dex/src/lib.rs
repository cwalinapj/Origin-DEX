use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

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

pub const HOUSE_FEE_REBATE_BPS: u16 = 0;

pub const FUNCTION_LINEAR: u8 = 1;
pub const FUNCTION_LOG: u8 = 2;

pub const PARAM_SCALE: i64 = 1_000_000;

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
        pool.next_position_id = 0;
        pool.bump = *ctx.bumps.get("pool").ok_or(DexError::MissingBump)?;
        registry.next_pool_id = registry
            .next_pool_id
            .checked_add(1)
            .ok_or(DexError::Overflow)?;
        Ok(())
    }

    pub fn create_lp_position(
        ctx: Context<CreateLpPosition>,
        min_price_cents: u64,
        max_price_cents: u64,
        left_function_type: u8,
        left_params: [i64; 5],
        right_function_type: u8,
        right_params: [i64; 5],
    ) -> Result<()> {
        let pool = &mut ctx.accounts.pool;
        if min_price_cents >= max_price_cents {
            return err!(DexError::InvalidPriceRange);
        }

        validate_function_spec(left_function_type, &left_params)?;
        validate_function_spec(right_function_type, &right_params)?;

        let position = &mut ctx.accounts.position;
        require_keys_eq!(
            position.pool,
            Pubkey::default(),
            DexError::PositionAlreadyInitialized
        );
        position.pool = pool.key();
        position.owner = ctx.accounts.owner.key();
        position.position_id = pool.next_position_id;
        position.lp_mint = ctx.accounts.lp_mint.key();
        position.min_price_cents = min_price_cents;
        position.max_price_cents = max_price_cents;
        position.left_function_type = left_function_type;
        position.right_function_type = right_function_type;
        position.left_params = left_params;
        position.right_params = right_params;
        position.bump = *ctx.bumps.get("position").ok_or(DexError::MissingBump)?;

        pool.next_position_id = pool
            .next_position_id
            .checked_add(1)
            .ok_or(DexError::Overflow)?;

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.owner_lp_token_account.to_account_info(),
                    authority: ctx.accounts.position.to_account_info(),
                },
                &[&[
                    b"position",
                    pool.key().as_ref(),
                    &position.position_id.to_le_bytes(),
                    &[position.bump],
                ]],
            ),
            1,
        )?;

        token::set_authority(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::SetAuthority {
                    current_authority: ctx.accounts.position.to_account_info(),
                    account_or_mint: ctx.accounts.lp_mint.to_account_info(),
                },
                &[&[
                    b"position",
                    pool.key().as_ref(),
                    &position.position_id.to_le_bytes(),
                    &[position.bump],
                ]],
            ),
            token::AuthorityType::MintTokens,
            None,
        )?;

        Ok(())
    }

    pub fn stake_lp_nft(ctx: Context<StakeLpNft>) -> Result<()> {
        require_keys_eq!(ctx.accounts.position.pool, ctx.accounts.pool.key(), DexError::InvalidPosition);
        require_keys_eq!(ctx.accounts.position.owner, ctx.accounts.owner.key(), DexError::Unauthorized);
        let stake = &mut ctx.accounts.stake;
        if stake.active {
            return err!(DexError::AlreadyStaked);
        }
        stake.pool = ctx.accounts.pool.key();
        stake.position = ctx.accounts.position.key();
        stake.owner = ctx.accounts.owner.key();
        stake.staked_at_slot = Clock::get()?.slot;
        stake.rebate_bps = HOUSE_FEE_REBATE_BPS;
        stake.active = true;
        stake.bump = *ctx.bumps.get("stake").ok_or(DexError::MissingBump)?;

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.owner_lp_token_account.to_account_info(),
                    to: ctx.accounts.stake_vault.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            1,
        )?;

        Ok(())
    }

    pub fn unstake_lp_nft(ctx: Context<UnstakeLpNft>) -> Result<()> {
        require_keys_eq!(ctx.accounts.position.pool, ctx.accounts.pool.key(), DexError::InvalidPosition);
        require_keys_eq!(ctx.accounts.position.owner, ctx.accounts.owner.key(), DexError::Unauthorized);
        let stake = &mut ctx.accounts.stake;
        if !stake.active {
            return err!(DexError::NotStaked);
        }
        require_keys_eq!(stake.owner, ctx.accounts.owner.key(), DexError::Unauthorized);
        stake.active = false;

        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::Transfer {
                    from: ctx.accounts.stake_vault.to_account_info(),
                    to: ctx.accounts.owner_lp_token_account.to_account_info(),
                    authority: ctx.accounts.stake.to_account_info(),
                },
                &[&[
                    b"stake",
                    ctx.accounts.position.key().as_ref(),
                    &[stake.bump],
                ]],
            ),
            1,
        )?;

        Ok(())
    }

    pub fn add_liquidity_to_position(ctx: Context<AddLiquidityToPosition>) -> Result<()> {
        require_keys_eq!(ctx.accounts.position.pool, ctx.accounts.pool.key(), DexError::InvalidPosition);
        require_keys_eq!(ctx.accounts.position.owner, ctx.accounts.owner.key(), DexError::Unauthorized);

        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                token::MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.owner_lp_token_account.to_account_info(),
                    authority: ctx.accounts.position.to_account_info(),
                },
                &[&[
                    b"position",
                    ctx.accounts.pool.key().as_ref(),
                    &ctx.accounts.position.position_id.to_le_bytes(),
                    &[ctx.accounts.position.bump],
                ]],
            ),
            1,
        )?;

        Ok(())
    }

    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        require_keys_eq!(ctx.accounts.position.pool, ctx.accounts.pool.key(), DexError::InvalidPosition);
        require_keys_eq!(ctx.accounts.position.owner, ctx.accounts.owner.key(), DexError::Unauthorized);
        if ctx.accounts.stake.active {
            return err!(DexError::AlreadyStaked);
        }

        token::burn(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                token::Burn {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    from: ctx.accounts.owner_lp_token_account.to_account_info(),
                    authority: ctx.accounts.owner.to_account_info(),
                },
            ),
            1,
        )?;

        token::close_account(CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::CloseAccount {
                account: ctx.accounts.owner_lp_token_account.to_account_info(),
                destination: ctx.accounts.owner.to_account_info(),
                authority: ctx.accounts.owner.to_account_info(),
            },
        ))?;

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

#[derive(Accounts)]
pub struct CreateLpPosition<'info> {
    #[account(
        mut,
        seeds = [b"pool", &pool.pool_id.to_le_bytes()],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        init,
        payer = owner,
        space = 8 + Position::SIZE,
        seeds = [b"position", pool.key().as_ref(), &pool.next_position_id.to_le_bytes()],
        bump
    )]
    pub position: Account<'info, Position>,

    #[account(
        init,
        payer = owner,
        mint::decimals = 0,
        mint::authority = position,
        seeds = [b"lp_mint", position.key().as_ref()],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = lp_mint,
        associated_token::authority = owner
    )]
    pub owner_lp_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct StakeLpNft<'info> {
    pub pool: Account<'info, Pool>,
    pub position: Account<'info, Position>,

    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + Stake::SIZE,
        seeds = [b"stake", position.key().as_ref()],
        bump
    )]
    pub stake: Account<'info, Stake>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = lp_mint,
        associated_token::authority = stake
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner_lp_token_account: Account<'info, TokenAccount>,

    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct UnstakeLpNft<'info> {
    pub pool: Account<'info, Pool>,
    pub position: Account<'info, Position>,

    #[account(
        mut,
        seeds = [b"stake", position.key().as_ref()],
        bump = stake.bump,
        close = owner
    )]
    pub stake: Account<'info, Stake>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = stake
    )]
    pub stake_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner_lp_token_account: Account<'info, TokenAccount>,

    pub lp_mint: Account<'info, Mint>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct AddLiquidityToPosition<'info> {
    pub pool: Account<'info, Pool>,
    #[account(
        mut,
        seeds = [b"position", pool.key().as_ref(), &position.position_id.to_le_bytes()],
        bump = position.bump
    )]
    pub position: Account<'info, Position>,

    #[account(
        mut,
        seeds = [b"lp_mint", position.key().as_ref()],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = owner
    )]
    pub owner_lp_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [b"position", pool.key().as_ref(), &position.position_id.to_le_bytes()],
        bump = position.bump,
        close = owner
    )]
    pub position: Account<'info, Position>,

    #[account(
        mut,
        seeds = [b"lp_mint", position.key().as_ref()],
        bump
    )]
    pub lp_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = lp_mint,
        associated_token::authority = owner
    )]
    pub owner_lp_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [b"stake", position.key().as_ref()],
        bump = stake.bump
    )]
    pub stake: Account<'info, Stake>,

    #[account(mut)]
    pub owner: Signer<'info>,

    pub token_program: Program<'info, Token>,
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
    pub next_position_id: u64,
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
        + 8
        + 1;
}

#[account]
pub struct Position {
    pub pool: Pubkey,
    pub owner: Pubkey,
    pub position_id: u64,
    pub lp_mint: Pubkey,
    pub min_price_cents: u64,
    pub max_price_cents: u64,
    pub left_function_type: u8,
    pub right_function_type: u8,
    pub left_params: [i64; 5],
    pub right_params: [i64; 5],
    pub bump: u8,
}

impl Position {
    pub const SIZE: usize = 32 + 32 + 8 + 32 + 8 + 8 + 1 + 1 + (8 * 5) + (8 * 5) + 1;
}

#[account]
pub struct Stake {
    pub pool: Pubkey,
    pub position: Pubkey,
    pub owner: Pubkey,
    pub staked_at_slot: u64,
    pub rebate_bps: u16,
    pub active: bool,
    pub bump: u8,
}

impl Stake {
    pub const SIZE: usize = 32 + 32 + 32 + 8 + 2 + 1 + 1;
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
    #[msg("Already staked")]
    AlreadyStaked,
    #[msg("Not staked")]
    NotStaked,
    #[msg("Invalid position")]
    InvalidPosition,
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Invalid price inputs")]
    InvalidPrice,
    #[msg("Position already initialized")]
    PositionAlreadyInitialized,
    #[msg("Invalid price range")]
    InvalidPriceRange,
    #[msg("Invalid function spec")]
    InvalidFunctionSpec,
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

fn validate_function_spec(function_type: u8, params: &[i64; 5]) -> Result<()> {
    match function_type {
        FUNCTION_LINEAR => {
            // f(x) = m(x - x0) + y0
            // params: [m, x0, y0, unused, unused]
            let _m = params[0];
            let _x0 = params[1];
            let _y0 = params[2];
        }
        FUNCTION_LOG => {
            // g(x) = A * log_B(C(-x + h)) + k
            // params: [A, B, C, h, k]
            if params[1] == 0 || params[2] == 0 {
                return err!(DexError::InvalidFunctionSpec);
            }
        }
        _ => return err!(DexError::InvalidFunctionSpec),
    }

    Ok(())
}
