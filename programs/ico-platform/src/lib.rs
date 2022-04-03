use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Burn, Mint, MintTo, TokenAccount, Transfer};
// use std::str::FromStr;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod ico_platform {
    use super::*;
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
    #[msg("ICO times are non-sequential")]
    SeqTimes,
    #[msg("Invalid param")]
    InvalidParam,
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
