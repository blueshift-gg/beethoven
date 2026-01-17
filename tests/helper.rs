use {
    litesvm::LiteSVM,
    solana_account::Account,
    solana_address::{address, Address},
    solana_instruction::{AccountMeta, Instruction},
    solana_keypair::Keypair,
    solana_program_option::COption,
    solana_program_pack::Pack,
    solana_rent::Rent,
    solana_signer::Signer,
    solana_transaction::Transaction,
    spl_token_interface::state::{Account as TokenAccount, AccountState, Mint},
};

// =============================================================================
// Constants
// =============================================================================

pub const TEST_PROGRAM_ID: Address = Address::new_from_array([0x01; 32]);
pub const TOKEN_PROGRAM_ID: Address = address!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

// Protocol program IDs (for detection)
pub const KAMINO_PROGRAM_ID: Address = address!("KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD");
pub const JUPITER_PROGRAM_ID: Address = address!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");
pub const PERENA_PROGRAM_ID: Address = address!("NUMERUNsFCP3kuNmWZuXtm1AaQCPj9uw6Guv2Ekoi5P");
pub const SOLFI_PROGRAM_ID: Address = address!("SoLFiHG9TfgtdUXUjWAxi3LtvYuFyDLVhBWxdMZxyCe");
pub const GAMMA_PROGRAM_ID: Address = address!("GAMMA7meSFWaBXF25oSUgmGRwaWJfSFLQzPiSfPKqp2W");
pub const MANIFEST_PROGRAM_ID: Address = address!("MNFSTqtC93rEfYHB6hF82sKdZpUDFWkViLByLd1k1Ms");
pub const SYSTEM_PROGRAM_ID: Address = address!("11111111111111111111111111111111");

pub mod discriminator {
    pub const DEPOSIT: u8 = 0;
    pub const SWAP: u8 = 1;
}

// =============================================================================
// SVM Setup
// =============================================================================

pub fn setup_svm() -> LiteSVM {
    LiteSVM::new()
}

pub fn setup_svm_with_program(program_bytes: &[u8]) -> LiteSVM {
    let mut svm = LiteSVM::new();
    let _ = svm.add_program(TEST_PROGRAM_ID, program_bytes);
    svm
}

// =============================================================================
// Token Program Helpers
// =============================================================================

