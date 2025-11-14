# Beethoven

**A universal interface for CPI interactions, bounded by actions and routed client-side on Solana**.

---

## What is this?

A Solana SDK that lets you build protocol-agnostic programs. Instead of hardcoding integrations for Kamino, Jupiter, Marginfi, etc., you write one instruction and let the client choose which protocol to route to.

```rust
// Your entire program
use beethoven::deposit;

#[program]
mod vault {
    pub fn deposit_anywhere(ctx: Context<Deposit>) -> Result<()> {
        deposit(&ctx.remaining_accounts, amount)?;
        Ok(())
    }
}
```

Client picks the protocol:
```typescript
// Kamino
tx.remainingAccounts([KAMINO_PROGRAM_ID, reserve, obligation, ...])

// Jupiter
tx.remainingAccounts([JUPITER_PROGRAM_ID, vault, userAccount, ...])

// Same instruction, different protocol
```

---

## How it works

**Protocol detection:** First account in the slice must be the target program ID.

```rust
pub fn try_from_deposit_context(accounts: &[AccountInfo])
    -> Result<DepositContext, ProgramError>
{
    let detector_account = accounts.first()?;

    if pubkey_eq(detector_account.key(), &KAMINO_PROGRAM_ID) {
        return Ok(DepositContext::Kamino(parse_kamino_accounts(accounts)?));
    }

    if pubkey_eq(detector_account.key(), &JUPITER_PROGRAM_ID) {
        return Ok(DepositContext::Jupiter(parse_jupiter_accounts(accounts)?));
    }

    Err(ProgramError::InvalidAccountData)
}
```

**Type-safe contexts:** Pattern match for custom validation before executing.

```rust
let ctx = try_from_deposit_context(accounts)?;

match &ctx {
    DepositContext::Kamino(k) => {
        require!(k.reserve.key() == approved_reserve);
    }
    DepositContext::Jupiter(j) => {
        // different validation
    }
}

DepositContext::deposit(&ctx, amount)?;
```

**Feature flags:** Explicit security model.

Protocols are opt-in, not opt-out. When new protocols are added to Beethoven:
- Existing programs are unaffected
- You choose which protocols to trust
- Each integration is a conscious security decision

```toml
# You audit and explicitly enable each protocol
beethoven = { features = ["kamino", "jupiter"] }  # Only these two
```

Your program will never route to protocols you haven't reviewed.

---

## API

Three usage levels:

```rust
// 1. Convenience - auto-detect protocol and execute
// Use when: You don't need custom validation
beethoven::deposit(&accounts, amount)?;

// 2. Protocol-agnostic validation - auto-detect then validate
// Use when: You need to inspect/validate accounts before executing,
//           but want to support multiple protocols
let ctx = try_from_deposit_context(&accounts)?;
match &ctx {
    DepositContext::Kamino(k) => {
        require!(k.reserve.key() == approved_reserve);
    }
    DepositContext::Jupiter(j) => {
        require!(j.vault.key() == approved_vault);
    }
}
DepositContext::deposit(&ctx, amount)?;

// 3. Protocol-specific - skip auto-detection
// Use when: You know exactly which protocol you're calling
let ctx = KaminoDepositAccounts::try_from(&accounts)?;
Kamino::deposit(&ctx, amount)?;
```

All support PDA signing via `deposit_signed(accounts, amount, &[signer_seeds])`.

---

## Integrating Your Protocol

**For protocol developers:** Submit a PR to make your protocol available to all Beethoven users.

Once integrated, any program using Beethoven can immediately route to your protocol - no upgrades needed on their end. Users just enable your feature flag and pass your accounts.

### What you need to implement

For each action (deposit, withdraw, borrow, etc.), implement the corresponding trait:

```rust
// src/programs/your_protocol/mod.rs

pub const YOUR_PROTOCOL_PROGRAM_ID: [u8; 32] = [...];

pub struct YourProtocol;

pub struct YourProtocolDepositAccounts<'info> {
    pub user: &'info AccountInfo,
    pub vault: &'info AccountInfo,
    // ... your protocol's required accounts
}

// Parse accounts from raw slice
impl<'info> TryFrom<&'info [AccountInfo]> for YourProtocolDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountInfo]) -> Result<Self, Self::Error> {
        // Validate account count and parse
    }
}

// Implement the action trait
impl<'info> Deposit<'info> for YourProtocol {
    type Accounts = YourProtocolDepositAccounts<'info>;

    fn deposit_signed(
        ctx: &Self::Accounts,
        amount: u64,
        signer_seeds: &[Signer]
    ) -> ProgramResult {
        // Build instruction + invoke_signed to your program
    }

    fn deposit(ctx: &Self::Accounts, amount: u64) -> ProgramResult {
        Self::deposit_signed(ctx, amount, &[])
    }
}
```

Then add your protocol to the action's context enum:

```rust
// src/traits/deposit.rs

pub enum DepositContext<'info> {
    #[cfg(feature = "kamino")]
    Kamino(crate::programs::kamino::KaminoDepositAccounts<'info>),

    #[cfg(feature = "jupiter")]
    Jupiter(crate::programs::jupiter::JupiterEarnDepositAccounts<'info>),

    #[cfg(feature = "your_protocol")]
    YourProtocol(crate::programs::your_protocol::YourProtocolDepositAccounts<'info>),
}
```

And add detection logic:

```rust
// In try_from_deposit_context()

#[cfg(feature = "your_protocol")]
if pubkey_eq(detector_account.key(), &crate::programs::your_protocol::YOUR_PROTOCOL_PROGRAM_ID) {
    let ctx = crate::programs::your_protocol::YourProtocolDepositAccounts::try_from(accounts)?;
    return Ok(DepositContext::YourProtocol(ctx));
}
```

**That's it.** Submit the PR and programs can start routing to you.

---

## Supported actions

- `deposit` / `deposit_signed` - Kamino, Jupiter

More actions (withdraw, borrow, repay) coming when needed.

---

## Built with Pinocchio

Uses [pinocchio](https://github.com/fuzz-land/pinocchio) for zero-overhead abstractions. No anchor bloat.

---

**Beethoven** - Client-side protocol routing for Solana programs.
