use bcrypt::{DEFAULT_COST, hash};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = password_arg()?;
    let password_hash = hash(password, DEFAULT_COST)?;
    println!("{password_hash}");
    Ok(())
}

fn password_arg() -> Result<String, String> {
    let mut args = std::env::args().skip(1);
    let Some(password) = args.next() else {
        return Err("usage: cargo run -p user --bin generate_password_hash -- <password>".into());
    };
    if args.next().is_some() {
        return Err("usage: cargo run -p user --bin generate_password_hash -- <password>".into());
    }
    Ok(password)
}
