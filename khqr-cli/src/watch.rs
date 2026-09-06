use clap::Args as ClapArgs;
use khqr_api::{Backoff, BakongClient, Environment, TxStatus};
use std::error::Error;
use std::time::{Duration, Instant};

#[derive(ClapArgs)]
pub struct Args {
    /// The MD5 handle of the payload, as printed by `khqr gen`.
    #[arg(long)]
    md5: String,
    /// Bakong API token.
    #[arg(long, env = "BAKONG_TOKEN", hide_env_values = true)]
    token: String,
    /// The email the token was registered with, so it can be renewed.
    #[arg(long, env = "BAKONG_EMAIL")]
    email: Option<String>,
    /// Use the sandbox rather than production.
    #[arg(long)]
    sandbox: bool,
    /// Give up after this many seconds.
    #[arg(long, default_value_t = 300)]
    timeout: u64,
}

pub fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(poll(args))
}

async fn poll(args: &Args) -> Result<(), Box<dyn Error>> {
    let environment = if args.sandbox {
        Environment::Sandbox
    } else {
        Environment::Production
    };

    let mut client = BakongClient::new(environment, &args.token);
    if let Some(email) = &args.email {
        client = client.with_renewal_email(email);
    }

    let mut backoff = Backoff::new();
    let deadline = Instant::now() + Duration::from_secs(args.timeout);

    loop {
        match client.check_transaction_by_md5(&args.md5).await? {
            TxStatus::Paid(transaction) => {
                println!("paid {}", transaction.hash);

                if let (Some(amount), Some(currency)) = (transaction.amount, &transaction.currency)
                {
                    println!("amount {amount} {currency}");
                }
                if let Some(from) = &transaction.from_account_id {
                    println!("from {from}");
                }

                return Ok(());
            }
            TxStatus::StaticQr => {
                return Err("static QR codes carry no amount and cannot be tracked".into())
            }
            TxStatus::NotFound => {}
        }

        let delay = backoff.next_delay();
        if Instant::now() + delay >= deadline {
            return Err("timed out waiting for payment".into());
        }

        eprintln!("waiting {}s", delay.as_secs());
        tokio::time::sleep(delay).await;
    }
}
