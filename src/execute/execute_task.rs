use {
    crate::{
        replicate::replicate_task,
        utils::{new_rpc_client, sign_and_submit},
    },
    anchor_lang::prelude::{AccountMeta, Pubkey},
    cronos_sdk::account::*,
    solana_sdk::signature::Signature,
};

#[cached::proc_macro::cached(size = 1_000_000, time = 5, option = true)]
pub fn execute_task(pubkey: Pubkey, daemon: Pubkey) -> Option<Signature> {
    let client = new_rpc_client();
    let data = client.get_account_data(&pubkey).unwrap();
    let task = Task::try_from(data).unwrap();
    match task.status {
        TaskStatus::Cancelled | TaskStatus::Done => {
            replicate_task(pubkey, task);
            return None;
        }
        TaskStatus::Queued => {
            let config = Config::pda().0;
            let fee = Fee::pda(daemon).0;
            let mut ix = cronos_sdk::instruction::task_execute(
                config,
                daemon,
                fee,
                pubkey,
                client.payer_pubkey(),
            );
            for acc in task.ix.accounts {
                match acc.is_writable {
                    true => ix.accounts.push(AccountMeta::new(acc.pubkey, false)),
                    false => ix
                        .accounts
                        .push(AccountMeta::new_readonly(acc.pubkey, false)),
                }
            }
            ix.accounts
                .push(AccountMeta::new_readonly(task.ix.program_id, false));
            Some(sign_and_submit(client, &[ix], "Executing task").unwrap())
        }
    }
}
