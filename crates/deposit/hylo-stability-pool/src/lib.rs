#![no_std]

use {
    beethoven_core::Deposit,
    core::mem::MaybeUninit,
    solana_account_view::AccountView,
    solana_address::{address, Address},
    solana_instruction_view::{
        cpi::{invoke_signed, Signer},
        InstructionAccount, InstructionView,
    },
    solana_program_error::{ProgramError, ProgramResult},
};

pub const HYLO_STABILITY_PROGRAM_ID: Address =
    address!("HysTabVUfmQBFcmzu1ctRd1Y1fxd66RBpboy1bmtDSQQ");

pub const USER_DEPOSIT_DISCRIMINATOR: [u8; 8] = [186, 198, 140, 233, 129, 39, 98, 153];

pub struct HyloStabilityPool;

pub struct HyloStabilityPoolDepositAccounts<'info> {
    pub hylo_stability_program: &'info AccountView,
    pub user: &'info AccountView,
    pub pool_config: &'info AccountView,
    pub hylo: &'info AccountView,
    pub stablecoin_mint: &'info AccountView,
    pub levercoin_mint: &'info AccountView,
    pub user_stablecoin_ta: &'info AccountView,
    pub user_lp_token_ta: &'info AccountView,
    pub pool_auth: &'info AccountView,
    pub stablecoin_pool: &'info AccountView,
    pub levercoin_pool: &'info AccountView,
    pub lp_token_auth: &'info AccountView,
    pub lp_token_mint: &'info AccountView,
    pub sol_usd_pyth_feed: &'info AccountView,
    pub system_program: &'info AccountView,
    pub token_program: &'info AccountView,
    pub associated_token_program: &'info AccountView,
    pub event_authority: &'info AccountView,
    pub program: &'info AccountView,
}

impl<'info> TryFrom<&'info [AccountView]> for HyloStabilityPoolDepositAccounts<'info> {
    type Error = ProgramError;

    fn try_from(accounts: &'info [AccountView]) -> Result<Self, Self::Error> {
        let [hylo_stability_program, user, pool_config, hylo, stablecoin_mint, levercoin_mint, user_stablecoin_ta, user_lp_token_ta, pool_auth, stablecoin_pool, levercoin_pool, lp_token_auth, lp_token_mint, sol_usd_pyth_feed, system_program, token_program, associated_token_program, event_authority, program, ..] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        Ok(HyloStabilityPoolDepositAccounts {
            hylo_stability_program,
            user,
            pool_config,
            hylo,
            stablecoin_mint,
            levercoin_mint,
            user_stablecoin_ta,
            user_lp_token_ta,
            pool_auth,
            stablecoin_pool,
            levercoin_pool,
            lp_token_auth,
            lp_token_mint,
            sol_usd_pyth_feed,
            system_program,
            token_program,
            associated_token_program,
            event_authority,
            program,
        })
    }
}

impl<'info> Deposit<'info> for HyloStabilityPool {
    type Accounts = HyloStabilityPoolDepositAccounts<'info>;
    type Data = ();

    fn deposit_signed(
        ctx: &HyloStabilityPoolDepositAccounts<'info>,
        amount: u64,
        _data: &Self::Data,
        signer_seeds: &[Signer],
    ) -> ProgramResult {
        let accounts = [
            InstructionAccount::writable_signer(ctx.user.address()),
            InstructionAccount::readonly(ctx.pool_config.address()),
            InstructionAccount::readonly(ctx.hylo.address()),
            InstructionAccount::readonly(ctx.stablecoin_mint.address()),
            InstructionAccount::readonly(ctx.levercoin_mint.address()),
            InstructionAccount::writable(ctx.user_stablecoin_ta.address()),
            InstructionAccount::writable(ctx.user_lp_token_ta.address()),
            InstructionAccount::readonly(ctx.pool_auth.address()),
            InstructionAccount::writable(ctx.stablecoin_pool.address()),
            InstructionAccount::readonly(ctx.levercoin_pool.address()),
            InstructionAccount::readonly(ctx.lp_token_auth.address()),
            InstructionAccount::writable(ctx.lp_token_mint.address()),
            InstructionAccount::readonly(ctx.sol_usd_pyth_feed.address()),
            InstructionAccount::readonly(ctx.system_program.address()),
            InstructionAccount::readonly(ctx.token_program.address()),
            InstructionAccount::readonly(ctx.associated_token_program.address()),
            InstructionAccount::readonly(ctx.event_authority.address()),
            InstructionAccount::readonly(ctx.program.address()),
        ];

        let account_infos = [
            ctx.user,
            ctx.pool_config,
            ctx.hylo,
            ctx.stablecoin_mint,
            ctx.levercoin_mint,
            ctx.user_stablecoin_ta,
            ctx.user_lp_token_ta,
            ctx.pool_auth,
            ctx.stablecoin_pool,
            ctx.levercoin_pool,
            ctx.lp_token_auth,
            ctx.lp_token_mint,
            ctx.sol_usd_pyth_feed,
            ctx.system_program,
            ctx.token_program,
            ctx.associated_token_program,
            ctx.event_authority,
            ctx.program,
        ];

        let mut instruction_data = MaybeUninit::<[u8; 16]>::uninit();
        unsafe {
            let ptr = instruction_data.as_mut_ptr() as *mut u8;
            core::ptr::copy_nonoverlapping(USER_DEPOSIT_DISCRIMINATOR.as_ptr(), ptr, 8);
            core::ptr::copy_nonoverlapping(amount.to_le_bytes().as_ptr(), ptr.add(8), 8);
        }

        let ix = InstructionView {
            program_id: &HYLO_STABILITY_PROGRAM_ID,
            accounts: &accounts,
            data: unsafe { instruction_data.assume_init_ref() },
        };

        invoke_signed(&ix, &account_infos, signer_seeds)
    }

    fn deposit(
        ctx: &HyloStabilityPoolDepositAccounts<'info>,
        amount: u64,
        data: &Self::Data,
    ) -> ProgramResult {
        Self::deposit_signed(ctx, amount, data, &[])
    }
}
