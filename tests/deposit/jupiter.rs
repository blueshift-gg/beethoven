use {crate::helper::*, solana_keypair::Keypair, solana_signer::Signer};

#[test]
fn test_jupiter_deposit() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // TODO: Load beethoven-test program
    // TODO: Load jupiter program or mock
    // TODO: Set up accounts from fixtures
    // TODO: Execute deposit instruction
    // TODO: Verify results
}
