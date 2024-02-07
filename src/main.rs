use reqwest::blocking::get;
use serde_derive::{Serialize,Deserialize};
use std::fs;
use std::process::exit;
use scraper;
use url::Url;
use toml;
use std::env;

#[derive(Serialize,Deserialize, Debug)]
struct ConfigData {
    config: Settings,
}
#[derive(Serialize,Deserialize, Debug)]
struct Settings {
    host: String,
	url: String,
	div_id_ultimos: String,
    link_id_download_torrent: String,
    link_text_download_torrent: String,
    output_file: String,
}

fn read_config(filename: &str) -> ConfigData{
    // Read the contents of the file using a `match` block 
    // to return the `data: Ok(c)` as a `String` 
    // or handle any `errors: Err(_)`.
    eprintln!("\nReading config file `{}`\n", filename);

    let contents:String = match fs::read_to_string(filename) {
        // If successful return the files text as `contents`.
        // `c` is a local variable.
        Ok(c) => c,
        // Handle the `error` case.
        Err(_) => {
            // Write `msg` to `stderr`.
            eprintln!("[!] Could not read config file `{}`", filename);
            // Exit the program with exit code `1`.
            exit(1);
        }
    };

    let replaced =  contents.clone()
                                    .replace("=", "=\"")
                                    .replace("\n", "\"\n")
                                    .replace("]\"\n","]\n")
                                    .replace("\"\"","\"");

    // Use a `match` block to return the 
    // file `contents` as a `Data struct: Ok(d)`
    // or handle any `errors: Err(_)`.
    let configdata: crate::ConfigData = match toml::from_str(&replaced) {
        // If successful, return data as `Data` struct.
        // `d` is a local variable.
        Ok(d) => d,
        // Handle the `error` case.
        Err(e) => {
            // Write `msg` to `stderr`.
            eprintln!("\n[!] Unable to load config data from `{}` \nError:[{}]", filename,e.message());
            println!("\n{}",&replaced.as_str());
            // Exit the program with exit code `1`.

            std::process::exit(1);
        }
    };
    return configdata;
}

fn write_config(filename: &str,configdata: &ConfigData){
    let toml_string = toml::to_string(configdata).expect("Could not encode TOML value").replace("\"", "");
    fs::write(filename, toml_string).expect("Could not write to file!");
}

fn get_last_don_torrent_domain(telegram_url: &str) -> String {
    println!("Reading:'{}'", telegram_url);
    let response = reqwest::blocking::get(telegram_url)
        .unwrap()
        .text()
        .unwrap();

    let document = scraper::Html::parse_document(&response);
    
    //println!("{:#?}",document);

    let selector = scraper::Selector::parse("div.tgme_widget_message_text>a").unwrap();
    let links = document.select(&selector).map(|x| x.inner_html());
    //println!("{:#?}",links.last());
    
    // let link =  match links.last(){
    //     Some(l) => l,
    //     _ => String::from(""),
    // };
    // match url::Url::parse(link.as_str()){
    //     Result::Ok(s) => s.as_str(),
    //     Result::Err(_e ) => "",
    // }
    let link =  match links.last(){
        Some(l) => 
            match url::Url::parse(&l.as_str()){
                Result::Ok(s) => String::from(s.as_str()),
                Result::Err(_e ) => String::from(""),
            },
        _ => String::from("")
    };

    let url = link.clone();

    return url;
    //println!("{}",link);

}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut option: &str = "";
    let mut parameter: &str  = "";
    let config_option: &str = "-c";
    let filename: &str;
    
    if args.len()>1{
        option = &args[1];
    };
    if args.len()>2{
        parameter = &args[2];
    } ;
    if args.len()>2 && option.eq(config_option){
        filename = parameter;
    } else {
        filename = "createDonTorrentRSS.conf";
    };
    let configdata: ConfigData = read_config(filename);

    let previous_domain = configdata.config.host.clone();
    // Print out the values to `stdout`.
    println!("host:                       {}", configdata.config.host); 
    println!("url:                        {}", configdata.config.url);
    println!("div_id_ultimos:             {}", configdata.config.div_id_ultimos);
    println!("link_id_download_torrent:   {}", configdata.config.link_id_download_torrent);    
    println!("link_text_download_torrent: {}", configdata.config.link_text_download_torrent);
    println!("output_file:                {}", configdata.config.output_file);

    let last_domain = get_last_don_torrent_domain("https://t.me/s/DonTorrent");
    
    println!("Previous domain:'{}'", previous_domain);
    println!("Last domain:'{}'", last_domain);
    
    if previous_domain.ne(&last_domain){
        let newsettings: Settings = Settings{
            host:last_domain,
            url:configdata.config.url,
            div_id_ultimos: configdata.config.div_id_ultimos,
            link_id_download_torrent:configdata.config.link_id_download_torrent,
            link_text_download_torrent:configdata.config.link_text_download_torrent,
            output_file:configdata.config.output_file,
        };

        let newconfigdata: ConfigData = ConfigData{
            config: newsettings,
        };
        write_config(filename,&newconfigdata);
    }

}
