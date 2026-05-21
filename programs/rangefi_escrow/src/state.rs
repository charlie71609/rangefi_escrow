use anchor_lang::prelude::*;

#[account]
pub struct EscrowState {
    pub borrower: Pubkey,       // wallet that deposited the position
    pub position: Pubkey,       // the Meteora position account owned by escrow PDA
    pub lb_pair: Pubkey,        // the Meteora pool
    pub bump: u8,               // PDA bump
}

impl EscrowState {
    pub const LEN: usize = 8 + 32 + 32 + 32 + 1;
}