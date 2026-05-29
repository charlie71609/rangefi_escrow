use anchor_lang::prelude::*;
use crate::state::EscrowState;
use crate::constants::*;

pub fn handler(ctx: Context<crate::Initialize>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_state;
    escrow.borrower = ctx.accounts.borrower.key();
    escrow.position = Pubkey::default();
    escrow.lb_pair = Pubkey::default();
    escrow.bump = ctx.bumps.escrow_pda;
    msg!("Escrow PDA: {:?}", ctx.accounts.escrow_pda.key());
    Ok(())
}