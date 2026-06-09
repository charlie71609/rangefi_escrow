use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
use anchor_lang::solana_program::program::invoke;
use crate::ESCROW_SEED;

pub const REMOVE_LIQUIDITY_DISCRIMINATOR: [u8; 8] = [80, 85, 209, 72, 24, 206, 177, 108];

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BinLiquidityReduction {
    pub bin_id: i32,
    pub bps_to_remove: u16,
}

pub fn handler(
    ctx: Context<crate::RemoveCollateral>,
    bin_liquidity_reductions: Vec<BinLiquidityReduction>,
) -> Result<()> {
    let remove_accounts = vec![
        AccountMeta::new(ctx.accounts.position.key(), false),
        AccountMeta::new(ctx.accounts.lb_pair.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_bitmap_ext.key(), false),
        AccountMeta::new(ctx.accounts.user_token_x.key(), false),
        AccountMeta::new(ctx.accounts.user_token_y.key(), false),
        AccountMeta::new(ctx.accounts.reserve_x.key(), false),
        AccountMeta::new(ctx.accounts.reserve_y.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_x_mint.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_y_mint.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_lower.key(), false),
        AccountMeta::new(ctx.accounts.bin_array_upper.key(), false),
        AccountMeta::new_readonly(ctx.accounts.borrower.key(), true),
        AccountMeta::new_readonly(ctx.accounts.token_x_program.key(), false),
        AccountMeta::new_readonly(ctx.accounts.token_y_program.key(), false),
        AccountMeta::new_readonly(ctx.accounts.event_authority.key(), false),
        AccountMeta::new_readonly(ctx.accounts.lb_clmm_program.key(), false),
    ];

    let mut data = REMOVE_LIQUIDITY_DISCRIMINATOR.to_vec();
    bin_liquidity_reductions.serialize(&mut data)?;

    let ix = Instruction {
        program_id: ctx.accounts.lb_clmm_program.key(),
        accounts: remove_accounts,
        data,
    };

    invoke(
        &ix,
        &[
            ctx.accounts.position.to_account_info(),
            ctx.accounts.lb_pair.to_account_info(),
            ctx.accounts.bin_array_bitmap_ext.to_account_info(),
            ctx.accounts.user_token_x.to_account_info(),
            ctx.accounts.user_token_y.to_account_info(),
            ctx.accounts.reserve_x.to_account_info(),
            ctx.accounts.reserve_y.to_account_info(),
            ctx.accounts.token_x_mint.to_account_info(),
            ctx.accounts.token_y_mint.to_account_info(),
            ctx.accounts.bin_array_lower.to_account_info(),
            ctx.accounts.bin_array_upper.to_account_info(),
            ctx.accounts.borrower.to_account_info(),
            ctx.accounts.token_x_program.to_account_info(),
            ctx.accounts.token_y_program.to_account_info(),
            ctx.accounts.event_authority.to_account_info(),
            ctx.accounts.lb_clmm_program.to_account_info(),
        ],
    )?;

    msg!("remove_liquidity succeeded");
    Ok(())
}