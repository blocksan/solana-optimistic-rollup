use std::str::from_utf8;

use solana_program::{
    account_info:: AccountInfo,
    entrypoint,
    entrypoint::ProgramResult,
    msg, 
    pubkey:: Pubkey
};

entrypoint!(program_instructions);

fn program_instructions(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8]
) -> ProgramResult{
    let inst_string = from_utf8(instruction_data).unwrap();
    msg!("block hash: {}", inst_string.to_string());

    Ok(())
}

