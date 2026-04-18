use {
    beethoven::{try_from_tagged_swap_context, SwapContext, SwapProtocolTag},
    solana_account_view::{AccountView, RuntimeAccount, NOT_BORROWED},
    solana_address::Address,
    solana_program_error::ProgramError,
};

fn make_account(address: Address) -> (Vec<u64>, AccountView) {
    let mut backing =
        vec![0u64; core::mem::size_of::<RuntimeAccount>() / core::mem::size_of::<u64>() + 1];
    let raw = backing.as_mut_ptr() as *mut RuntimeAccount;

    unsafe {
        (*raw).borrow_state = NOT_BORROWED;
        (*raw).is_signer = 0;
        (*raw).is_writable = 1;
        (*raw).executable = 0;
        (*raw).resize_delta = 0;
        (*raw).address = address;
        (*raw).owner = Address::new_from_array([9u8; 32]);
        (*raw).lamports = 0;
        (*raw).data_len = 8;
    }

    let view = unsafe { AccountView::new_unchecked(raw) };
    (backing, view)
}

fn build_accounts(
    total_accounts: usize,
    first_account: Address,
) -> (Vec<Vec<u64>>, Vec<AccountView>) {
    let mut storage = Vec::with_capacity(total_accounts);
    let mut views = Vec::with_capacity(total_accounts);

    for i in 0..total_accounts {
        let address = if i == 0 {
            first_account
        } else {
            Address::new_from_array([i as u8; 32])
        };
        let (backing, view) = make_account(address);
        storage.push(backing);
        views.push(view);
    }

    (storage, views)
}

#[test]
fn test_swap_protocol_tag_invalid_byte_fails() {
    let err = SwapProtocolTag::from_byte(255).unwrap_err();
    assert_eq!(err, ProgramError::InvalidInstructionData);
}

#[test]
fn test_try_from_tagged_swap_context_consumes_fixed_prefix() {
    let (_storage, accounts) = build_accounts(15, beethoven::gamma::GAMMA_PROGRAM_ID);

    let (ctx, rest) =
        try_from_tagged_swap_context(SwapProtocolTag::Gamma, accounts.as_slice(), 0).unwrap();

    assert!(matches!(ctx, SwapContext::Gamma(_)));
    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].address(), accounts[14].address());
}

#[test]
fn test_try_from_tagged_swap_context_consumes_explicit_dynamic_tail() {
    let (_storage, accounts) = build_accounts(18, beethoven::scale_amm::SCALE_AMM_PROGRAM_ID);

    let (ctx, rest) =
        try_from_tagged_swap_context(SwapProtocolTag::ScaleAmm, accounts.as_slice(), 2).unwrap();

    match ctx {
        SwapContext::ScaleAmm(scale_amm) => {
            assert_eq!(scale_amm.beneficiary_accounts.len(), 2);
            assert_eq!(
                scale_amm.beneficiary_accounts[0].address(),
                accounts[15].address(),
            );
            assert_eq!(
                scale_amm.beneficiary_accounts[1].address(),
                accounts[16].address(),
            );
        }
        _ => panic!("expected ScaleAmm context"),
    }

    assert_eq!(rest.len(), 1);
    assert_eq!(rest[0].address(), accounts[17].address());
}

#[test]
fn test_try_from_tagged_swap_context_fixed_protocol_consumes_explicit_extra_accounts() {
    let (_storage, accounts) = build_accounts(15, beethoven::gamma::GAMMA_PROGRAM_ID);

    let (ctx, rest) =
        try_from_tagged_swap_context(SwapProtocolTag::Gamma, accounts.as_slice(), 1).unwrap();

    assert!(matches!(ctx, SwapContext::Gamma(_)));
    assert!(rest.is_empty());
}

#[test]
fn test_try_from_tagged_swap_context_rejects_mismatched_first_account() {
    let (_storage, accounts) = build_accounts(15, beethoven::manifest::MANIFEST_PROGRAM_ID);

    let err = match try_from_tagged_swap_context(SwapProtocolTag::Gamma, accounts.as_slice(), 0) {
        Ok(_) => panic!("expected InvalidAccountData"),
        Err(err) => err,
    };

    assert_eq!(err, ProgramError::InvalidAccountData);
}
