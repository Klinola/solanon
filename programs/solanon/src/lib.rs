#![allow(unexpected_cfgs)]
use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_lang::solana_program::{
    program::invoke_signed,
    system_instruction,
    sysvar::rent::Rent,
};
pub mod program_id;

declare_id!("EYutos9XnBjwzGUs1rz1s29f1AUwcEDWq3eaJm43pEex");

#[program]
pub mod solanon {


    use super::*;

    /// The `mix` instruction: Users pass a nonce and a vector of output details.
    /// The remaining accounts should be passed in the following order:
    /// - The first N accounts are the intermediate accounts (PDAs generated with seed: ["intermediate", user, nonce, index]).
    /// - The next N accounts are the final output accounts.
    pub fn mix<'a, 'b, 'c, 'info>(ctx: Context<'a, 'b, 'c,'info, Mix<'info>>, nonce: u64, outputs: Vec<OutputDetail>) -> Result<()> {
        let user = &ctx.accounts.user;
        let system_program = &ctx.accounts.system_program;
        let remaining_accounts = &ctx.remaining_accounts;
        let count = outputs.len();

        // Validate that the remaining accounts length equals 2 * count (intermediate accounts and output accounts)
        if remaining_accounts.len() != count * 2 {
            return Err(ErrorCode::InvalidRemainingAccounts.into());
        }

        // Loop through each output detail, processing each pair of intermediate and output accounts
        for (i, output) in outputs.iter().enumerate() {
            // Construct PDA seeds: fixed prefix, user address, nonce, and index
            let seed_index = i as u64;
            // Construct seed buffers
            let nonce_bytes = nonce.to_le_bytes();
            let index_bytes = seed_index.to_le_bytes();
            msg!("Seed for index {}: intermediate, user: {}, nonce: {:?}, index: {:?}", 
                i, user.key, nonce_bytes, index_bytes);
            let seeds: &[&[u8]] = &[
                b"intermediate",
                user.key.as_ref(),
                &nonce_bytes,
                &index_bytes,
            ];
            // Derive PDA and bump based on seeds
            let (pda, bump) = Pubkey::find_program_address(seeds, ctx.program_id);
            msg!("Derived PDA for index {}: {} with bump: {}", i, pda, bump);
            // Get the corresponding intermediate account (first group)
            let intermed_ai = &remaining_accounts[i];

            // Validate that the provided intermediate account matches the derived PDA
            if intermed_ai.key != &pda {
                return Err(ErrorCode::InvalidIntermediateAccount.into());
            }

            // If intermediate account is not yet initialized (e.g., lamports == 0), create it.
            // The account will be initialized with rent-exempt minimum balance + output.amount.
            if intermed_ai.lamports() == 0 {
                let space: u64 = 8; // minimal space
                let rent = Rent::get()?;
                let rent_exempt = rent.minimum_balance(space as usize);
                let lamports_required = rent_exempt
                    .checked_add(output.amount)
                    .ok_or(ErrorCode::MathError)?;
                msg!("Creating intermediate account {} with lamports: {}", pda, lamports_required);
                let create_ix = system_instruction::create_account(
                    &user.key(),
                    &pda,
                    lamports_required,
                    space,
                    ctx.program_id,
                );
                // Invoke system program to create the account, using user as payer.
                invoke_signed(
                    &create_ix,
                    &[
                        user.to_account_info(),
                        intermed_ai.clone(),
                        system_program.to_account_info(),
                    ],
                    // signer_seeds for the PDA
                    &[&[
                        b"intermediate",
                        user.key.as_ref(),
                        &nonce_bytes,
                        &index_bytes,
                        &[bump],
                    ]],
                )?;
                
               
            } else {
                // If the account is already initialized, you may add additional checks
                // to prevent accumulating funds from multiple calls if needed.
            }

            // Get the corresponding final output account (second group)
            let output_ai = &remaining_accounts[count + i];
            msg!("1 Create a transfer instruction to move funds from the intermediate account to the output account.");
            {
                // 获取中间账户和目标账户的 AccountInfo
                let vault_account_info = intermed_ai.clone(); // PDA 对应的账户：我们的中间账户
                let owner_account_info = output_ai.clone();   // 目标输出账户
            
                {
                    // 从账户中获取 mutable 借用
                    let mut vault_lamports = vault_account_info.lamports.borrow_mut();
                    let mut owner_lamports = owner_account_info.lamports.borrow_mut();
                
                    // 取出当前余额，解引用得到 u64 类型
                    let current_vault: u64 = **vault_lamports;
                    let current_owner: u64 = **owner_lamports;
                
                    let transfer_amount = output.amount;
                
                    // 检查中间账户余额是否足够
                    if current_vault < transfer_amount {
                        return Err(ErrorCode::InsufficientFunds.into());
                    }
                
                    // 将计算后的结果重新写回 mutable 借用中
                    **vault_lamports = current_vault
                    .checked_sub(transfer_amount)
                    .ok_or(ErrorCode::MathError)?;
                    **owner_lamports = current_owner
                    .checked_add(transfer_amount)
                    .ok_or(ErrorCode::MathError)?;
                
                    msg!("Transferred {} lamports from {} to {}",
                         transfer_amount,
                         vault_account_info.key,
                         owner_account_info.key,
                    );
                }
                
                
            }
            
            msg!("3 Create a transfer instruction to move funds from the intermediate account to the output account.");
        }

        Ok(())
    }
}

/// OutputDetail defines the structure for each output detail: target output account and the amount to transfer.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct OutputDetail {
    pub address: Pubkey,
    pub amount: u64,
}

/// Define the accounts structure required by the `mix` instruction.
#[derive(Accounts)]
pub struct Mix<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// The system program for account creation and transfers.
    pub system_program: Program<'info, System>,
    // Remaining accounts are passed in via ctx.remaining_accounts:
    // The first N accounts are the intermediate accounts (PDAs),
    // and the following N accounts are the final output accounts.
}

/// Custom error codes for the solanon program.
#[error_code]
pub enum ErrorCode {
    #[msg("The number of provided intermediate accounts does not match the expected count (should be 2x the output count).")]
    InvalidRemainingAccounts,
    #[msg("The provided intermediate account does not match the expected PDA.")]
    InvalidIntermediateAccount,
    #[msg("Math calculation error.")]
    MathError,
    #[msg("Insufficient funds in the intermediate account.")]
    InsufficientFunds,
}
