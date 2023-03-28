use anchor_lang::{
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    },
    InstructionData,
};
use mat_clockwork_network_program::state::*;

pub fn worker_update(authority: Pubkey, settings: WorkerSettings, worker: Pubkey) -> Instruction {
    Instruction {
        program_id: mat_clockwork_network_program::ID,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new(worker, false),
        ],
        data: mat_clockwork_network_program::instruction::WorkerUpdate { settings }.data(),
    }
}
