use anchor_lang::prelude::*;
use crate::constants::*;
use crate::state::EscrowState;

/// Proof of concept: derive an escrow PDA and record it on-chain.
/// Next step: this PDA will CPI into Meteora to open a position.
pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_state;
    escrow.borrower = ctx.accounts.borrower.key();
    escrow.position = Pubkey::default(); // will be set after Meteora CPI
    escrow.lb_pair = Pubkey::default();  // will be set after Meteora CPI
    escrow.bump = ctx.bumps.escrow_pda;

    msg!(
        "Escrow PDA created: {:?}",
        ctx.accounts.escrow_pda.key()
    );
    msg!(
        "Borrower: {:?}",
        ctx.accounts.borrower.key()
    );

    Ok(())
}

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
        seeds = [ESCROW_SEED, borrower.key().as_ref()],
        bump
    )]
    pub escrow_pda: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}