#![no_std]

use {
    beethoven_core::Swap,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const NIRVANA_GOVERNANCE_PROGRAM_ID: Address =
    address!("NirvHuZvrm2zSxjkBvSbaF2tHfP5j7cvMj9QmdoHVwb");
pub const NIRV_MINT: Address = address!("3eamaYJ7yicyRd3mYz4YeNyNPGVo6zMmKUp5UP25AxRM");

const BUY_EXACT2_DISCRIMINATOR: [u8; 8] = [109, 5, 199, 243, 164, 233, 19, 152];
const SELL2_DISCRIMINATOR: [u8; 8] = [47, 191, 120, 1, 28, 35, 253, 79];

pub struct NirvanaGovernance;

impl NirvanaGovernanceSwapAccounts<'_> {
    pub const NUM_ACCOUNTS: usize = 14;
}

pub struct NirvanaGovernanceSwapHeadAccounts<'info> {
    pub nirvana_program: &'info AccountView,
    pub payer: &'info AccountView,
    pub tenant: &'info AccountView,
    pub price_curve: &'info AccountView,
    pub mint_ana: &'info AccountView,
}

pub enum NirvanaGovernanceSwapBodyAccounts<'info> {
    BuyExact2 {
        mint_niv: &'info AccountView,
        mint_main: &'info AccountView,
        backing_vault_main: &'info AccountView,
        backing_vault_nirv: &'info AccountView,
        escrow_rev_ana: &'info AccountView,
        backing_src: &'info AccountView,
        ana_dst: &'info AccountView,
    },
    Sell2 {
        backing_dst: &'info AccountView,
        escrow_rev_ana: &'info AccountView,
        backing_vault_main: &'info AccountView,
        backing_vault_nirv: &'info AccountView,
        ana_src: &'info AccountView,
        mint_nirv: &'info AccountView,
        mint_main: &'info AccountView,
    },
}

pub struct NirvanaGovernanceSwapLegAccounts<'info> {
    pub token_program: &'info AccountView,
    pub token_program_main: &'info AccountView,
}

pub struct NirvanaGovernanceSwapAccounts<'info> {
    pub head: NirvanaGovernanceSwapHeadAccounts<'info>,
    pub body: NirvanaGovernanceSwapBodyAccounts<'info>,
    pub leg: NirvanaGovernanceSwapLegAccounts<'info>,
}

impl<'info> TryFrom<&'info [AccountView]> for NirvanaGovernanceSwapAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        if accounts.len() != Self::NUM_ACCOUNTS {
            return Err(ProgramError::NotEnoughAccountKeys);
        }

        let head = NirvanaGovernanceSwapHeadAccounts {
            nirvana_program: &accounts[0],
            payer: &accounts[1],
            tenant: &accounts[2],
            price_curve: &accounts[3],
            mint_ana: &accounts[4],
        };

        // instruction is buy if account at index 5 is NIRV_MINT
        let body = if accounts[5].address() == &NIRV_MINT {
            NirvanaGovernanceSwapBodyAccounts::BuyExact2 {
                mint_niv: &accounts[5],
                mint_main: &accounts[6],
                backing_vault_main: &accounts[7],
                backing_vault_nirv: &accounts[8],
                escrow_rev_ana: &accounts[9],
                backing_src: &accounts[10],
                ana_dst: &accounts[11],
            }
        } else {
            NirvanaGovernanceSwapBodyAccounts::Sell2 {
                backing_dst: &accounts[5],
                escrow_rev_ana: &accounts[6],
                backing_vault_main: &accounts[7],
                backing_vault_nirv: &accounts[8],
                ana_src: &accounts[9],
                mint_nirv: &accounts[10],
                mint_main: &accounts[11],
            }
        };

        let leg = NirvanaGovernanceSwapLegAccounts {
            token_program: &accounts[12],
            token_program_main: &accounts[13],
        };

        Ok(NirvanaGovernanceSwapAccounts { head, body, leg })
    }
}

impl<'info> Swap<'info> for NirvanaGovernance {
    type Accounts = NirvanaGovernanceSwapAccounts<'info>;
    type Data = ();

