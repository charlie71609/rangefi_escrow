pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use instructions::escrow_open::LB_CLMM_PROGRAM;
use instructions::deposit_collateral::{BinLiquidityReduction, AddLiquiditySingleSidePreciseParameter};
pub use constants::*;
pub use state::*;

declare_id!("4bVzaKedFWhFrJLkg46a7d77vcUj331HPpW8KLQY1HdS");

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    #[account(
        init,
        payer = borrower,
        space = EscrowState::LEN,
        seeds = [ESCROW_SEED, borrower.key().as_ref()],
        bump
    )]
    pub escrow_state: Account<'info, EscrowState>,

    /// CHECK: This is the escrow PDA that will own Meteora positions
    #[account(
        seeds = [ESCROW_SEED, b"pda", borrower.key().as_ref()],
        bump
    )]
    pub escrow_pda: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(lower_bin_id: i32, width: i32)]
pub struct EscrowOpen<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// CHECK: PDA derived from escrow seeds
    #[account(
        seeds = [ESCROW_SEED, b"pda", borrower.key().as_ref()],
        bump
    )]
    pub escrow_pda: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, borrower.key().as_ref()],
        bump,
        constraint = escrow_state.borrower == borrower.key() @ crate::error::EscrowError::Unauthorized,
    )]
    pub escrow_state: Account<'info, EscrowState>,

    /// CHECK: Created by Meteora CPI
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: Validated by Meteora program
    pub lb_pair: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

   /// CHECK: Sysvar
   pub rent: Sysvar<'info, Rent>,

   /// CHECK: Meteora event authority PDA
   pub event_authority: UncheckedAccount<'info>,

  /// CHECK: Meteora DLMM program
  #[account(address = LB_CLMM_PROGRAM)]
  pub lb_clmm_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct DepositCollateral<'info> {
   #[account(mut)]
   pub borrower: Signer<'info>,

   /// CHECK: Escrow PDA — signs remove_liquidity and add_liquidity via invoke_signed
   #[account(
       seeds = [ESCROW_SEED, b"pda", borrower.key().as_ref()],
       bump
   )]
   pub escrow_pda: UncheckedAccount<'info>,

   #[account(
    mut,
    seeds = [ESCROW_SEED, borrower.key().as_ref()],
    bump,
    constraint = escrow_state.borrower == borrower.key() @ crate::error::EscrowError::Unauthorized,
)]
pub escrow_state: Account<'info, EscrowState>,

   /// CHECK: The borrower's existing Meteora position — must be owned by borrower
   #[account(mut)]
   pub position: UncheckedAccount<'info>,

   /// CHECK: The new escrow-owned position — created by prior escrow_open call
   #[account(mut)]
   pub escrow_position: UncheckedAccount<'info>,

   /// CHECK: lb_pair for the pool
   #[account(mut)]
   pub lb_pair: UncheckedAccount<'info>,

   /// CHECK: bin_array_bitmap_ext — optional, pass as program ID if unused
   #[account(mut)]
   pub bin_array_bitmap_ext: UncheckedAccount<'info>,

   /// CHECK: Borrower ATA for token_x
   #[account(mut)]
   pub user_token_x: UncheckedAccount<'info>,

   /// CHECK: Borrower ATA for token_y (wSOL)
   #[account(mut)]
   pub user_token_y: UncheckedAccount<'info>,

   /// CHECK: Pool reserve_x
   #[account(mut)]
   pub reserve_x: UncheckedAccount<'info>,

   /// CHECK: Pool reserve_y
   #[account(mut)]
   pub reserve_y: UncheckedAccount<'info>,

   /// CHECK: token_x mint
   pub token_x_mint: UncheckedAccount<'info>,

   /// CHECK: token_y mint
   pub token_y_mint: UncheckedAccount<'info>,

   /// CHECK: bin_array_lower
   #[account(mut)]
   pub bin_array_lower: UncheckedAccount<'info>,

   /// CHECK: bin_array_upper
   #[account(mut)]
   pub bin_array_upper: UncheckedAccount<'info>,

   /// CHECK: token_x program (Token or Token-2022)
   pub token_x_program: UncheckedAccount<'info>,

   /// CHECK: token_y program
   pub token_y_program: UncheckedAccount<'info>,

   /// CHECK: Meteora event authority
   pub event_authority: UncheckedAccount<'info>,

   /// CHECK: Meteora DLMM program
   #[account(address = LB_CLMM_PROGRAM)]
   pub lb_clmm_program: UncheckedAccount<'info>,

   /// CHECK: Escrow PDA's wSOL ATA — receives tokens from remove, sends to add_liquidity
   #[account(mut)]
   pub escrow_token_y: UncheckedAccount<'info>,

   pub system_program: Program<'info, System>,
}
#[derive(Accounts)]
pub struct UpdateLbPair<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
        mut,
        seeds = [ESCROW_SEED, borrower.key().as_ref()],
        bump,
        has_one = borrower,
    )]
    pub escrow_state: Account<'info, EscrowState>,
}