/// Create an Account for a Mint
pub fn create_account_for_mint(mint_data: Mint) -> Account {
    let mut data = vec![0u8; Mint::LEN];
    Mint::pack(mint_data, &mut data).unwrap();

    Account {
        lamports: Rent::default().minimum_balance(Mint::LEN),
        data,
        owner: TOKEN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create an Account for a Token Account
pub fn create_account_for_token_account(token_account_data: TokenAccount) -> Account {
    let mut data = vec![0u8; TokenAccount::LEN];
    TokenAccount::pack(token_account_data, &mut data).unwrap();

    Account {
        lamports: Rent::default().minimum_balance(TokenAccount::LEN),
        data,
        owner: TOKEN_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create and set a token account in the SVM
pub fn create_token_account(
    svm: &mut LiteSVM,
    owner: &Address,
    mint: &Address,
    amount: u64,
) -> Address {
    let pubkey = Keypair::new().pubkey();
    let account = create_account_for_token_account(TokenAccount {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    });
    svm.set_account(pubkey, account).unwrap();
    pubkey
}

/// Create and set a token account at a specific address
pub fn create_token_account_at(
    svm: &mut LiteSVM,
    pubkey: Address,
    owner: &Address,
    mint: &Address,
    amount: u64,
) {
    let account = create_account_for_token_account(TokenAccount {
        mint: *mint,
        owner: *owner,
        amount,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    });
    svm.set_account(pubkey, account).unwrap();
}

/// Create and set a mint in the SVM
pub fn create_mint(svm: &mut LiteSVM, mint_authority: &Address, decimals: u8) -> Address {
    let pubkey = Keypair::new().pubkey();
    let account = create_account_for_mint(Mint {
        mint_authority: COption::Some(*mint_authority),
        supply: 0,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    });
    svm.set_account(pubkey, account).unwrap();
    pubkey
}

/// Create and set a mint at a specific address
pub fn create_mint_at(
    svm: &mut LiteSVM,
    pubkey: Address,
    mint_authority: &Address,
    decimals: u8,
    supply: u64,
) {
    let account = create_account_for_mint(Mint {
        mint_authority: COption::Some(*mint_authority),
        supply,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    });
    svm.set_account(pubkey, account).unwrap();
}

// =============================================================================
// Mock Protocol Account Helpers
// =============================================================================

pub fn create_program_account(svm: &mut LiteSVM, program_id: Address) {
    svm.set_account(
        program_id,
        Account {
            lamports: Rent::default().minimum_balance(0),
            data: vec![],
            owner: solana_sdk_ids::bpf_loader::ID,
            executable: true,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

pub fn create_mock_account(svm: &mut LiteSVM, owner: &Address, data: Vec<u8>) -> Address {
    let pubkey = Keypair::new().pubkey();
    svm.set_account(
        pubkey,
        Account {
            lamports: Rent::default().minimum_balance(data.len()),
            data,
            owner: *owner,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
    pubkey
}

pub fn create_mock_account_at(svm: &mut LiteSVM, pubkey: Address, owner: &Address, data: Vec<u8>) {
    svm.set_account(
        pubkey,
        Account {
            lamports: Rent::default().minimum_balance(data.len()),
            data,
            owner: *owner,
            executable: false,
            rent_epoch: 0,
        },
    )
    .unwrap();
}

// =============================================================================
// Instruction Builders
// =============================================================================

pub fn build_deposit_instruction(accounts: Vec<AccountMeta>, amount: u64) -> Instruction {
    let mut data = vec![discriminator::DEPOSIT];
    data.extend_from_slice(&amount.to_le_bytes());

    Instruction {
        program_id: TEST_PROGRAM_ID,
        accounts,
        data,
    }
}

pub fn build_swap_instruction(
    accounts: Vec<AccountMeta>,
    in_amount: u64,
    min_out_amount: u64,
    extra_data: &[u8],
) -> Instruction {
    let mut data = vec![discriminator::SWAP];
    data.extend_from_slice(&in_amount.to_le_bytes());
    data.extend_from_slice(&min_out_amount.to_le_bytes());
    data.extend_from_slice(extra_data);

    Instruction {
        program_id: TEST_PROGRAM_ID,
        accounts,
        data,
    }
}

// =============================================================================
// Transaction Helpers
// =============================================================================

pub fn send_transaction(
    svm: &mut LiteSVM,
    payer: &Keypair,
    instruction: Instruction,
) -> Result<(), String> {
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

pub fn send_transaction_with_signers(
    svm: &mut LiteSVM,
    payer: &Keypair,
    signers: &[&Keypair],
    instruction: Instruction,
) -> Result<(), String> {
    let mut all_signers: Vec<&Keypair> = vec![payer];
    all_signers.extend(signers);

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &all_signers,
        svm.latest_blockhash(),
    );

    svm.send_transaction(tx)
        .map(|_| ())
        .map_err(|e| format!("{:?}", e))
}

// =============================================================================
// Fixture Loading
// =============================================================================

pub fn load_fixture_bytes(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read fixture: {}", path))
}

pub fn load_fixture_account(path: &str, owner: &Address) -> Account {
    let data = load_fixture_bytes(path);
    Account {
        lamports: Rent::default().minimum_balance(data.len()),
        data,
        owner: *owner,
        executable: false,
        rent_epoch: 0,
    }
}

/// Load a JSON fixture exported by `solana account --output json-compact`
/// Returns (pubkey, Account)
pub fn load_json_fixture(path: &str) -> (Address, Account) {
    use {
        base64::{engine::general_purpose::STANDARD, Engine},
        std::str::FromStr,
    };

    let contents = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", path));
    let json: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|_| panic!("Failed to parse JSON: {}", path));

    let pubkey_str = json["pubkey"].as_str().expect("Missing pubkey field");
    let pubkey = Address::from_str(pubkey_str).expect("Invalid pubkey");

    let account_json = &json["account"];
    let lamports = account_json["lamports"].as_u64().expect("Missing lamports");
    let owner_str = account_json["owner"].as_str().expect("Missing owner");
    let owner = Address::from_str(owner_str).expect("Invalid owner pubkey");
    let executable = account_json["executable"].as_bool().unwrap_or(false);

    let data_array = account_json["data"].as_array().expect("Missing data array");
    let data_b64 = data_array[0].as_str().expect("Missing data string");
    let data = STANDARD
        .decode(data_b64)
        .expect("Failed to decode base64 data");

    (
        pubkey,
        Account {
            lamports,
            data,
            owner,
            executable,
            rent_epoch: 0,
        },
    )
}

/// Load JSON fixture and set it in the SVM
pub fn load_and_set_json_fixture(svm: &mut LiteSVM, path: &str) -> Address {
    let (pubkey, account) = load_json_fixture(path);
    svm.set_account(pubkey, account).unwrap();
    pubkey
}

/// Load and deploy a program from .so file
pub fn load_program(svm: &mut LiteSVM, program_id: Address, so_path: &str) {
    let program_bytes = load_fixture_bytes(so_path);
    let _ = svm.add_program(program_id, &program_bytes);
}
