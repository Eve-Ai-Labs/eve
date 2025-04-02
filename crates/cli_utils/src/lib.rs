use clap::Parser;

#[derive(Debug, Parser)]
pub struct Prompt {
    /// always yes
    #[arg(short, long)]
    pub yes: bool,

    #[arg(long, hide = true)]
    pub no_display: bool,
}

impl Prompt {
    pub fn prompt_yes<P: std::fmt::Display>(&self, prompt: P) -> bool {
        if self.no_display {
            return true;
        }
        let mut result: Result<bool, ()> = Err(());

        // Read input until a yes or a no is given
        while result.is_err() {
            println!();
            println!("{prompt}");
            println!("[Y]es/[N]o",);

            if self.yes {
                println!("yes");
                println!();
                return true;
            }

            let mut input = String::new();
            if std::io::stdin().read_line(&mut input).is_err() {
                continue;
            }
            result = match input.trim().to_lowercase().as_str() {
                "yes" | "y" => Ok(true),
                "no" | "n" => {
                    println!();
                    println!("Cancelled");
                    Ok(false)
                }
                _ => Err(()),
            };
        }

        println!();

        result.unwrap()
    }
}
