use {
    anchor_lang::{
        solana_program::{
            instruction::{AccountMeta, Instruction},
            pubkey::Pubkey,
        },
        InstructionData,
    },
    mat_clockwork_network_program::state::*,
};

pub fn delete_snapshot(thread: Pubkey) -> Instruction {
    Instruction {
        program_id: mat_clockwork_network_program::ID,
        accounts: vec![
            AccountMeta::new_readonly(Config::pubkey(), false),
            AccountMeta::new(Registry::pubkey(), false),
            AccountMeta::new_readonly(thread, true),
        ],
        data: mat_clockwork_network_program::instruction::DeleteSnapshotJob {}.data(),
    }
}
