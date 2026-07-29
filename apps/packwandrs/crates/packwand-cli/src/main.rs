#![forbid(unsafe_code)]

mod api_cmd;
mod build_cmd;
mod cli;
mod dispatch;
mod pages;
mod script_cmd;
mod serve;
mod test_cmd;

fn main() {
    if let Err(error) = dispatch::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
