use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use crate::{ReleaseCollateral, ESCROW_SEED};
use crate::instructions::deposit_collateral::AddLiquiditySingleSidePreciseParameter;
use crate::instructions::remove_collateral::BinLiquidityReduction;

pub const REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108];
pub const ADD_LIQUIDITY_ONE_SIDE_PRECISE_DISCRIMINATOR: [u8; 8] =
    [161, 194, 103, 84, 171, 71, 250, 154];
pub const CLOSE_POSITION_IF_EMPTY_DISCRIMINATOR: [u8; 8] =
    [59, 124, 212, 118, 91, 152, 110, 157];

pub fn handler(
    ctx: Context<ReleaseCollateral>,
    liquidity_parameter: AddLiquiditySingleSidePreciseParameter,
) -> Result<()> {
    let borrower_key = ctx.accounts.borrower.key();
    let escrow_pda_bump = ctx.bumps.escrow_pda;
    let signer_seeds: &[&[&[u8]]] = &[&[
        ESCROW_SEED,
        b"pda",
        borrower_key.as_ref(),
        &[escrow_pda_bump],
    ]];

    let reductions: Vec<BinLiquidityReduction> = liquidity_parameter
        .bins
        .iter()
        .map(|b| BinLiquidityReduction { bin_id: b.bin_id, bps_to_remove: 10000 })
        .collect();

    let remove_accounts = vec![
        AccountMeta::new(ctx.accounts.escrow_position.key(), false),
        AccountMeta::new(ctx.accounts.lb_pair.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_bitmap_ext.key(), false),
        AccountMeta::new(ctx.accounts.escrow_token_x.key(), false),
        AccountMeta::new(ctx.accounts.escrow_token_y.key(), false),
        AccountMeta::new(ctx.accounts.reserve_x.key(), false),
        AccountMeta::new(ctx.accounts.reserve_y.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_x_mint.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_y_mint.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_lower.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_upper.key(), false),
        AccountMeta::new_readonly(ctx.accounts.escrow_pda.key(), true),
        AccountMeta::new_readonly(ctx.accounts.token_x_program.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_y_program.key(), false),
        AccountMeta::new_readonly(ctx.accounts.event_authority.key(), false),
        AccountMeta::new_readonly(ctx.accounts.lb_clmm_program.key(), false),
    ];
    let mut remove_data = REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
    reductions.serialize(&mut remove_data)?;
    let remove_ix = Instruction {
        program_id: ctx.accounts.lb_clmm_program.key(),
        accounts: remove_accounts,
        data: remove_data,
    };
    invoke_signed(
        &remove_ix,
        &[
            ctx.accounts.escrow_position.to_account_info(),
            ctx.accounts.lb_pair.to_account_info(),
            ctx.accounts.bin_array_bitmap_ext.to_account_info(),
            ctx.accounts.escrow_token_x.to_account_info(),
            ctx.accounts.escrow_token_y.to_account_info(),
            ctx.accounts.reserve_x.to_account_info(),
            ctx.accounts.reserve_y.to_account_info(),
            ctx.accounts.token_x_mint.to_account_info(),
            ctx.accounts.token_y_mint.to_account_info(),
            ctx.accounts.bin_array_lower.to_account_info(),
            ctx.accounts.bin_array_upper.to_account_info(),
            ctx.accounts.escrow_pda.to_account_info(),
            ctx.accounts.token_x_program.to_account_info(),
            ctx.accounts.token_y_program.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.lb_clmm_program.to_account_info(),
        ],
        signer_seeds,
    )?;
    msg!("escrow remove_liquidity (100% drain) succeeded");

    let multiplier = liquidity_parameter.decompress_multiplier;
    let mut total_y: u64 = 0;
    for b in liquidity_parameter.bins.iter() {
        let bin_amount = (b.amount as u64)
            .checked_mul(multiplier)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        total_y = total_y
            .checked_add(bin_amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }
    msg!("Transferring exact {} lamports wSOL escrow ATA -> borrower ATA", total_y);
    let transfer_ix = spl_token_transfer_instruction(
        &ctx.accounts.token_y_program.key(),
        &ctx.accounts.escrow_token_y.key(),
        &ctx.accounts.user_token_y.key(),
        &ctx.accounts.escrow_pda.key(),
        total_y,
    );
    invoke_signed(
        &transfer_ix,
        &[
            ctx.accounts.escrow_token_y.to_account_info(),
            ctx.accounts.user_token_y.to_account_info(),
            ctx.accounts.escrow_pda.to_account_info(),
            ctx.accounts.token_y_program.to_account_info(),
        ],
        signer_seeds,
    )?;
    msg!("wSOL transfer to borrower ATA succeeded");

    let add_accounts = vec![
        AccountMeta::new(ctx.accounts.borrower_position.key(), false),
        AccountMeta::new(ctx.accounts.lb_pair.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_bitmap_ext.key(), false),
        AccountMeta::new(ctx.accounts.user_token_y.key(), false),
        AccountMeta::new(ctx.accounts.reserve_y.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_y_mint.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_lower.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_upper.key(), false),
        AccountMeta::new_readonly(ctx.accounts.borrower.key(), true),
        AccountMeta::new_readonly(ctx.accounts.token_y_program.key(), false),
        AccountMeta::new_readonly(ctx.accounts.event_authority.key(), false),
        AccountMeta::new_readonly(ctx.accounts.lb_clmm_program.key(), false),
    ];
    let mut add_data = ADD_LIQUIDITY_ONE_SIDE_PRECISE_DISCRIMINATOR.to_vec();
    liquidity_parameter.serialize(&mut add_data)?;
    let add_ix = Instruction {
        program_id: ctx.accounts.lb_clmm_program.key(),
        accounts: add_accounts,
        data: add_data,
    };
    invoke(
        &add_ix,
        &[
            ctx.accounts.borrower_position.to_account_info(),
            ctx.accounts.lb_pair.to_account_info(),
            ctx.accounts.bin_array_bitmap_ext.to_account_info(),
            ctx.accounts.user_token_y.to_account_info(),
            ctx.accounts.reserve_y.to_account_info(),
            ctx.accounts.token_y_mint.to_account_info(),
            ctx.accounts.bin_array_lower.to_account_info(),
            ctx.accounts.bin_array_upper.to_account_info(),
            ctx.accounts.borrower.to_account_info(),
            ctx.accounts.token_y_program.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.lb_clmm_program.to_account_info(),
        ],
    )?;
    msg!("add_liquidity_one_side_precise into borrower position succeeded");

    let close_accounts = vec![
        AccountMeta::new(ctx.accounts.escrow_position.key(), false),
        AccountMeta::new_readonly(ctx.accounts.escrow_pda.key(), true),
        AccountMeta::new(ctx.accounts.borrower.key(), false),
        AccountMeta::new_readonly(ctx.accounts.event_authority.key(), false),
        AccountMeta::new_readonly(ctx.accounts.lb_clmm_program.key(), false),
    ];
    let close_ix = Instruction {
        program_id: ctx.accounts.lb_clmm_program.key(),
        accounts: close_accounts,
        data: CLOSE_POSITION_IF_EMPTY_DISCRIMINATOR.to_vec(),
    };
    invoke_signed(
        &close_ix,
        &[
            ctx.accounts.escrow_position.to_account_info(),
            ctx.accounts.escrow_pda.to_account_info(),
            ctx.accounts.borrower.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.lb_clmm_program.to_account_info(),
        ],
        signer_seeds,
    )?;
    msg!("close_position_if_empty on escrow position succeeded");

    ctx.accounts.escrow_state.position = Pubkey::default();
    Ok(())
}

fn spl_token_transfer_instruction(
    token_program: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*source, false),
        AccountMeta::new(*destination, false),
        AccountMeta::new_readonly(*authority, true),
    ];
    let mut data = vec![3u8];
    data.extend_from_slice(&amount.to_le_bytes());
    Instruction { program_id: *token_program, accounts, data }
}