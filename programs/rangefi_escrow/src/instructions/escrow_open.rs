use anchor_lang::prelude::*;
use anchor_lang::prelude::rent;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};
use crate::constants::*;
use crate::state::EscrowState;

pub const LB_CLMM_PROGRAM: Pubkey = pubkey!("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
pub const INITIALIZE_POSITION_PDA_DISC: [u8; 8] = [46, 82, 125, 146, 85, 141, 228, 153];

pub fn handler(ctx: Context<crate::EscrowOpen>, lower_bin_id: i32, width: i32) -> Result<()> {
    let borrower_key = ctx.accounts.borrower.key();
    let escrow_pda_bump = ctx.bumps.escrow_pda;

    let escrow_signer_seeds: &[&[&[u8]]] = &[&[
        ESCROW_SEED,
        b"pda",
        borrower_key.as_ref(),
        &[escrow_pda_bump],
    ]];

    let mut data = INITIALIZE_POSITION_PDA_DISC.to_vec();
    data.extend_from_slice(&lower_bin_id.to_le_bytes());
    data.extend_from_slice(&width.to_le_bytes());

    let (event_authority, _) = Pubkey::find_program_address(
        &[b"__event_authority"],
        &LB_CLMM_PROGRAM,
    );

    let accounts = vec![
        AccountMeta::new(ctx.accounts.borrower.key(), true),
        AccountMeta::new_readonly(ctx.accounts.escrow_pda.key(), true),
        AccountMeta::new(ctx.accounts.position.key(), false),
        AccountMeta::new_readonly(ctx.accounts.lb_pair.key(), false),
        AccountMeta::new_readonly(ctx.accounts.escrow_pda.key(), false),
        AccountMeta::new_readonly(anchor_lang::solana_program::system_program::ID, false),
        AccountMeta::new_readonly(rent::ID, false),
        AccountMeta::new_readonly(event_authority, false),
        AccountMeta::new_readonly(LB_CLMM_PROGRAM, false),
    ];

    let ix = Instruction {
        program_id: LB_CLMM_PROGRAM,
        accounts,
        data,
    };

    invoke_signed(
        &ix,
        &[
            ctx.accounts.borrower.to_account_info(),
            ctx.accounts.escrow_pda.to_account_info(),
            ctx.accounts.position.to_account_info(),
            ctx.accounts.lb_pair.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.lb_clmm_program.to_account_info(),
        ],
        escrow_signer_seeds,
    )?;

    let escrow_state = &mut ctx.accounts.escrow_state;
    escrow_state.position = ctx.accounts.position.key();
    escrow_state.lb_pair = ctx.accounts.lb_pair.key();

    msg!("Position opened: {:?}", ctx.accounts.position.key());
    msg!("Owner (escrow PDA): {:?}", ctx.accounts.escrow_pda.key());

    Ok(())
}