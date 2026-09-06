use clap::{Args as ClapArgs, ValueEnum};
use khqr_core::{
    md5, to_png_with, to_svg_with, Currency, ErrorCorrection, ImageOptions, Khqr, KhqrBuilder,
};
use std::error::Error;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum EccArg {
    Low,
    Medium,
    Quartile,
    High,
}

impl From<EccArg> for ErrorCorrection {
    fn from(level: EccArg) -> Self {
        match level {
            EccArg::Low => Self::Low,
            EccArg::Medium => Self::Medium,
            EccArg::Quartile => Self::Quartile,
            EccArg::High => Self::High,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum CurrencyArg {
    Khr,
    Usd,
}

impl From<CurrencyArg> for Currency {
    fn from(currency: CurrencyArg) -> Self {
        match currency {
            CurrencyArg::Khr => Self::Khr,
            CurrencyArg::Usd => Self::Usd,
        }
    }
}

#[derive(ClapArgs)]
pub struct Args {
    /// Bakong account, as name@bank.
    #[arg(long)]
    account: String,
    /// Merchant name, at most 25 characters.
    #[arg(long)]
    name: String,
    /// Merchant city, at most 15 characters.
    #[arg(long)]
    city: String,
    /// Amount. Leaving it out makes a reusable static QR.
    #[arg(long)]
    amount: Option<f64>,
    #[arg(long, value_enum, default_value_t = CurrencyArg::Khr)]
    currency: CurrencyArg,
    /// Write to tag 30 as a business rather than tag 29 as a person.
    #[arg(long)]
    merchant: bool,
    #[arg(long)]
    merchant_id: Option<String>,
    #[arg(long)]
    acquiring_bank: Option<String>,
    #[arg(long)]
    bill_number: Option<String>,
    #[arg(long)]
    mobile_number: Option<String>,
    #[arg(long)]
    store_label: Option<String>,
    #[arg(long)]
    terminal_label: Option<String>,
    /// Merchant category code.
    #[arg(long)]
    mcc: Option<String>,
    /// Seconds until the QR expires.
    #[arg(long)]
    expires_in: Option<u64>,
    /// Write a PNG here.
    #[arg(long)]
    png: Option<PathBuf>,
    /// Write an SVG here.
    #[arg(long)]
    svg: Option<PathBuf>,
    /// Width of the PNG in pixels.
    #[arg(long, default_value_t = 512)]
    size: u32,
    /// How much of the code can be covered and still read. Use high if you
    /// intend to draw a logo over the middle.
    #[arg(long, value_enum, default_value_t = EccArg::Medium)]
    ecc: EccArg,
    /// Blank modules around the code. Set 0 when your own layout pads it.
    #[arg(long, default_value_t = 4)]
    quiet_zone: u32,
}

pub fn run(args: &Args) -> Result<(), Box<dyn Error>> {
    let created_at = now_ms()?;
    let mut builder = if args.merchant {
        Khqr::merchant(&args.account)
    } else {
        Khqr::individual(&args.account)
    };

    builder = builder
        .merchant_name(&args.name)
        .merchant_city(&args.city)
        .currency(args.currency.into())
        .created_at_ms(created_at);

    if let Some(amount) = args.amount {
        builder = builder.amount(amount);
    }
    if let Some(seconds) = args.expires_in {
        let expires_at = seconds
            .checked_mul(1_000)
            .and_then(|millis| created_at.checked_add(millis))
            .ok_or("expires-in is too far in the future")?;

        builder = builder.expires_at_ms(expires_at);
    }

    builder = optional(builder, args);

    let qr = builder.build()?.to_qr_string()?;
    println!("{qr}");
    println!("md5 {}", md5(&qr));

    let options = ImageOptions {
        size: args.size,
        error_correction: args.ecc.into(),
        quiet_zone: args.quiet_zone,
    };

    if let Some(path) = &args.png {
        std::fs::write(path, to_png_with(&qr, &options)?)?;
        println!("png {}", path.display());
    }
    if let Some(path) = &args.svg {
        std::fs::write(path, to_svg_with(&qr, &options)?)?;
        println!("svg {}", path.display());
    }

    Ok(())
}

fn optional(mut builder: KhqrBuilder, args: &Args) -> KhqrBuilder {
    if let Some(value) = &args.merchant_id {
        builder = builder.merchant_id(value);
    }
    if let Some(value) = &args.acquiring_bank {
        builder = builder.acquiring_bank(value);
    }
    if let Some(value) = &args.bill_number {
        builder = builder.bill_number(value);
    }
    if let Some(value) = &args.mobile_number {
        builder = builder.mobile_number(value);
    }
    if let Some(value) = &args.store_label {
        builder = builder.store_label(value);
    }
    if let Some(value) = &args.terminal_label {
        builder = builder.terminal_label(value);
    }
    if let Some(value) = &args.mcc {
        builder = builder.merchant_category_code(value);
    }

    builder
}

fn now_ms() -> Result<u64, Box<dyn Error>> {
    let millis = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

    Ok(u64::try_from(millis)?)
}
