use {crate::helper::*, solana_keypair::Keypair, solana_signer::Signer};

#[test]
fn test_perena_swap() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // TODO: Load beethoven-test program
    // TODO: Load perena program or mock
    // TODO: Set up accounts from fixtures/swap/perena/
    // TODO: Execute swap instruction with extra_data: [in_index, out_index]
    // TODO: Verify results
}
