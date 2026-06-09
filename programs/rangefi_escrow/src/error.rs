use anchor_lang::prelude::*;
#[error_code]
pub enum EscrowError {
    #[msg("Invalid Meteora DLMM program")]
    InvalidMeteoraProgram,
    #[msg("Position not owned by escrow PDA")]
    InvalidPositionOwner,
    #[msg("Unauthorized: borrower mismatch")]
    Unauthorized,
}