    fn swap_signed(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        _data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let mut accounts = MaybeUninit::<[InstructionAccount<'info>; 13]>::uninit();
        let accounts_ptr = accounts.as_mut_ptr() as *mut InstructionAccount;

        unsafe {
            core::ptr::write(
                accounts_ptr,
                InstructionAccount::writable_signer(ctx.head.payer.address()),
            );
            core::ptr::write(
                accounts_ptr.add(1),
                InstructionAccount::writable(ctx.head.tenant.address()),
            );
            core::ptr::write(
                accounts_ptr.add(2),
                match ctx.body {
                    NirvanaGovernanceSwapBodyAccounts::BuyExact2 { .. } => {
                        InstructionAccount::readonly(ctx.head.price_curve.address())
                    }
                    NirvanaGovernanceSwapBodyAccounts::Sell2 { .. } => {
                        InstructionAccount::writable(ctx.head.price_curve.address())
                    }
                },
            );
            core::ptr::write(
                accounts_ptr.add(3),
                InstructionAccount::writable(ctx.head.mint_ana.address()),
            );

            match ctx.body {
                NirvanaGovernanceSwapBodyAccounts::BuyExact2 {
                    mint_niv,
                    mint_main,
                    backing_vault_main,
                    backing_vault_nirv,
                    escrow_rev_ana,
                    backing_src,
                    ana_dst,
                } => {
                    core::ptr::write(
                        accounts_ptr.add(4),
                        InstructionAccount::readonly(mint_niv.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(5),
                        InstructionAccount::readonly(mint_main.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(6),
                        InstructionAccount::writable(backing_vault_main.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(7),
                        InstructionAccount::writable(backing_vault_nirv.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(8),
                        InstructionAccount::writable(escrow_rev_ana.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(9),
                        InstructionAccount::writable(backing_src.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(10),
                        InstructionAccount::writable(ana_dst.address()),
                    );
                }
                NirvanaGovernanceSwapBodyAccounts::Sell2 {
                    backing_dst,
                    escrow_rev_ana,
                    backing_vault_main,
                    backing_vault_nirv,
                    ana_src,
                    mint_nirv,
                    mint_main,
                } => {
                    core::ptr::write(
                        accounts_ptr.add(4),
                        InstructionAccount::writable(backing_dst.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(5),
                        InstructionAccount::writable(escrow_rev_ana.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(6),
                        InstructionAccount::writable(backing_vault_main.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(7),
                        InstructionAccount::writable(backing_vault_nirv.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(8),
                        InstructionAccount::writable(ana_src.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(9),
                        InstructionAccount::readonly(mint_nirv.address()),
                    );
                    core::ptr::write(
                        accounts_ptr.add(10),
                        InstructionAccount::readonly(mint_main.address()),
                    );
                }
            }

            core::ptr::write(
                accounts_ptr.add(11),
                InstructionAccount::readonly(ctx.leg.token_program.address()),
            );
            core::ptr::write(
                accounts_ptr.add(12),
                InstructionAccount::readonly(ctx.leg.token_program_main.address()),
            );
        }

        let accounts = unsafe { core::slice::from_raw_parts(accounts_ptr, 13) };

        let mut accounts_views = [ctx.head.nirvana_program; 13];
        accounts_views[0] = ctx.head.payer;
        accounts_views[1] = ctx.head.tenant;
        accounts_views[2] = ctx.head.price_curve;
        accounts_views[3] = ctx.head.mint_ana;

        match ctx.body {
            NirvanaGovernanceSwapBodyAccounts::BuyExact2 {
                mint_niv,
                mint_main,
                backing_vault_main,
                backing_vault_nirv,
                escrow_rev_ana,
                backing_src,
                ana_dst,
            } => {
                accounts_views[4] = mint_niv;
                accounts_views[5] = mint_main;
                accounts_views[6] = backing_vault_main;
                accounts_views[7] = backing_vault_nirv;
                accounts_views[8] = escrow_rev_ana;
                accounts_views[9] = backing_src;
                accounts_views[10] = ana_dst;
            }
            NirvanaGovernanceSwapBodyAccounts::Sell2 {
                backing_dst,
                escrow_rev_ana,
                backing_vault_main,
                backing_vault_nirv,
                ana_src,
                mint_nirv,
                mint_main,
            } => {
                accounts_views[4] = backing_dst;
                accounts_views[5] = escrow_rev_ana;
                accounts_views[6] = backing_vault_main;
                accounts_views[7] = backing_vault_nirv;
                accounts_views[8] = ana_src;
                accounts_views[9] = mint_nirv;
                accounts_views[10] = mint_main;
            }
        }

        accounts_views[11] = ctx.leg.token_program;
        accounts_views[12] = ctx.leg.token_program_main;

        let mut instruction_data = MaybeUninit::<[u8; 24]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            let discriminator = match ctx.body {
                NirvanaGovernanceSwapBodyAccounts::BuyExact2 { .. } => BUY_EXACT2_DISCRIMINATOR,
                NirvanaGovernanceSwapBodyAccounts::Sell2 { .. } => SELL2_DISCRIMINATOR,
            };
            core::ptr::copy_nonoverlapping(discriminator.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(in_amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
            core::ptr::copy_nonoverlapping(
                minimum_out_amount.to_le_bytes().as_ptr(),
                ptr.add(16),
                8,
            );
        }

        let instruction = InstructionView {
            program_id: &NIRVANA_GOVERNANCE_PROGRAM_ID,
            accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed(&instruction, &accounts_views, signer_seeds)
    }

    fn swap(
        ctx: &Self::Accounts,
        in_amount: u64,
        minimum_out_amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::swap_signed(ctx, in_amount, minimum_out_amount, data, &[])
    }
}
