use std::env;

mod app;
mod domain;
mod render;
mod terminal;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("--probe") => {
            terminal::metrics::print_probe(terminal::metrics::probe_terminal());
            Ok(())
        }
        Some("--dump-layout") => {
            terminal::layout::print_layout_fixture();
            Ok(())
        }
        Some("--kitty-demo") => terminal::kitty::run_kitty_demo(),
        Some("--svg-demo") => render::run_svg_demo(),
        Some("--splash-demo") => render::run_splash_demo(),
        Some("--ascii-splash-demo") => terminal::video::run_ascii_splash_demo(),
        Some("--rgb-splash-demo") => match terminal::video::run_rgb_splash_demo() {
            Ok(terminal::video::VideoExit::Start | terminal::video::VideoExit::Finished) => {
                app::run_mvp_svg_loop()
            }
            Ok(terminal::video::VideoExit::Quit) => Ok(()),
            Err(error) => Err(error),
        },
        Some("--watch-metrics") => terminal::metrics::run_watch_metrics(),
        Some("--domain-demo") => {
            domain::print_domain_demo();
            Ok(())
        }
        Some("--mvp-loop") => app::run_mvp_loop(),
        Some("dev") | Some("--mvp-svg-loop") => app::run_mvp_svg_loop(),
        None => app::run_mvp_svg_loop(),
        Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some(arg) => Err(format!("unknown argument: {arg}")),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(2);
    }
}

fn print_help() {
    println!("Project-Y MVP tools");
    println!();
    println!("USAGE:");
    println!("  furina-advocate-sim");
    println!("  furina-advocate-sim dev");
    println!("  furina-advocate-sim --probe");
    println!("  furina-advocate-sim --dump-layout");
    println!("  furina-advocate-sim --kitty-demo");
    println!("  furina-advocate-sim --svg-demo");
    println!("  furina-advocate-sim --splash-demo");
    println!("  furina-advocate-sim --ascii-splash-demo");
    println!("  furina-advocate-sim --rgb-splash-demo");
    println!("  furina-advocate-sim --watch-metrics");
    println!("  furina-advocate-sim --domain-demo");
    println!("  furina-advocate-sim --mvp-loop");
    println!("  furina-advocate-sim --mvp-svg-loop");
}
