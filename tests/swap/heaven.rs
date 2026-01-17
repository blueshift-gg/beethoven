use {crate::helper::*, solana_keypair::Keypair, solana_signer::Signer};

#[test]
fn test_heaven_swap() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // TODO: Load beethoven-test program
    // TODO: Load heaven program or mock
    // TODO: Set up accounts from fixtures/swap/heaven/
    // TODO: Execute swap instruction with extra_data: event bytes (can be empty)
    // TODO: Verify results
}
