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

#[account]
pub struct Config {
    pub admin: Pubkey,
    pub bump: u8,
    pub initialized: bool,
}

impl Config {
    pub const SIZE: usize = 32 + 1 + 1;
}

#[error_code]
pub enum DexError {
    #[msg("Config already initialized")]
    AlreadyInitialized,
    #[msg("Missing PDA bump")]
    MissingBump,
    #[msg("Unauthorized")]
    Unauthorized,
}
