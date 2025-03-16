use reqwest;
use serde_derive::{Serialize,Deserialize};
use std::fmt::Debug;
use std::fs;
use std::io::Write;
use std::process::exit;
use scraper;
use url;
use toml;
use std::env;
use regex;
use chrono::prelude::*;
// use std::fs::File;
// use std::io::prelude::*;


#[derive(Serialize,Deserialize, Debug)]
struct ConfigData {
    config: Settings,
}
#[derive(Serialize,Deserialize, Debug)]
struct Settings {
    telegram_url: String,
	website_url: String,
    website_path: String,
    div_id_ultimos: String,
    link_id_download_torrent: String,
    link_text_download_torrent: String,
    output_file: String,
}

#[derive(Serialize,Deserialize, Debug)]
struct RSSRoot {
    version:String,
    channels: Vec<RSSChannel>,
}
    impl RSSRoot{
        fn write_to_file(&self,file_name: &str){
            println!("\nWriting to: ´{}´",file_name);
            // Create a new file for writing
            let mut rss_file = std::fs::File::create(file_name).expect("rss file could not be created");

            // Write rss root
            rss_file.write(format!("<rss version=\"{}\">\n",&self.version).as_bytes()).expect("rss file write failed");

            // Write rss channel
            for channel in &self.channels{
                rss_file.write(b"<channel>\n").expect("rss file write failed");
                rss_file.write(format!("<title>{}</title>\n",channel.title).as_bytes()).expect("rss file write failed");
                rss_file.write(format!("<link>{}</link>\n",channel.link).as_bytes()).expect("rss file write failed");
                rss_file.write(format!("<description>{}</description>\n",channel.description).as_bytes()).expect("rss file write failed");
                rss_file.write(format!("<lastBuildDate>{}</lastBuildDate>\n",channel.last_build_date).as_bytes()).expect("rss file write failed");

                // Write rss channel items
                for item in &channel.items{
                    rss_file.write(b"<item>\n").expect("rss file write failed");
                    rss_file.write(format!("<title>{}</title>\n",item.title).as_bytes()).expect("rss file write failed");
                    rss_file.write(format!("<category>{}</category>\n",item.category).as_bytes()).expect("rss file write failed");
                    rss_file.write(format!("<season>{}</season>\n",item.season).as_bytes()).expect("rss file write failed");
                    rss_file.write(format!("<episode>{}</episode>\n",item.episode).as_bytes()).expect("rss file write failed");
                    rss_file.write(format!("<link>{}</link>\n",item.link).as_bytes()).expect("rss file write failed");
                    rss_file.write(format!("<quality>{}</quality>\n",item.quality).as_bytes()).expect("rss file write failed");
                    rss_file.write(format!("<pubDate>{}</pubDate>\n",item.pub_date).as_bytes()).expect("rss file write failed");
                    rss_file.write(format!("<enclosure url=\"{}\" length=\"{}\" type=\"{}\"/>\n",item.enclosure_url,item.enclosure_length,item.enclosure_type).as_bytes()).expect("rss file write failed");
                    rss_file.write(b"</item>\n").expect("rss file write failed");
                }
                // Write rss channel closure
                rss_file.write(b"</channel>\n").expect("rss file write failed");

            }
            // Write rss root closure
            rss_file.write(b"</rss>\n").expect("rss file write failed");
            rss_file.flush().expect("rss file flush failed");
        }
    }


#[derive(Serialize,Deserialize, Debug)]
struct RSSChannel {
    title: String,
	link: String,
    description: String,
    last_build_date: String,
    items: Vec<RSSItem>,
}

#[derive(Serialize,Deserialize, Debug)]
struct RSSItem {
    title: String,
	category: String,
    season: String,
    episode: String,
    link: String,
    quality: String,
    pub_date: String,
    enclosure_url: String,
    enclosure_length: i32,
    enclosure_type: String,
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
            eprintln!("\n[!] Could not read config file `{}`", filename);
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
    let toml_string = toml::to_string(configdata).expect("\n[!] Could not encode TOML value")
        .replace("\"", "")
        .replace(" ", "");
    fs::write(filename, toml_string).expect("\n[!] Could not write config to file!");
}

