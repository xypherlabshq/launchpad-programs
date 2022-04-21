use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};

declare_id!("4UJV9VxwoewYhw1qZKtPoVAdhd4tW8AaaDzpawuN9YuA");

#[program]
pub mod ico_platform {
    use super::*;

    #[access_control(InitializePool::accounts(&ctx, nonce) future_start_time(&ctx, start_ico_ts))]
    pub fn initialize_pool(ctx: Context<InitializePool>, num_ico_tokens: u64, nonce: u8, start_ico_ts: i64, end_ico_ts: i64, withdraw_native_ts: i64) -> Result<()> {

        if !(start_ico_ts < end_ico_ts
            && end_ico_ts <= withdraw_native_ts)
        {
            return Err(ErrorCode::SeqTimes.into());
        }
        if num_ico_tokens == 0 {
            return Err(ErrorCode::InvalidParam.into());
        }

        let pool_account = &mut ctx.accounts.pool_account;
        
        pool_account.redeemable_mint = *ctx.accounts.redeemable_mint.to_account_info().key;
        pool_account.pool_native = *ctx.accounts.pool_native.to_account_info().key;
        pool_account.native_mint = ctx.accounts.pool_native.mint;
        pool_account.pool_usdc = *ctx.accounts.pool_usdc.to_account_info().key;
        pool_account.distribution_authority = *ctx.accounts.distribution_authority.key;
        pool_account.nonce = nonce;
        pool_account.num_ico_tokens = num_ico_tokens;
        pool_account.start_ico_ts = start_ico_ts;
        pool_account.end_ico_ts = end_ico_ts;
        pool_account.withdraw_native_ts = withdraw_native_ts;

        //Transfer Native tokens from Creator to Pool Account
        let cpi_accounts = Transfer {
            from: ctx.accounts.creator_native.to_account_info(),
            to: ctx.accounts.pool_native.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, num_ico_tokens)?;

        Ok(())
    }

    pub fn modify_ico_time(
        ctx: Context<ModifyIcoTime>,
        start_ico_ts: i64,
        end_ico_ts: i64,
        withdraw_native_ts: i64,
    ) -> Result<()> {
        if !(start_ico_ts < end_ico_ts
            && end_ico_ts < withdraw_native_ts)
        {
            return Err(ErrorCode::SeqTimes.into());
        }
    
        let pool_account = &mut ctx.accounts.pool_account;
        pool_account.start_ico_ts = start_ico_ts;
        pool_account.end_ico_ts = end_ico_ts;
        pool_account.withdraw_native_ts = withdraw_native_ts;
        
        Ok(())
    }

    #[access_control(unrestricted_phase(&ctx))]
    pub fn exchange_usdc_for_redeemable(
        ctx: Context<ExchangeUsdcForRedeemable>,
        amount: u64,
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidParam.into());
        }
        
        if ctx.accounts.user_usdc.amount < amount {
            return Err(ErrorCode::LowUsdc.into());
        }

        let cpi_accounts = Transfer {
            from: ctx.accounts.user_usdc.to_account_info(),
            to: ctx.accounts.pool_usdc.to_account_info(),
            authority: ctx.accounts.user_authority.clone(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        let seeds = &[
            ctx.accounts.pool_account.native_mint.as_ref(),
            &[ctx.accounts.pool_account.nonce],
        ];

        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            to: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.pool_signer.clone(),
        };

        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, amount)?;

        Ok(())
    }

    #[access_control(ico_over(&ctx.accounts.pool_account, &ctx.accounts.clock))]
    pub fn exchange_redeemable_for_native(
        ctx: Context<ExchangeRedeemableForNative>,
        amount: u64,
    ) -> Result<()> {
        if amount == 0 {
            return Err(ErrorCode::InvalidParam.into());
        }

        if ctx.accounts.user_redeemable.amount < amount {
            return Err(ErrorCode::LowRedeemable.into())
        }

        // let real_pool_supply = ctx.accounts.pool_native.amount;
        // let real_redeemable_supply = ctx.accounts.redeemable_mint.supply * u64::pow(10, 3);

        // let token_price: f64 =
        //     (real_redeemable_supply as f64 / real_pool_supply as f64) * f64::powf(10.0, 9.0);

        let native_amount = (amount as u128)
            .checked_mul(ctx.accounts.pool_native.amount as u128)
            .unwrap()
            .checked_div(ctx.accounts.redeemable_mint.supply as u128)
            .unwrap();
            
        let cpi_accounts = Burn {
            mint: ctx.accounts.redeemable_mint.to_account_info(),
            from: ctx.accounts.user_redeemable.to_account_info(),
            authority: ctx.accounts.user_authority.to_account_info(),
        };

        let cpi_program = ctx.accounts.token_program.clone();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::burn(cpi_ctx, amount)?;

        let seeds = &[
            ctx.accounts.pool_account.native_mint.as_ref(),
            &[ctx.accounts.pool_account.nonce],
        ];

        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.pool_native.to_account_info(),
            to: ctx.accounts.user_native.to_account_info(),
            authority: ctx.accounts.pool_signer.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.clone();

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, native_amount as u64)?;

        Ok(())
    }

    #[access_control(ico_over(&ctx.accounts.pool_account, &ctx.accounts.clock))]
        pub fn withdraw_pool_usdc(ctx: Context<WithdrawPoolUsdc>, amount: u64) -> Result<()> {            
            let seeds = &[
                ctx.accounts.pool_account.native_mint.as_ref(),
                &[ctx.accounts.pool_account.nonce],
            ];
            let signer = &[&seeds[..]];
            let cpi_accounts = Transfer {
                from: ctx.accounts.pool_usdc.to_account_info(),
                to: ctx.accounts.creator_usdc.to_account_info(),
                authority: ctx.accounts.pool_signer.to_account_info(),
            };
            let cpi_program = ctx.accounts.token_program.clone();
            let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
            token::transfer(cpi_ctx, amount)?;
    
            Ok(())
        }
}



