use clap::{Parser, ArgGroup};

mod sxp;
use crate::sxp::sxp::{SxpFrame, Smp};

#[derive(Debug, Parser)]
#[clap(author = "qdu", version = "1.0.0", about, long_about = None)]
#[clap(group(ArgGroup::new("input").required(true).args(&["data", "file"])),)]
struct Args{
    /// parse data
    #[clap(short, long)]
    data: Option<String>,

    #[clap(short, long)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();
    

    let mut sxp = SxpFrame::default();
    let parse_ret;
    let mut error_info = String::new();

    if let Some(data) = args.data {
        parse_ret = sxp.load_data(data.as_str());
        error_info.push_str(format!("data=\"{}\"", &data).as_str());
    } else if let Some(file) = args.file {
        parse_ret = sxp.load_file(file.as_str());
        error_info.push_str(format!("file=\"{}\"", &file).as_str());
    } else {
        parse_ret = Err(anyhow::format_err!("No Input"));
    }

    match parse_ret {
        Ok(_) => {
            match sxp.parse_smp() {
                Ok(_) => ok!("parse success"),
                Err(e) => error!("error: {:?}", e),
            }
        },
        Err(e) => {
            error!("load error: {:?}, {}", e, &error_info);
        },
    }
}