#[derive(Accounts)]
pub struct RemoveCollateral<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// CHECK: Borrower's Meteora position
    #[account(mut)]
    pub position: UncheckedAccount<'info>,

    /// CHECK: lb_pair
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: bin_array_bitmap_ext
    #[account(mut)]
    pub bin_array_bitmap_ext: UncheckedAccount<'info>,

    /// CHECK: Borrower ATA for token_x
    #[account(mut)]
    pub user_token_x: UncheckedAccount<'info>,

    /// CHECK: Borrower ATA for token_y
    #[account(mut)]
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: Pool reserve_x
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: Pool reserve_y
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: token_x mint
    pub token_x_mint: UncheckedAccount<'info>,

    /// CHECK: token_y mint
    pub token_y_mint: UncheckedAccount<'info>,

    /// CHECK: bin_array_lower
    #[account(mut)]
    pub bin_array_lower: UncheckedAccount<'info>,

    /// CHECK: bin_array_upper
    #[account(mut)]
    pub bin_array_upper: UncheckedAccount<'info>,

    /// CHECK: token_x program
    pub token_x_program: UncheckedAccount<'info>,

    /// CHECK: token_y program
    pub token_y_program: UncheckedAccount<'info>,

    /// CHECK: Meteora event authority
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Meteora DLMM program
    #[account(address = LB_CLMM_PROGRAM)]
    pub lb_clmm_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct ReleaseCollateral<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,

    /// CHECK: Escrow PDA — signs remove, transfer, and close via invoke_signed
    #[account(seeds = [ESCROW_SEED, b"pda", borrower.key().as_ref()], bump)]
    pub escrow_pda: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [ESCROW_SEED, borrower.key().as_ref()],
        bump,
        constraint = escrow_state.borrower == borrower.key() @ crate::error::EscrowError::Unauthorized,
    )]
    pub escrow_state: Account<'info, EscrowState>,

    /// CHECK: Escrow-owned position — drained then closed
    #[account(mut)]
    pub escrow_position: UncheckedAccount<'info>,

    /// CHECK: Fresh borrower-owned position (created client-side) — fill target
    #[account(mut)]
    pub borrower_position: UncheckedAccount<'info>,

    /// CHECK: lb_pair
    #[account(mut)]
    pub lb_pair: UncheckedAccount<'info>,

    /// CHECK: bin_array_bitmap_ext
    #[account(mut)]
    pub bin_array_bitmap_ext: UncheckedAccount<'info>,

    /// CHECK: Escrow USDC ATA — X-side remove destination (0 removed, layout-required)
    #[account(mut)]
    pub escrow_token_x: UncheckedAccount<'info>,

    /// CHECK: Escrow wSOL ATA — Y remove destination + transfer source
    #[account(mut)]
    pub escrow_token_y: UncheckedAccount<'info>,

    /// CHECK: Borrower wSOL ATA — transfer destination + add source
    #[account(mut)]
    pub user_token_y: UncheckedAccount<'info>,

    /// CHECK: reserve_x
    #[account(mut)]
    pub reserve_x: UncheckedAccount<'info>,

    /// CHECK: reserve_y
    #[account(mut)]
    pub reserve_y: UncheckedAccount<'info>,

    /// CHECK: token_x mint
    pub token_x_mint: UncheckedAccount<'info>,

    /// CHECK: token_y mint
    pub token_y_mint: UncheckedAccount<'info>,

    /// CHECK: bin_array_lower
    #[account(mut)]
    pub bin_array_lower: UncheckedAccount<'info>,

    /// CHECK: bin_array_upper
    #[account(mut)]
    pub bin_array_upper: UncheckedAccount<'info>,

    /// CHECK: token_x program
    pub token_x_program: UncheckedAccount<'info>,

    /// CHECK: token_y program
    pub token_y_program: UncheckedAccount<'info>,

    /// CHECK: Meteora event authority
    pub event_authority: UncheckedAccount<'info>,

    /// CHECK: Meteora DLMM program
    #[account(address = LB_CLMM_PROGRAM)]
    pub lb_clmm_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[program]
pub mod rangefi_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn escrow_open(ctx: Context<EscrowOpen>, lower_bin_id: i32, width: i32) -> Result<()> {
        instructions::escrow_open::handler(ctx, lower_bin_id, width)
    }

    pub fn deposit_collateral(
        ctx: Context<DepositCollateral>,
        bin_liquidity_reductions: Vec<BinLiquidityReduction>,
        liquidity_parameter: AddLiquiditySingleSidePreciseParameter,
    ) -> Result<()> {
        instructions::deposit_collateral::handler(ctx, bin_liquidity_reductions, liquidity_parameter)
    }

    pub fn update_lb_pair(ctx: Context<UpdateLbPair>, new_lb_pair: Pubkey) -> Result<()> {
        ctx.accounts.escrow_state.lb_pair = new_lb_pair;
        Ok(())
    }

    pub fn remove_collateral(
        ctx: Context<RemoveCollateral>,
        bin_liquidity_reductions: Vec<instructions::remove_collateral::BinLiquidityReduction>,
    ) -> Result<()> {
        instructions::remove_collateral::handler(ctx, bin_liquidity_reductions)
    }

    pub fn release_collateral(
        ctx: Context<ReleaseCollateral>,
        liquidity_parameter: AddLiquiditySingleSidePreciseParameter,
    ) -> Result<()> {
        instructions::release_collateral::handler(ctx, liquidity_parameter)
    }
}