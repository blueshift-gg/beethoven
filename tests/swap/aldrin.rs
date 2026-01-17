use {crate::helper::*, solana_keypair::Keypair, solana_signer::Signer};

#[test]
fn test_aldrin_swap() {
    let mut svm = setup_svm();
    let payer = Keypair::new();
    svm.airdrop(&payer.pubkey(), 10_000_000_000).unwrap();

    // TODO: Load beethoven-test program
    // TODO: Load aldrin program or mock
    // TODO: Set up accounts from fixtures/swap/aldrin/
    // TODO: Execute swap instruction with extra_data: [side] (0=Bid, 1=Ask)
    // TODO: Verify results
}
