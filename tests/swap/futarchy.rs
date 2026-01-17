use {crate::helper::*, solana_keypair::Keypair, solana_signer::Signer};

#[test]
fn test_futarchy_swap() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // TODO: Load beethoven-test program
    // TODO: Load futarchy program or mock
    // TODO: Set up accounts from fixtures/swap/futarchy/
    // TODO: Execute swap instruction with extra_data: [swap_type] (0=Buy, 1=Sell)
    // TODO: Verify results
}
