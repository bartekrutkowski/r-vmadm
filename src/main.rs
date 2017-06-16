#[macro_use]
extern crate clap;

fn main() {
    use clap::App;
    let yaml = load_yaml!("cli.yml");
    let mut help_app = App::from_yaml(yaml).version(crate_version!());
    let matches = App::from_yaml(yaml).version(crate_version!()).get_matches();
    if matches.is_present("startup") {
        match matches.subcommand() {
            ("", None) =>println!("startup"),
            _ => println!("Can not use startup with a subcommand")
        }

    } else {
        match matches.subcommand() {
            ("list", Some(list_matches)) => println!("list jails"),
            ("create", Some(create_matches)) => println!("create jail"),
            ("update", Some(update_matches)) => println!("update jail"),
            ("destroy", Some(destroy_matches)) => println!("destroy jail"),
            ("start", Some(destroy_matches)) => println!("start jail"),
            ("stop", Some(destroy_matches)) => println!("stop jail"),
            ("", None) => help_app.print_help().unwrap(),
            _ => unreachable!(),
        };
    };
    
}