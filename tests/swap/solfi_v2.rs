use {crate::helper::*, solana_keypair::Keypair, solana_signer::Signer};

#[test]
fn test_solfi_v2_swap() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // TODO: Load beethoven-test program
    // TODO: Load solfi_v2 program or mock
    // TODO: Set up accounts from fixtures/swap/solfi_v2/
    // TODO: Execute swap instruction with extra_data: [is_quote_to_base]
    // TODO: Verify results
}
