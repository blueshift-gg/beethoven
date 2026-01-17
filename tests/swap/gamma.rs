use {crate::helper::*, solana_keypair::Keypair, solana_signer::Signer};

#[test]
fn test_gamma_swap() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // TODO: Load beethoven-test program
    // TODO: Load gamma program or mock
    // TODO: Set up accounts from fixtures/swap/gamma/
    // TODO: Execute swap instruction with extra_data: [] (gamma has no extra data)
    // TODO: Verify results
}
