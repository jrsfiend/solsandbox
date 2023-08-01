use anchor_lang::prelude::*;
use anchor_spl::{token::{self, Token, Mint, TokenAccount}, associated_token::{AssociatedToken}};
use anchor_lang::solana_program::{system_program, sysvar};
use whirlpools::{self, state::*};
use crate::state::Authority;
use { 
  clockwork_sdk::{
      state::{Thread, ThreadAccount, ThreadResponse},
  }
};
#[derive(Accounts)]
pub struct ProxyIncreaseLiquidity<'info> {

  #[account(mut, address = hydra.pubkey(), signer)]
  pub hydra: Account<'info, Thread>,
  pub whirlpool_program: Program<'info, whirlpools::program::Whirlpool>,

  #[account(mut)]
  pub whirlpool: Box<Account<'info, Whirlpool>>,

  #[account(address = token::ID)]
  pub token_program: Program<'info, Token>,

  

  #[account(mut, has_one = whirlpool)]
  pub position: Box<Account<'info, Position>>,
  #[account(
      constraint = position_token_account.mint == position.position_mint,
      constraint = position_token_account.amount == 1
  )]
  pub position_token_account: Box<Account<'info, TokenAccount>>,

  #[account(mut)]
  pub token_owner_account_a: Box<Account<'info, TokenAccount>>,
  #[account(mut)]
  pub token_owner_account_b: Box<Account<'info, TokenAccount>>,
  pub mint_a: Box<Account<'info, Mint>>,
  pub mint_b: Box<Account<'info, Mint>>,
  

  #[account(mut, constraint = token_vault_a.key() == whirlpool.token_vault_a)]
  pub token_vault_a: Box<Account<'info, TokenAccount>>,
  #[account(mut, constraint = token_vault_b.key() == whirlpool.token_vault_b)]
  pub token_vault_b: Box<Account<'info, TokenAccount>>,

  #[account(mut)]
  pub tick_array_lower: UncheckedAccount<'info>,
  #[account(mut)]
  pub tick_array_upper: UncheckedAccount<'info>,

  /// CHECK: safe
  #[account(seeds = [b"authority", position.key().as_ref()], bump)]
  pub authority: Box<Account<'info, Authority>>,
  /// The Solana system program.
  #[account(address = system_program::ID)]
  pub system_program: Program<'info, System>,
  #[account(address = sysvar::rent::ID)]
  pub rent: Sysvar<'info, Rent>,
  #[account(address = anchor_spl::associated_token::ID)]
  pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn handler(
  ctx: Context<ProxyIncreaseLiquidity>,
  liquidity_amount: u128,
  token_max_a: u64,
  token_max_b: u64,
  bump: u8
) -> Result<ThreadResponse> {
  let whirlpool = &ctx.accounts.whirlpool;
  let position = &ctx.accounts.position;

  let tick_lower_index = &whirlpool.tick_current_index
      - &whirlpool.tick_current_index % whirlpool.tick_spacing as i32
      - whirlpool.tick_spacing as i32 * 1;
  let tick_upper_index = &whirlpool.tick_current_index
      - &whirlpool.tick_current_index % whirlpool.tick_spacing as i32
      + whirlpool.tick_spacing as i32 * 1;
  let tlip = position.tick_lower_index;
  let tuip = position.tick_upper_index;
  // on start we init, hab a mint. we hab other mints lined up.
  let cpi_program = ctx.accounts.whirlpool_program.to_account_info();

  let cpi_accounts = whirlpools::cpi::accounts::IncreaseLiquidity {
    whirlpool: ctx.accounts.whirlpool.to_account_info(),
    token_program: ctx.accounts.token_program.to_account_info(),
    position_authority: ctx.accounts.authority.to_account_info(),
    position: ctx.accounts.position.clone().to_account_info(),
    position_token_account: ctx.accounts.position_token_account.to_account_info(),
    token_owner_account_a: ctx.accounts.token_owner_account_a.to_account_info(),
    token_owner_account_b: ctx.accounts.token_owner_account_b.to_account_info(),
    token_vault_a: ctx.accounts.token_vault_a.to_account_info(),
    token_vault_b: ctx.accounts.token_vault_b.to_account_info(),
    tick_array_lower: ctx.accounts.tick_array_lower.to_account_info(),
    tick_array_upper: ctx.accounts.tick_array_upper.to_account_info(),
  };

  
  let key_ref = ctx.accounts.position.key();
  let authority_seeds = [b"authority",key_ref.as_ref(), &[bump]];
  
  let signer_seeds = [authority_seeds.as_ref()];
  let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);

  // execute CPI
  msg!("CPI: whirlpool increase_liquidity instruction");
  whirlpools::cpi::increase_liquidity(
    cpi_ctx,
    liquidity_amount,
    token_max_a,
    token_max_b,
  )?;

  Ok(ThreadResponse {
       next_instruction: None,
       kickoff_instruction: None,
   })
}