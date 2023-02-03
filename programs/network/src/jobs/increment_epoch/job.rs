use {crate::state::*, anchor_lang::prelude::*, clockwork_utils::ThreadResponse};

#[derive(Accounts)]
pub struct EpochCutover<'info> {
    #[account(address = Config::pubkey())]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [SEED_REGISTRY],
        bump,
    )]
    pub registry: Account<'info, Registry>,

    #[account(address = config.epoch_thread)]
    pub thread: Signer<'info>,
}

pub fn handler(ctx: Context<EpochCutover>) -> Result<ThreadResponse> {
    let registry = &mut ctx.accounts.registry;
    registry.current_epoch = registry.current_epoch.checked_add(1).unwrap();
    registry.locked = false;
    Ok(ThreadResponse {
        next_instruction: None,
    })
}