#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = payer, space = PoolAccount::LEN)]
    pub pool_account: Box<Account<'info, PoolAccount>>,
    #[account(mut)]
    pub pool_signer: AccountInfo<'info>,
    #[account(
        constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key), 
        constraint = redeemable_mint.supply == 0
    )]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(constraint = usdc_mint.decimals == redeemable_mint.decimals)]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(constraint = pool_native.mint == *native_mint.to_account_info().key)]
    pub native_mint: Box<Account<'info, Mint>>,
    #[account(mut, constraint = pool_native.owner == *pool_signer.key)]
    pub pool_native: Box<Account<'info, TokenAccount>>,
    #[account(constraint = pool_usdc.owner == *pool_signer.key)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub distribution_authority: AccountInfo<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub creator_native: Box<Account<'info, TokenAccount>>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

impl<'info> InitializePool<'info> {
    fn accounts(ctx: &Context<InitializePool<'info>>, nonce: u8) -> Result<()> {
        let expected_signer = Pubkey::create_program_address(
            &[ctx.accounts.pool_native.mint.as_ref(), &[nonce]],
            ctx.program_id,
        )
        .map_err(|_| ErrorCode::InvalidNonce)?;
        if ctx.accounts.pool_signer.key != &expected_signer {
            return Err(ErrorCode::InvalidNonce.into());
        }
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ModifyIcoTime<'info> {
    #[account(mut, has_one = distribution_authority)]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(signer)]
    pub distribution_authority: AccountInfo<'info>,
    #[account(signer)]
    pub payer: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ExchangeUsdcForRedeemable<'info> {
    #[account(has_one = redeemable_mint, has_one = pool_usdc)]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(seeds = [pool_account.native_mint.as_ref()], bump = pool_account.nonce)]
    pool_signer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key)
    )]
    pub redeemable_mint: Account<'info, Mint>,
    #[account(mut, constraint = pool_usdc.owner == *pool_signer.key)]
    pub pool_usdc: Account<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut, constraint = user_usdc.owner == *user_authority.key)]
    pub user_usdc: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_redeemable.owner == *user_authority.key)]
    pub user_redeemable: Account<'info, TokenAccount>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct ExchangeRedeemableForNative<'info> {
    #[account(has_one = redeemable_mint, has_one = pool_native)]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(seeds = [pool_account.native_mint.as_ref()], bump = pool_account.nonce)]
    pool_signer: AccountInfo<'info>,
    #[account(
        mut,
        constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key)
    )]
    pub redeemable_mint: Account<'info, Mint>,
    #[account(mut, constraint = pool_native.owner == *pool_signer.key)]
    pub pool_native: Account<'info, TokenAccount>,
    #[account(signer)]
    pub user_authority: AccountInfo<'info>,
    #[account(mut, constraint = user_native.owner == *user_authority.key)]
    pub user_native: Account<'info, TokenAccount>,
    #[account(mut, constraint = user_redeemable.owner == *user_authority.key)]
    pub user_redeemable: Account<'info, TokenAccount>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