fn get_last_dontorrent_domain(telegram_url: &str) -> String {
    // Ref: https://www.scrapingbee.com/blog/web-scraping-rust/
    println!("\nReading telegram channel:'{}'", telegram_url);
    let response = reqwest::blocking::get(telegram_url)
        .unwrap()
        .text()
        .unwrap();

    let document = scraper::Html::parse_document(&response);

    let selector = scraper::Selector::parse("div.tgme_widget_message_text>a").unwrap();
    let links = document.select(&selector).map(|x| x.inner_html());

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

}

fn get_substring_between(original_string: &String,start_crop_delimiter: &str, end_crop_delimiter: &str) -> String {
    //<a class="text-primary" href="documental/4263/4264/Frmula-1-La-emocin-de-un-Grand-Prix">Fórmula 1: La emoción de un Grand Prix: 2x09 &amp; 2x10</a>
    // let start_crop_delimiter="href=\"";
    // let end_crop_delimiter="\"";
    let start_crop_position: usize;
    let end_crop_position: usize;

    if original_string.find(start_crop_delimiter).is_some() {
        start_crop_position = original_string.find(start_crop_delimiter).unwrap() + start_crop_delimiter.len();
    } else{
        start_crop_position = 0;
    };

    let rest_of_original_string = String::from(&original_string.clone()[start_crop_position..]);

    if rest_of_original_string.find(end_crop_delimiter).is_some() {
        end_crop_position = rest_of_original_string.find(end_crop_delimiter).unwrap();
    }else{
        end_crop_position = 0;
    };

    if end_crop_position>=1{
        return String::from(&rest_of_original_string[..end_crop_position]);
    }else{
        return String::from("");
    };
}

fn get_href_path(html_a_element: &String) -> String {
    //<a class="text-primary" href="documental/4263/4264/Frmula-1-La-emocin-de-un-Grand-Prix">Fórmula 1: La emoción de un Grand Prix: 2x09 &amp; 2x10</a>
    let start_delimiter="href=\"";
    let end_delimiter="\"";
    let start_crop_position: usize;
    let end_crop_position: usize;

    if html_a_element.find(start_delimiter).is_some() {
        start_crop_position = html_a_element.find(&start_delimiter).unwrap() + start_delimiter.len();
    } else{
        start_crop_position = 0;
    };

    let rest_of_html_a_element = String::from(&html_a_element.clone()[start_crop_position..]);

    if rest_of_html_a_element.find(end_delimiter).is_some() {
        end_crop_position = rest_of_html_a_element.find(&end_delimiter).unwrap();
    }else{
        end_crop_position = rest_of_html_a_element.len()+1;
    };

    if end_crop_position>=1{
        return String::from(&rest_of_html_a_element[..end_crop_position]);
    }else{
        return String::from("");
    };
}

fn get_title(html_a_element: &String) -> String {
    //<a class="text-primary" href="documental/4263/4264/Frmula-1-La-emocin-de-un-Grand-Prix">Fórmula 1: La emoción de un Grand Prix: 2x09 &amp; 2x10</a>
    let start_delimiter="\">";
    let end_delimiter="</a>";
    let start_crop_position: usize;
    let end_crop_position: usize;

    if html_a_element.find(start_delimiter).is_some() {
        start_crop_position = html_a_element.find(&start_delimiter).unwrap() + start_delimiter.len();

    } else{
        start_crop_position = 0;
    };

    if html_a_element.find(end_delimiter).is_some() {
        end_crop_position = html_a_element.find(&end_delimiter).unwrap();
    }else{
        end_crop_position = 0;
    };

    if start_crop_position>=1 && end_crop_position>=1 && start_crop_position<end_crop_position{
        return String::from(&html_a_element[start_crop_position..end_crop_position]);
    }else{
        return String::from("");
    };
}

