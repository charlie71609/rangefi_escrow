pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use instructions::escrow_open::LB_CLMM_PROGRAM;
pub use constants::*;
pub use state::*;

declare_id!("9xbgm1HSMM8aUn4SAzrh1bfMm4of1GtYzsebnouogmby");

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
        has_one = borrower,
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

#[program]
pub mod rangefi_escrow {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    pub fn escrow_open(ctx: Context<EscrowOpen>, lower_bin_id: i32, width: i32) -> Result<()> {
        instructions::escrow_open::handler(ctx, lower_bin_id, width)
    }
}