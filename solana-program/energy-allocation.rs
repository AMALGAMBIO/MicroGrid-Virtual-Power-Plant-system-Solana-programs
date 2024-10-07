use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("EnrgyA11ocXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX");

#[program]
pub mod energy_allocation {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, capacity: u64) -> Result<()> {
        let battery = &mut ctx.accounts.battery;
        battery.capacity = capacity;
        battery.available = capacity;
        battery.authority = ctx.accounts.authority.key();
        Ok(())
    }

    pub fn allocate(ctx: Context<Allocate>, amount: u64) -> Result<()> {
        let battery = &mut ctx.accounts.battery;
        let user_account = &mut ctx.accounts.user_account;

        require!(battery.available >= amount, ErrorCode::InsufficientCapacity);

        battery.available -= amount;
        user_account.allocated += amount;
        Ok(())
    }

    pub fn deallocate(ctx: Context<Deallocate>, amount: u64) -> Result<()> {
        let battery = &mut ctx.accounts.battery;
        let user_account = &mut ctx.accounts.user_account;

        require!(user_account.allocated >= amount, ErrorCode::InsufficientAllocation);

        battery.available += amount;
        user_account.allocated -= amount;
        Ok(())
    }

    pub fn deposit_energy(ctx: Context<DepositEnergy>, amount: u64) -> Result<()> {
        let battery = &mut ctx.accounts.battery;
        let user_account = &mut ctx.accounts.user_account;

        require!(battery.available + amount <= battery.capacity, ErrorCode::BatteryFull);

        battery.available += amount;
        user_account.energy_balance += amount;

        // Transfer energy tokens from user to battery
        let cpi_accounts = Transfer {
            from: ctx.accounts.user_token_account.to_account_info(),
            to: ctx.accounts.battery_token_account.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }

    pub fn withdraw_energy(ctx: Context<WithdrawEnergy>, amount: u64) -> Result<()> {
        let battery = &mut ctx.accounts.battery;
        let user_account = &mut ctx.accounts.user_account;

        require!(user_account.energy_balance >= amount, ErrorCode::InsufficientBalance);
        require!(battery.available >= amount, ErrorCode::InsufficientEnergy);

        battery.available -= amount;
        user_account.energy_balance -= amount;

        // Transfer energy tokens from battery to user
        let cpi_accounts = Transfer {
            from: ctx.accounts.battery_token_account.to_account_info(),
            to: ctx.accounts.user_token_account.to_account_info(),
            authority: ctx.accounts.battery_authority.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, payer = authority, space = 8 + 8 + 8 + 32)]
    pub battery: Account<'info, Battery>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Allocate<'info> {
    #[account(mut)]
    pub battery: Account<'info, Battery>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    pub user_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Deallocate<'info> {
    #[account(mut)]
    pub battery: Account<'info, Battery>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    pub user_authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DepositEnergy<'info> {
    #[account(mut)]
    pub battery: Account<'info, Battery>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub battery_token_account: Account<'info, TokenAccount>,
    pub user_authority: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct WithdrawEnergy<'info> {
    #[account(mut)]
    pub battery: Account<'info, Battery>,
    #[account(mut)]
    pub user_account: Account<'info, UserAccount>,
    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub battery_token_account: Account<'info, TokenAccount>,
    pub user_authority: Signer<'info>,
    /// CHECK: This is the battery authority, which should be a PDA
    pub battery_authority: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

#[account]
pub struct Battery {
    pub capacity: u64,
    pub available: u64,
    pub authority: Pubkey,
}

#[account]
pub struct UserAccount {
    pub allocated: u64,
    pub energy_balance: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Insufficient capacity in the battery")]
    InsufficientCapacity,
    #[msg("Insufficient allocation for the user")]
    InsufficientAllocation,
    #[msg("Battery is full")]
    BatteryFull,
    #[msg("Insufficient energy balance")]
    InsufficientBalance,
    #[msg("Insufficient energy in the battery")]
    InsufficientEnergy,
}