fn get_cathegory(href_path: &String) -> String {
    //<a class="text-primary" href="documental/4263/4264/Frmula-1-La-emocin-de-un-Grand-Prix">Fórmula 1: La emoción de un Grand Prix: 2x09 &amp; 2x10</a>
    let start_delimiter="/";
    let end_delimiter="/";
    let start_crop_position: usize;
    let end_crop_position: usize;
    let cathegory: String;

    if href_path.starts_with(start_delimiter){
        cathegory = String::from(&href_path.clone()[1..]);
        start_crop_position = 0;
    } else {
        cathegory = String::from(&href_path.clone());
        start_crop_position = 0;
    }

    let rest_of_cathegory = String::from(&cathegory.clone()[start_crop_position..]);

    if rest_of_cathegory.find(end_delimiter).is_some() {
        end_crop_position = rest_of_cathegory.find(&end_delimiter).unwrap();
    }else{
        end_crop_position = rest_of_cathegory.len()+1;
    };

    if end_crop_position>=1{
        return String::from(&rest_of_cathegory[..end_crop_position]);
    }else{
        return String::from("");
    };
}

fn get_season(title: &String) -> String {
    let re = regex::Regex::new(r"(?ix)
                                            (?:\s|s|season|temp-|temporada|temporada-)
                                            (\d+)([\S.]*)
                                            (?:e|x|episode|cap|cap-|\n)
                                            (\d{1,2})
                                            ").unwrap();
    // let re = regex::Regex::new(r"(?ix)(?:\s|s|season|temporada-)
    //                                         (\d+)
    //                                         ").unwrap();
    if let Some(captures) = re.captures(&title.to_lowercase()) {
        let season = String::from(captures.get(1).unwrap().as_str());
        if season[0..1] == String::from("0") {
            return String::from(&season.clone()[1..]);
        } else {
            return String::from(&season.clone());
        }
        //return String::from(captures.get(1).unwrap().as_str());
    } else {
        return String::from("");
    }
}

fn get_episode(title: &String) -> String {
    let re = regex::Regex::new(r"(?ix)
                            (?:
                            e|x|cap|cap-|episode
                            )
                            \s*
                            (\d{1,2})
                            ").unwrap();
    if let Some(captures) = re.captures(&title.to_lowercase()) {
        let episode = String::from(captures.get(1).unwrap().as_str());
        if episode[0..1] == String::from("0") {
            return String::from(&episode.clone()[1..]);
        } else {
            return String::from(&episode.clone());
        }
        //return String::from(captures.get(1).unwrap().as_str());
    } else {
        return String::from("");
    }
}

fn get_pub_date(date_yyyy_mm_dd: &String) -> String {
    let re = regex::Regex::new(r"(?ix)(\d{4})-(\d{2})-(\d{2})").unwrap();
    if let Some(captures) = re.captures(&date_yyyy_mm_dd) {
        return String::from(captures.get(0).unwrap().as_str());
    } else {
        return String::from("");
    }
}

fn get_quality(title: &String) -> String {
    let mut quality: String = String::from("");

    if title.find("480p").is_some() {
        quality = String::from("480p");
    }

    if title.find("720p").is_some(){
        quality = String::from("720p");
    }

    if title.find("1080p").is_some(){
        quality = String::from("1080p");
    }

    if title.find("HDTV").is_some(){
        quality = String::from("1080p");
    }

    if title.find("1440p").is_some(){
        quality =  String::from("1440p");
    }

    if title.find("2K").is_some(){
        quality = String::from("1080p");
    }

    if title.find("4K").is_some(){
        quality = String::from("2160p");
    }

    if title.find("8K").is_some(){
        quality =  String::from("4320p");
    }
    return String::from(&quality.clone());

}

fn make_ascii_titlecase(s: &str) -> String {

    let letra_inical = s.get(0..1).unwrap_or("");
    let resto_palabra = s.get(1..).unwrap_or("");
    return format!("{}{}",String::from(letra_inical.to_uppercase()),String::from(resto_palabra));
}

fn capitalize_each_word (a_string: &String) -> String {
    let mut cap_result: String = String::from("");

    for a_word in a_string.split_whitespace() {
        cap_result=format!("{} {}",&cap_result.trim(),make_ascii_titlecase(a_word));
    };
    return String::from(cap_result.trim());
}

fn get_clean_name(title: &String) -> String {
    let words_to_remove: [&str; 25]=[
        "- 1ª Temporada",
        "- 2ª Temporada",
        "- 3ª Temporada",
        "- 4ª Temporada",
        "- 5ª Temporada",
        "- 6ª Temporada",
        "- 7ª Temporada",
        "- 8ª Temporada",
        "- 9ª Temporada",
        "480p",
        "720p",
        "1080p",
        "1440p",
        "2160p",
        "4320p",
        "2K",
        "4K",
        "8K",
        "DVD",
        "BlueRay",
        "&amp;",
        "HD",
        "AC3 5.1",
        "AC3-5-1",
        ":"
    ];
    let mut cleaned_title: String=title.clone().to_lowercase();

    // Clean words between [ and ] or , and >
    let re_words_in_brackets = regex::Regex::new(r"(<.*?>|\[.*?\])").unwrap();
    if let Some(to_delete_in_brackets) = re_words_in_brackets.captures(&cleaned_title){
        cleaned_title = String::from(cleaned_title.replace(to_delete_in_brackets.get(0).unwrap().as_str().trim(), ""));
    }

    // Clean season and episode
    // .-first single season and episode
    let re_season_and_episode = regex::Regex::new(r"(?ix)(?:\s|s|season)(\d+)(?:e|x|episode|\n)(\d{2})").unwrap();
    if let Some(to_delete_season_episode) = re_season_and_episode.captures(&cleaned_title){
        cleaned_title = String::from(cleaned_title.replace(to_delete_season_episode.get(0).unwrap().as_str().trim(), ""));
    }
    // .-second when there is a range from episodes like "SxEE al SxEE"
    let re_season_and_episode = regex::Regex::new(r"(?:\sal)(?ix)(?:\s|s|season)(\d+)(?:e|x|episode|\n)(\d{2})").unwrap();
    if let Some(to_delete_season_episode) = re_season_and_episode.captures(&cleaned_title){
        cleaned_title = String::from(cleaned_title.replace(to_delete_season_episode.get(0).unwrap().as_str().trim(), ""));
    }

    // Clean removable words
    for removable_word in words_to_remove{
        if cleaned_title.find(&removable_word.to_lowercase()).is_some() {
            cleaned_title = cleaned_title.replace(&removable_word.to_lowercase(), "");
        }
    };

    // Clean empty brackets
    cleaned_title = cleaned_title.replace("[]", "");

    // Clean double spaces
    cleaned_title = cleaned_title.replace("  ", " ").replace("  ", " ");

    return String::from(capitalize_each_word(&cleaned_title));

}


fn scrape_download_page_and_get_torrent_link(href_link: &String,element_id: &String, search_for_string: &String) -> Vec<String> {
    let mut torrent_links: Vec<String> = Vec::new();//vec![String::from("")];

    let torrents_page = reqwest::blocking::get(href_link.as_str())
        .unwrap()
        .text()
        .unwrap_or(String::from(" "));
    let torrents_page_document = scraper::Html::parse_document(&torrents_page);
    let a_element_id = format!("a#{}",element_id);    // use the # to specify the id attribute of an html element
    let torrents_page_selector = scraper::Selector::parse(&a_element_id).unwrap();
    let torrents_links_list = torrents_page_document
        .select(&torrents_page_selector)
        .filter(|item| item.inner_html() == String::from(search_for_string))
        .map(|item_text: scraper::ElementRef| item_text.html());

    torrents_links_list
        .zip(1..100)
        .for_each(|(item, _number)|{
            let href_path = get_href_path(&item).replace("//", "https://");
            torrent_links.push(String::from(&href_path));
        });

    return torrent_links;
}

fn get_latest_torrents (configdata: &ConfigData) -> RSSRoot {

    let mut root_rss: RSSRoot;
    let mut channel_rss: RSSChannel;
    let now_date_time: String = Local::now().to_rfc3339().replace("T"," ");

    //let url_path = configdata.config.website_path.clone();
    let last_torrents_url = format!("{}/{}",configdata.config.website_url,configdata.config.website_path)
        .replace("//", "/")
        .replace(":/", "://");

    println!("\nScraping last torrents from:'{}'", last_torrents_url);

    let links_page = reqwest::blocking::get(last_torrents_url.as_str())
        .unwrap()
        .text()
        .unwrap_or(String::from(" "));

    let document = scraper::Html::parse_document(&links_page);

    //let div_id_ultimos = format!("{}","div.seccion > * > * > * > a.text-primary");
    let div_id_ultimos = format!("{}",configdata.config.div_id_ultimos);
    let links_page_selector = scraper::Selector::parse(div_id_ultimos.as_str()).unwrap();

    let links_list = document
        .select(&links_page_selector)
        .map(|item_text: scraper::ElementRef| item_text.html());

    //println!("{:#?}",links_list);

    // Write some data to the file
    root_rss = RSSRoot{
        version:String::from("2.0"),
        channels:Vec::new(),
    };

    channel_rss = RSSChannel{
        title:String::from("DonTorrent RSS"),
        link:String::from("https://20.12.69.250"),
        description:String::from("DonTorrent - ultimos torrents publicados"),
        last_build_date:String::from(&now_date_time),
        items: Vec::new(),
    };

    links_list
        .zip(1..121)
        .for_each(|(item, number)|{

            if item.find("href=\"").is_some(){

                println!("   {}............................................................................", number);

                let href_path = get_href_path(&item);
                let href_link = format!("{}{}",&configdata.config.website_url,&href_path);

                let title =  get_title(&item);
                let quality = get_quality(&title);
                let cathegory = get_cathegory(&href_path);
                let season = get_season(&title);
                let episode: String ;
                if season.len()>0{
                    episode = get_episode(&title);
                } else {
                    episode = String::from("");
                }
                let cleaned_title =  get_clean_name(&title);


                println!("          href link:´{}´", &href_link);
                println!("          cathegory:´{}´", &cathegory);
                println!("              title:´{}´", &title);
                println!("      cleaned title:´{}´", &cleaned_title);
                println!("      torrent links:");

                let torrent_download_links: Vec<String> = scrape_download_page_and_get_torrent_link( &href_link,
                                                                                        &configdata.config.link_id_download_torrent,
                                                                                    &configdata.config.link_text_download_torrent);

                let mut torr_quality: String = String::from("");
                let torrents_list = torrent_download_links.iter();
                torrents_list
                    .for_each(|torr_item|{
                        println!("                  (found).- ´{}´  ({}x{})",&torr_item,get_season(&torr_item),get_episode(&torr_item));

                        if  (episode.len()==0 && season.len()==0) ||
                            (episode.len()>0 && season.len()>0 && get_season(&torr_item).len()==0 && get_episode(&torr_item).eq(&episode)) ||
                            (episode.len()>0 && season.len()>0 && get_season(&torr_item).eq(&season) && get_episode(&torr_item).eq(&episode)){

                            println!("               >selected>.- ´{}´  ({}x{})",&torr_item,get_season(&torr_item),get_episode(&torr_item));

                            let mut item_rss: RSSItem = RSSItem{
                                title: String::from(&cleaned_title),
                                category: String::from(&cathegory),
                                season: String::from(""),
                                episode: String::from(""),
                                link: String::from(&href_link),
                                quality: String::from(&quality),
                                pub_date: String::from(&now_date_time),
                                enclosure_url: String::from(&torr_item.clone()),
                                enclosure_length: 201269,
                                enclosure_type:String::from("application/x-bittorrent"),
                            };

                            if season.len()>0 && episode.len()>0 {
                                item_rss.title=format!("{} {}x{}",&cleaned_title,&season,&episode);
                                item_rss.season = String::from(&season);
                                item_rss.episode = String::from(&episode);
                            };


                            // torr_quality = get_quality(&torr_item);
                            // if torr_quality.len()>0{
                            //     item_rss.quality=String::from(&torr_quality);
                            // };

                            if item_rss.enclosure_url.len()>0{   // adding only items with some download link
                                channel_rss.items.push(item_rss);
                            }
                        };

                    });
                println!("            quality:´{}´", &quality);
                println!("             season:´{}´", &season);
                println!("            episode:´{}´", &episode);
            };
    });


    root_rss.channels.push(channel_rss);
    return root_rss;

}

fn main() {
    // IMPORTANT:
    // ==========
    // To be able to compile in Alpine Linux in arm64,
    // 1.) install these packages in the Alpine:
    //         sudo apk add pkgconfig
    //         sudo apk add gcc musl-dev openssl openssl-dev
    // 2.) and add to Cargo.toml the following dependency:
    //         git2 = {version="0.13.22", features = ["vendored-libgit2"]}
    // 3.) and compile passing the -Ctarget-features=-crt-static argument like:
    //         RUSTFLAGS="-Ctarget-feature=-crt-static" cargo build
    // because rust only links to static libraries when building a static binary,
    // which is the default for the musl target
    // but to build a dynamic binary which can link to dynamic libraries,
    // you need to use RUSTFLAGS="-Ctarget-feature=-crt-static".

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

    let previous_domain = configdata.config.website_url.clone();

    let now_date_time: String = Local::now().to_rfc3339().replace("T"," ");
    println!("\nRun time     : {}",now_date_time);

    // Print out the values to `stdout`.
    println!("\nConfiguration:");
    println!("------------------------------------------------------------------------");
    println!("telegram_url:               {}", &configdata.config.telegram_url);
    println!("website_url:                {}", &configdata.config.website_url);
    println!("website_path:               {}", &configdata.config.website_path);
    println!("div_id_ultimos:             {}", &configdata.config.div_id_ultimos);
    println!("link_id_download_torrent:   {}", &configdata.config.link_id_download_torrent);
    println!("link_text_download_torrent: {}", &configdata.config.link_text_download_torrent);
    println!("output_file:                {}", &configdata.config.output_file);

    let last_domain = get_last_dontorrent_domain(&configdata.config.telegram_url);

    println!("\nPrevious domain:'{}'", previous_domain);
    println!("Last domain:'{}'", last_domain);

    let mut newconfigdata: ConfigData;

    if previous_domain.ne(&last_domain){
        let newsettings: Settings = Settings{
            telegram_url:configdata.config.telegram_url,
            website_url:last_domain.clone(),
            website_path:configdata.config.website_path.clone(),
            div_id_ultimos: configdata.config.div_id_ultimos.clone(),
            link_id_download_torrent:configdata.config.link_id_download_torrent.clone(),
            link_text_download_torrent:configdata.config.link_text_download_torrent.clone(),
            output_file:configdata.config.output_file.clone(),
        };

        newconfigdata = ConfigData{
            config: newsettings,
        };
        write_config(filename,&newconfigdata);
    } else{
        newconfigdata = configdata;
    };

    let root_rss = get_latest_torrents(&newconfigdata);
    root_rss.write_to_file(&newconfigdata.config.output_file);

}
