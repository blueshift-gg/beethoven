use {
    beethoven::{try_from_tagged_swap_context, SwapContext, SwapData, SwapProtocolTag},
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
fn test_scale_vmm_swap_data_parses_buy_and_sell() {
    let buy = beethoven::scale_vmm::ScaleVmmSwapData::try_from(&[0u8][..]).unwrap();
    assert_eq!(buy.side, beethoven::scale_vmm::ScaleVmmSide::Buy);

    let sell = beethoven::scale_vmm::ScaleVmmSwapData::try_from(&[1u8][..]).unwrap();
    assert_eq!(sell.side, beethoven::scale_vmm::ScaleVmmSide::Sell);
}

#[test]
fn test_scale_vmm_swap_data_invalid_side_fails() {
    let err = beethoven::scale_vmm::ScaleVmmSwapData::try_from(&[2u8][..]).unwrap_err();
    assert_eq!(err, ProgramError::InvalidInstructionData);
}

#[test]
fn test_scale_vmm_swap_data_empty_fails() {
    let empty: &[u8] = &[];
    let err = beethoven::scale_vmm::ScaleVmmSwapData::try_from(empty).unwrap_err();
    assert_eq!(err, ProgramError::InvalidInstructionData);
}

#[test]
fn test_scale_vmm_accounts_try_from_requires_minimum_accounts() {
    let (_storage, accounts) = build_accounts(21, beethoven::scale_vmm::SCALE_VMM_PROGRAM_ID);

    let err = match beethoven::scale_vmm::ScaleVmmSwapAccounts::try_from(accounts.as_slice()) {
        Ok(_) => panic!("expected NotEnoughAccountKeys"),
        Err(err) => err,
    };
    assert_eq!(err, ProgramError::NotEnoughAccountKeys);
}

#[test]
fn test_scale_vmm_accounts_try_from_parses_beneficiary_tail() {
    let (_storage, accounts) = build_accounts(24, beethoven::scale_vmm::SCALE_VMM_PROGRAM_ID);

    let ctx = beethoven::scale_vmm::ScaleVmmSwapAccounts::try_from(accounts.as_slice()).unwrap();
    assert_eq!(ctx.beneficiary_accounts.len(), 2);
    assert_eq!(
        ctx.beneficiary_accounts[0].address(),
        accounts[22].address(),
    );
    assert_eq!(
        ctx.beneficiary_accounts[1].address(),
        accounts[23].address(),
    );
}

#[test]
fn test_try_from_tagged_swap_context_selects_scale_vmm() {
    let (_storage, accounts) = build_accounts(22, beethoven::scale_vmm::SCALE_VMM_PROGRAM_ID);

    let (ctx, rest) =
        try_from_tagged_swap_context(SwapProtocolTag::ScaleVmm, accounts.as_slice(), 0).unwrap();
    assert!(matches!(ctx, SwapContext::ScaleVmm(_)));
    assert!(rest.is_empty());
}

#[test]
fn test_scale_vmm_context_try_from_swap_data_variants() {
    let (_storage, accounts) = build_accounts(22, beethoven::scale_vmm::SCALE_VMM_PROGRAM_ID);
    let (ctx, _) =
        try_from_tagged_swap_context(SwapProtocolTag::ScaleVmm, accounts.as_slice(), 0).unwrap();

    let (buy_data, rest) = ctx.try_from_swap_data(&[0u8][..]).unwrap();
    assert!(rest.is_empty());
    match buy_data {
        SwapData::ScaleVmm(data) => {
            assert_eq!(data.side, beethoven::scale_vmm::ScaleVmmSide::Buy);
        }
        _ => panic!("expected ScaleVmm swap data"),
    }

    let err = match ctx.try_from_swap_data(&[9u8][..]) {
        Ok(_) => panic!("expected InvalidInstructionData"),
        Err(err) => err,
    };
    assert_eq!(err, ProgramError::InvalidInstructionData);
}

#[test]
fn test_scale_vmm_accounts_reject_too_many_beneficiaries() {
    let (_storage, accounts) = build_accounts(28, beethoven::scale_vmm::SCALE_VMM_PROGRAM_ID);

    let err = match beethoven::scale_vmm::ScaleVmmSwapAccounts::try_from(accounts.as_slice()) {
        Ok(_) => panic!("expected InvalidAccountData"),
        Err(err) => err,
    };
    assert_eq!(err, ProgramError::InvalidAccountData);
}
