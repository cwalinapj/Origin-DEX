use anchor_lang::prelude::*;

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

    pub fn create_pool(ctx: Context<CreatePool>) -> Result<()> {
        let registry = &mut ctx.accounts.registry;
        require_keys_eq!(registry.admin, ctx.accounts.admin.key(), DexError::Unauthorized);

        let pool = &mut ctx.accounts.pool;
        pool.pool_id = registry.next_pool_id;
        pool.creator = ctx.accounts.admin.key();
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
    pub bump: u8,
}

impl Pool {
    pub const SIZE: usize = 8 + 32 + 1;
}

#[error_code]
pub enum DexError {
    #[msg("Config already initialized")]
    AlreadyInitialized,
    #[msg("Missing PDA bump")]
    MissingBump,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Arithmetic overflow")]
    Overflow,
}