pub struct WithdrawPoolUsdc<'info> {
    #[account(has_one = pool_usdc, has_one = distribution_authority)]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(seeds = [pool_account.native_mint.as_ref()], bump = pool_account.nonce)]
    pub pool_signer: AccountInfo<'info>,
    #[account(mut, constraint = pool_usdc.owner == *pool_signer.key)]
    pub pool_usdc: Account<'info, TokenAccount>,
    #[account(signer)]
    pub distribution_authority: AccountInfo<'info>,
    #[account(signer)]
    pub payer: AccountInfo<'info>,
    #[account(mut)]
    pub creator_usdc: Account<'info, TokenAccount>,
    #[account(constraint = token_program.key == &token::ID)]
    pub token_program: AccountInfo<'info>,
    pub clock: Sysvar<'info, Clock>,
}

#[account]
pub struct PoolAccount {
    pub redeemable_mint: Pubkey,
    pub pool_native: Pubkey,
    pub native_mint: Pubkey,
    pub pool_usdc: Pubkey,
    pub distribution_authority: Pubkey,
    pub nonce: u8,
    pub num_ico_tokens: u64,
    pub start_ico_ts: i64,
    pub end_ico_ts: i64,
    pub withdraw_native_ts: i64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("ICO must start in the future")]
    IcoFuture,
    #[msg("ICO times are non-sequential")]
    SeqTimes,
    #[msg("ICO has not started")]
    StartIcoTime,
    #[msg("ICO has ended")]
    EndIcoTime,
    #[msg("ICO has not finished yet")]
    IcoNotOver,
    #[msg("Insufficient USDC")]
    LowUsdc,
    #[msg("Insufficient redeemable tokens")]
    LowRedeemable,
    #[msg("USDC total and redeemable total don't match")]
    UsdcNotEqRedeem,
    #[msg("Given nonce is invalid")]
    InvalidNonce,
    #[msg("Invalid param")]
    InvalidParam,
    #[msg("Cannot withdraw USDC after depositing")]
    UsdcWithdrawNotAllowed,
    #[msg("Tokens still need to be redeemed")]
    WithdrawTokensNotAllowed,
}


// Access Control Modifiers

// ICO Starts in the Future
fn future_start_time<'info>(ctx: &Context<InitializePool<'info>>, start_ico_ts: i64) -> Result<()> {
    if !(ctx.accounts.clock.unix_timestamp < start_ico_ts) {
        return Err(ErrorCode::IcoFuture.into());
    }
    Ok(())
}

// Unrestricted Phase (Before ICO)
fn unrestricted_phase<'info>(ctx: &Context<ExchangeUsdcForRedeemable<'info>>) -> Result<()> {
    if !(ctx.accounts.pool_account.start_ico_ts < ctx.accounts.clock.unix_timestamp) {
        return Err(ErrorCode::StartIcoTime.into());
    }
    Ok(())
}

//ICO Over
fn ico_over<'info>(
    pool_account: &Account<'info, PoolAccount>,
    clock: &Sysvar<'info, Clock>,
) -> Result<()> {
    if !(pool_account.withdraw_native_ts < clock.unix_timestamp) {
        return Err(ErrorCode::IcoNotOver.into());
    }
    Ok(())
}


const DISCRIMATOR_LENGTH: usize = 8;
const PUBLIC_KEY_LENGTH: usize = 32;
const NONCE_LENGTH: usize = 1;
const NUM_LENGTH: usize = 8;
const TIMESTAMP_LENGTH: usize = 8;

impl PoolAccount {
    const LEN: usize = DISCRIMATOR_LENGTH
        + PUBLIC_KEY_LENGTH //Redeemable Mint
        + PUBLIC_KEY_LENGTH //Pool Native
        + PUBLIC_KEY_LENGTH //Native Mint
        + PUBLIC_KEY_LENGTH //Pool USDC
        + PUBLIC_KEY_LENGTH //Distribution Authority
        + NONCE_LENGTH //Nonce
        + NUM_LENGTH //Number of ICO Tokens
        + TIMESTAMP_LENGTH //ICO Start Timestamp
        + TIMESTAMP_LENGTH //ICO End Timestamp
        + TIMESTAMP_LENGTH; //Withdraw Native Token Timestamp
}
