use {
    anchor_lang::{solana_program::instruction::Instruction, InstructionData, ToAccountMetas},
    litesvm::LiteSVM,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_keypair::Keypair,
    solana_transaction::versioned::VersionedTransaction,
    anchor_lang::solana_program::pubkey::Pubkey,
};

#[test]
fn test_initialize() {
    let program_id = rangefi_escrow::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!("../../../target/deploy/rangefi_escrow.so");
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    let (escrow_state, _) = Pubkey::find_program_address(
        &[b"escrow", payer.pubkey().as_ref()],
        &program_id,
    );
    let (escrow_pda, _) = Pubkey::find_program_address(
        &[b"escrow", payer.pubkey().as_ref()],
        &program_id,
    );

    let instruction = Instruction::new_with_bytes(
        program_id,
        &rangefi_escrow::instruction::Initialize {}.data(),
        rangefi_escrow::accounts::Initialize {
            borrower: payer.pubkey(),
            escrow_state,
            escrow_pda,
            system_program: anchor_lang::solana_program::system_program::id(),
        }.to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&payer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[payer]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok());
}