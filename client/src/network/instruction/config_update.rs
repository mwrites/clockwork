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

pub fn config_update(admin: Pubkey, settings: ConfigSettings) -> Instruction {
    Instruction {
        program_id: mat_clockwork_network_program::ID,
        accounts: vec![
            AccountMeta::new(admin, true),
            AccountMeta::new(Config::pubkey(), false),
        ],
        data: mat_clockwork_network_program::instruction::ConfigUpdate { settings }.data(),
    }
}
