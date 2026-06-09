use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::program::invoke;
use crate::{DepositCollateral, ESCROW_SEED};

// Meteora: add_liquidity_one_side_precise
pub const ADD_LIQUIDITY_ONE_SIDE_PRECISE_DISCRIMINATOR: [u8; 8] =
    [161, 194, 103, 84, 171, 71, 250, 154];

// Kept so lib.rs's instruction signature is unchanged (this arg is unused here;
// liquidity removal from the borrower is done by the separate remove_collateral ix).
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BinLiquidityReduction {
    pub bin_id: i32,
    pub bps_to_remove: u16,
}

// Exact per-bin deposit amount. Real lamports in a bin = amount * decompress_multiplier.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CompressedBinDepositAmount {
    pub bin_id: i32,
    pub amount: u32,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AddLiquiditySingleSidePreciseParameter {
    pub bins: Vec<CompressedBinDepositAmount>,
    pub decompress_multiplier: u64,
}

pub fn handler(
    ctx: Context<DepositCollateral>,
    _bin_liquidity_reductions: Vec<BinLiquidityReduction>,
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

    // -------------------------------------------------------
    // Step 1: Transfer the EXACT total to be deposited
    // (sum of per-bin amounts * multiplier) from borrower ATA -> escrow PDA ATA.
    // Borrower signs the SPL token transfer. No more full-balance sweep.
    // -------------------------------------------------------
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

    msg!("Transferring exact total {} lamports of wSOL to escrow PDA ATA", total_y);

    let spl_transfer_ix = spl_token_transfer_instruction(
        &ctx.accounts.token_y_program.key(),
        &ctx.accounts.user_token_y.key(),
        &ctx.accounts.escrow_token_y.key(),
        &ctx.accounts.borrower.key(),
        total_y,
    );

    invoke(
        &spl_transfer_ix,
        &[
            ctx.accounts.user_token_y.to_account_info(),
            ctx.accounts.escrow_token_y.to_account_info(),
            ctx.accounts.borrower.to_account_info(),
            ctx.accounts.token_y_program.to_account_info(),
        ],
    )?;

    msg!("wSOL transfer to escrow PDA ATA succeeded");

    // -------------------------------------------------------
    // Step 2: add_liquidity_one_side_precise to escrow-owned position.
    // escrow_pda signs via invoke_signed. One-sided (Y) => only Y-side accounts.
    // IDL account order:
    //  0 position, 1 lb_pair, 2 bin_array_bitmap_extension (optional),
    //  3 user_token, 4 reserve, 5 token_mint, 6 bin_array_lower,
    //  7 bin_array_upper, 8 sender(signer), 9 token_program,
    //  10 event_authority, 11 program
    // -------------------------------------------------------
    let add_accounts = vec![
        AccountMeta::new(ctx.accounts.escrow_position.key(), false),
        AccountMeta::new(ctx.accounts.lb_pair.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_bitmap_ext.key(), false),
        AccountMeta::new(ctx.accounts.escrow_token_y.key(), false),
        AccountMeta::new(ctx.accounts.reserve_y.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_y_mint.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_lower.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_upper.key(), false),
        AccountMeta::new_readonly(ctx.accounts.escrow_pda.key(), true),
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

    invoke_signed(
        &add_ix,
        &[
            ctx.accounts.escrow_position.to_account_info(),
            ctx.accounts.lb_pair.to_account_info(),
            ctx.accounts.bin_array_bitmap_ext.to_account_info(),
            ctx.accounts.escrow_token_y.to_account_info(),
            ctx.accounts.reserve_y.to_account_info(),
            ctx.accounts.token_y_mint.to_account_info(),
            ctx.accounts.bin_array_lower.to_account_info(),
            ctx.accounts.bin_array_upper.to_account_info(),
            ctx.accounts.escrow_pda.to_account_info(),
            ctx.accounts.token_y_program.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.lb_clmm_program.to_account_info(),
        ],
        signer_seeds,
    )?;

    msg!("add_liquidity_one_side_precise CPI succeeded");

    ctx.accounts.escrow_state.position = ctx.accounts.escrow_position.key();

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
    Instruction {
        program_id: *token_program,
        accounts,
        data,
    }
}