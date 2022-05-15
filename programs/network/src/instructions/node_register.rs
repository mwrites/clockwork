use {
    crate::state::*,
    anchor_lang::{
        prelude::*,
        solana_program::{instruction::Instruction, system_program, sysvar},
    },
    anchor_spl::{
        associated_token::AssociatedToken,
        token::{Mint, Token, TokenAccount},
    },
    cronos_scheduler::program::CronosScheduler,
    std::mem::size_of,
};

#[derive(Accounts)]
pub struct NodeRegister<'info> {
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(seeds = [SEED_AUTHORITY], bump)]
    pub authority: Box<Account<'info, Authority>>,

    #[account(seeds = [SEED_CONFIG], bump)]
    pub config: Box<Account<'info, Config>>,

    #[account()]
    pub delegate: Signer<'info>,

    #[account(
        init,
        seeds = [
            SEED_SNAPSHOT_ENTRY,
            snapshot.key().as_ref(),
            snapshot.entry_count.to_be_bytes().as_ref(),
        ],
        bump,
        payer = owner,
        space = 8 + size_of::<SnapshotEntry>(),
    )]
    pub entry: Account<'info, SnapshotEntry>,

    #[account(address = config.mint)]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        seeds = [
            SEED_NODE,
            delegate.key().as_ref()
        ],
        bump,
        payer = owner,
        space = 8 + size_of::<Node>(),
    )]
    pub node: Account<'info, Node>,

    #[account(mut, constraint = owner.key() != delegate.key())]
    pub owner: Signer<'info>,

    #[account(
        mut, 
        seeds = [SEED_REGISTRY], 
        bump,
        constraint = !registry.is_locked
    )]
    pub registry: Account<'info, Registry>,

    #[account(address = sysvar::rent::ID)]
    pub rent: Sysvar<'info, Rent>,

    #[account(address = cronos_scheduler::ID)]
    pub scheduler_program: Program<'info, CronosScheduler>,

    #[account(
        mut,
        seeds = [
            SEED_SNAPSHOT,
            snapshot.id.to_be_bytes().as_ref(),
        ],
        bump,
        constraint = snapshot.status == SnapshotStatus::Current
    )]
    pub snapshot: Account<'info, Snapshot>,

    #[account(
        init,
        payer = owner,
        associated_token::authority = node,
        associated_token::mint = mint,
    )]
    pub stake: Account<'info, TokenAccount>,

    #[account(address = system_program::ID)]
    pub system_program: Program<'info, System>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, NodeRegister<'info>>) -> Result<()> {
    // Get accounts
    let authority = &ctx.accounts.authority;
    let config = &ctx.accounts.config;
    let delegate = &ctx.accounts.delegate;
    let entry = &mut ctx.accounts.entry;
    let node = &mut ctx.accounts.node;
    let owner = &mut ctx.accounts.owner;
    let registry = &mut ctx.accounts.registry;
    let scheduler_program = &ctx.accounts.scheduler_program;
    let snapshot = &mut ctx.accounts.snapshot;
    let system_program = &ctx.accounts.system_program;
    let stake = &mut ctx.accounts.stake;

    // Get remaining accounts
    let cycler_action = ctx.remaining_accounts.get(0).unwrap();
    let cycler_task = ctx.remaining_accounts.get(1).unwrap();
    let queue = ctx.remaining_accounts.get(2).unwrap();
    let snapshot_action = ctx.remaining_accounts.get(3).unwrap();
    let snapshot_task = ctx.remaining_accounts.get(4).unwrap();

    // Get bumps
    let authority_bump = *ctx.bumps.get("authority").unwrap();

    // Add node to the registry
    registry.new_node(delegate, owner, node, stake)?;

    // Add an empty entry to the current snapshot
    snapshot.capture(entry, node, stake)?;

    // Add an action to the cycler task to check the snapshot entry for this node
    let cycler_run_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new(Cycler::pda().0, false),
            AccountMeta::new_readonly(entry.key(), false),
            AccountMeta::new(cronos_pool::state::Pool::pda().0, false),
            AccountMeta::new_readonly(cronos_pool::state::Config::pda().0, false),
            AccountMeta::new_readonly(cronos_pool::ID, false),
            AccountMeta::new_readonly(queue.key(), true),
            AccountMeta::new_readonly(registry.key(), false),
            AccountMeta::new_readonly(snapshot.key(), false),
        ],
        data: sighash("global", "cycler_run").into(),
    };
    cronos_scheduler::cpi::action_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::ActionNew {
                action: cycler_action.to_account_info(),
                owner: authority.to_account_info(),
                payer: owner.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: cycler_task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[authority_bump]]],
        ),
        vec![cycler_run_ix.into()],
    )?;

    // Add an action to the snapshot task to capture an entry for this node
    let current_snapshot_pubkey = Snapshot::pda(registry.snapshot_count.checked_sub(1).unwrap()).0;
    let next_snapshot_pubkey = Snapshot::pda(registry.snapshot_count).0;
    let next_entry_pubkey = SnapshotEntry::pda(next_snapshot_pubkey, node.id).0;
    let stake_pubkey = stake.key();
    let snapshot_capture_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(config.key(), false),
            AccountMeta::new(next_entry_pubkey, false),
            AccountMeta::new_readonly(node.key(), false,),
            AccountMeta::new(cronos_scheduler::delegate::ID, true),
            AccountMeta::new_readonly(queue.key(), true),
            AccountMeta::new_readonly(registry.key(), false),
            AccountMeta::new(next_snapshot_pubkey, false),
            AccountMeta::new_readonly(stake_pubkey, false),
            AccountMeta::new_readonly(system_program.key(), false)
        ],
        data: sighash("global", "snapshot_capture").into(),
    };
    let snapshot_rotate_ix = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new_readonly(authority.key(), false),
            AccountMeta::new_readonly(sysvar::clock::ID, false),
            AccountMeta::new_readonly(config.key(), false),
            AccountMeta::new(current_snapshot_pubkey, false),
            AccountMeta::new(next_snapshot_pubkey, false),
            AccountMeta::new_readonly(queue.key(), true),
            AccountMeta::new(registry.key(), false),
        ],
        data: sighash("global", "snapshot_rotate").into(),
    };
    cronos_scheduler::cpi::action_new(
        CpiContext::new_with_signer(
            scheduler_program.to_account_info(),
            cronos_scheduler::cpi::accounts::ActionNew {
                action: snapshot_action.to_account_info(),
                owner: authority.to_account_info(),
                payer: owner.to_account_info(),
                queue: queue.to_account_info(),
                system_program: system_program.to_account_info(),
                task: snapshot_task.to_account_info(),
            },
            &[&[SEED_AUTHORITY, &[authority_bump]]],
        ),
        vec![snapshot_capture_ix.into(), snapshot_rotate_ix.into()],
    )?;

    Ok(())
}

fn sighash(namespace: &str, name: &str) -> [u8; 8] {
    let preimage = format!("{}:{}", namespace, name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(
        &anchor_lang::solana_program::hash::hash(preimage.as_bytes()).to_bytes()
            [..8],
    );
    sighash
}