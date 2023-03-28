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

pub fn registry_unlock(admin: Pubkey) -> Instruction {
    Instruction {
        program_id: mat_clockwork_network_program::ID,
        accounts: vec![
            AccountMeta::new(admin, true),
            AccountMeta::new_readonly(Config::pubkey(), false),
            AccountMeta::new(Registry::pubkey(), false),
        ],
        data: mat_clockwork_network_program::instruction::RegistryUnlock {}.data(),
    }
}
