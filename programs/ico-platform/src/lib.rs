use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};
// use std::str::FromStr;

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
        pool_account.native_mint = *ctx.accounts.pool_native.to_account_info().key;
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
}

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(init, payer = payer, space = PoolAccount::LEN)]
    pub pool_account: Box<Account<'info, PoolAccount>>,
    #[account(mut)]
    pub pool_signer: AccountInfo<'info>,
    #[account(constraint = redeemable_mint.mint_authority == COption::Some(*pool_signer.key), constraint = redeemable_mint.supply == 0)]
    pub redeemable_mint: Box<Account<'info, Mint>>,
    #[account(constraint = usdc_mint.decimals == redeemable_mint.decimals)]
    pub usdc_mint: Box<Account<'info, Mint>>,
    #[account(constraint = pool_native.mint == *native_mint.to_account_info().key)]
    pub native_mint: Box<Account<'info, TokenAccount>>,
    #[account(constraint = pool_native.owner == *pool_signer.key)]
    pool_native: Box<Account<'info, TokenAccount>>,
    #[account(constraint = pool_usdc.owner == *pool_signer.key)]
    pub pool_usdc: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub distribution_authority: Signer<'info>,
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
    #[msg("IDO must start in the future")]
    IdoFuture,
    #[msg("ICO times are non-sequential")]
    SeqTimes,
    #[msg("Given nonce is invalid")]
    InvalidNonce,
    #[msg("Invalid param")]
    InvalidParam,
}


// Access Control Modifiers

// ICO Starts in the Future
fn future_start_time<'info>(ctx: &Context<InitializePool<'info>>, start_ico_ts: i64) -> Result<()> {
    if !(ctx.accounts.clock.unix_timestamp < start_ico_ts) {
        return Err(ErrorCode::IdoFuture.into());
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
