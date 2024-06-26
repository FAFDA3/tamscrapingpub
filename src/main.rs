use reqwest::{Error, header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, REFERER, CONNECTION, UPGRADE_INSECURE_REQUESTS, ACCEPT_ENCODING}};
use scraper::{Html, Selector};
use tokio::time::{sleep, Duration};
use tokio;
use tokio:: sync:: oneshot;
use std::fs;
use std::path::Path;
use flate2::read::GzDecoder;
use std::io::{self, Read, Write};
use std::sync::Mutex;
use std::env;
use regex:: Regex;
use serde:: {Deserialize, Serialize};
use serde_json::{Value, Result as Resultserde};
use actix_web::{Responder, web, get, HttpResponse, HttpServer, App};

fn create_airbnb_url( checkin: String, checkout:String, adults: u64, children: u64, infants:u64, latitude1: f64, latitude2: f64, longitude1: f64, longitude2: f64, cursor: String) -> String {
    format!("https://it.airbnb.com/s/homes?refinement_paths%5B%5D=%2Fhomes&place_id=ChIJu46S-ZZhLxMROG5lkwZ3D7k&checkin={}&checkout={}&adults={}&children={}&infants={}&tab_id=home_tab&query=Rome%2C+Italie&flexible_trip_lengths%5B%5D=one_week&monthly_start_date=2024-05-01&monthly_length=3&monthly_end_date=2029-10-01&search_mode=regular_search&price_filter_input_type=0&price_filter_num_nights=12&channel=EXPLORE&ne_lat={}&ne_lng={}&sw_lat={}&sw_lng={}&zoom=12.930721908719006&zoom_level=12.930721908719006&search_by_map=true&search_type=user_map_move&cursor={}", checkin, checkout, adults, children, infants, latitude1, longitude1, latitude2, longitude2, cursor)
}

/*  cursor list:
yJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0IjowLCJ2ZXJzaW9uIjoxfQ%3D%3D
eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0IjoxOCwidmVyc2lvbiI6MX0%3D
eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0IjozNiwidmVyc2lvbiI6MX0%3D
eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0Ijo1NCwidmVyc2lvbiI6MX0%3D
eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0Ijo3MiwidmVyc2lvbiI6MX0%3D
eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0Ijo5MCwidmVyc2lvbiI6MX0%3D
eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0IjEwOCwidmVyc2lvbiI6MX0%3D
eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0IjEyNiwidmVyc2lvbiI6MX0%3D

*/
async fn fetch_html(url: &str) -> Result<String, Error> {
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"));
    headers.insert(REFERER, HeaderValue::from_static("https://www.google.com"));
    headers.insert(CONNECTION, HeaderValue::from_static("keep-alive"));
    headers.insert(UPGRADE_INSECURE_REQUESTS, HeaderValue::from_static("1"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip, deflate, br, zstd"));

    let response = client.get(url)
        .headers(headers.clone())
        .send()
        .await?;

    let final_url = response.url().clone();

    let response = client.get(final_url)
        .headers(headers)
        .send()
        .await?;

    let local_response = &response;
    //println!("{}", response);

    let status = response.status();
    let headers = response.headers().clone();
   // let body = response.text().await?;
    let body= response.bytes().await?;
  //  println!("Response Body {}",  body);
   // println!("Response Status: {}", status);
   // println!("Response Headers:\n{:#?}", headers);

   
    let mut gz = GzDecoder::new(&body[..]);
    let mut s = String::new();
    gz.read_to_string( &mut s);

    Ok(s)
}


fn extract_json (html_content: &str)->Option<String>{
    let re = Regex::new(r#"data-deferred-state-0="true" type="application/json">([^<]+)</script></body></html>"#).unwrap();
    re.captures(html_content).and_then(|cap| cap.get(1)).map(|m| m.as_str().to_string())



}

fn save_html(content: &str, folder: &str, filename: &str) -> std::io::Result<()> {
    let path = Path::new(folder);
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    let filepath = path.join(filename);
    fs::write(filepath, content)?;
    Ok(())
}

fn extract_data(html_content: &str) -> String {
    html_content.to_string()
}



fn use_json(path: &str) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let json_str = fs::read_to_string(path)?;
    let parsed_json: Value = serde_json::from_str(&json_str)?;
    Ok(parsed_json)
}



/*
async fn run_scraper(checkin: String, checkout: String, adults: u64, children: u64, infants: u64,  lat1: f64, lat2: f64, long1: f64, long2: f64, cursor: String, app_state: web::Data<AppState>) {
    loop {
        let url = create_airbnb_url(checkin.clone(), checkout.clone(), adults, children, infants, lat1, lat2, long1, long2, cursor.clone());
        println!("URL created: {} here the html", url);

        match fetch_html(&url).await {
            Ok(html_content) => {
                let data = extract_data(&html_content);

                {
                    let mut html_lock = app_state.html.lock().unwrap();
                    *html_lock = Ok(data.clone());
                }

                if let Err(e) = save_html(&data, "HTML", "test20240608.html") {
                    eprintln!("Error saving HTML: {}", e);
                } else {
                    println!("HTML saved successfully.");
                }

                if let Some(json_content) = extract_json(&html_content) {
                    if let Err(e) = save_html(&json_content, "HTML", "extracted_data.json") {
                        eprintln!("Error saving JSON: {}", e);
                    } else {
                        println!("JSON saved successfully.");

                        match use_json("HTML/extracted_data.json") {
                            Ok(parsed_json) => {
                                let mut listings_lock = app_state.listings.lock().unwrap();
                                *listings_lock = extract_listings(&parsed_json);
                                println!("Extracted listings successfully.");
                            }
                            Err(e) => {
                                eprintln!("Error parsing JSON: {}", e);
                            }
                        }
                    }
                } else {
                    println!("No JSON content found.");
                }
            }
            Err(e) => eprintln!("Error fetching HTML: {}", e),
        }

        sleep(Duration::from_secs(1800)).await;
    }
}*/




fn extract_listings(json: &Value) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    println!{"extracting_listing"};
    let listings_array = json["niobeMinimalClientData"][0][1]["data"]["presentation"]["staysSearch"]["results"]["searchResults"]
        .as_array()
        .ok_or("Failed to find listings")?
        .clone();

    Ok(listings_array)
}

async fn run_scraper(checkin: String, checkout: String, adults: u64, children: u64, infants: u64,  lat1: f64, lat2: f64, long1: f64, long2: f64, cursor: String, app_state: web::Data<AppState>, tx: oneshot::Sender<()>) {
    let url = create_airbnb_url(checkin.clone(), checkout.clone(), adults, children, infants, lat1, lat2, long1, long2, cursor.clone());
    println!("URL created: {} here the html", url);

    match fetch_html(&url).await {
        Ok(html_content) => {
            let data = extract_data(&html_content);

            {
                let mut html_lock = app_state.html.lock().unwrap();
                *html_lock = Ok(data.clone());
            }

            if let Err(e) = save_html(&data, "HTML", "test20240608.html") {
                eprintln!("Error saving HTML: {}", e);
            } else {
                println!("HTML saved successfully.");
            }

            if let Some(json_content) = extract_json(&html_content) {
                if let Err(e) = save_html(&json_content, "HTML", "extracted_data.json") {
                    eprintln!("Error saving JSON: {}", e);
                } else {
                    println!("JSON saved successfully.");

                    match use_json("HTML/extracted_data.json") {
                        Ok(parsed_json) => {
                            let mut listings_lock = app_state.listings.lock().unwrap();
                            *listings_lock = extract_listings(&parsed_json);
                            println!("Extracted listings successfully.");
                        }
                        Err(e) => {
                            eprintln!("Error parsing JSON: {}", e);
                        }
                    }
                }
            } else {
                println!("No JSON content found.");
            }
        }
        Err(e) => eprintln!("Error fetching HTML: {}", e),
    }

    let _ = tx.send(());

    sleep(Duration::from_secs(1800)).await;
}





#[get("/listings")]
async fn listings(data: web::Data<AppState>) -> impl Responder {
    let listings = data.listings.lock().unwrap();
    match &*listings {
        Ok(listings) => HttpResponse::Ok().json(listings),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[get("/html")]
async fn html(data: web::Data<AppState>) -> impl Responder {
    let html = data.html.lock().unwrap();
    match &*html {
        Ok(html) => HttpResponse::Ok().body(html.clone()),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}



struct AppState {
    listings: Mutex<Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>>>,
    html: Mutex<Result<String, Box<dyn std::error::Error + Send + Sync>>>,
}


#[get("/start_scraper")]
async fn start_scraper(
    query: web::Query<StartScraperParams>,
    data: web::Data<AppState>,
) -> impl Responder {
    let params = query.into_inner();
    let app_state_clone = data.clone();
    let (tx, rx) = oneshot::channel();

    tokio::spawn(run_scraper(
        params.checkin,
        params.checkout,
        params.adults,
        params.children,
        params.infants,
        params.lat1,
        params.lat2,
        params.long1,
        params.long2,
        params.cursor,
        app_state_clone,
        tx,
    ));

    // Wait for the scraper to finish its first run
    rx.await.unwrap();

    // Return the listings
    let listings_lock = data.listings.lock().unwrap();
    match &*listings_lock {
        Ok(listings_lock) => HttpResponse::Ok().json(listings_lock),
        Err(e) => HttpResponse::InternalServerError().body(format!("Error: {}", e)),
    }
}

#[derive(Deserialize)]
struct StartScraperParams {
    checkin: String,
    checkout: String,
    adults: u64,
    children: u64,
    infants: u64,
    lat1: f64,
    lat2: f64,
    long1: f64,
    long2: f64,
    cursor: String,
}


#[actix_web::main]

async fn main() -> std::io::Result<()> {
    println!("start the scraping");

    let app_state = web::Data::new(AppState {
        listings: Mutex::new(Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No data yet")))),
        html: Mutex::new(Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No data yet")))),
    });

    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    let server = HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(listings)
            .service(html)
            .service(start_scraper)
    })
    .bind(("0.0.0.0", port.parse().unwrap()))?
    .run();

    server.await
}




/*async fn main() -> std::io::Result<()>{

    println!("start the scraping");

    let checkin_: &str = "2024-08-01";
    let checkout_: &str = "2024-08-20";
    
    let adults_ : u64 = 1;
    let children_: u64 = 0;
    let infants_: u64 = 0;
     
    let lat1: f64 = 43.947613;
    let lat2: f64 = 43.8520324685;

    let long1: f64 = 12.5242224779;
    let long2: f64 = 12.412739371;

    let cursor_: &str = "eyJzZWN0aW9uX29mZnNldCI6MCwiaXRlbXNfb2Zmc2V0IjoxOCwidmVyc2lvbiI6MX0%3Dgo";

    let app_state = web::Data::new(AppState{
        listings: Mutex :: new(Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No data yet")))),
        html: Mutex::new(Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "No data yet")))),
    
    });

    let app_state_clone = app_state.clone();
    
    let port = env::var("PORT").unwrap_or_else(|_| "8000".to_string());

    let server = HttpServer::new(move ||{
        App::new()
            .app_data(app_state.clone())
            .service(listings)
            .service(html)   
    
    })
    .bind(("0.0.0.0", port.parse().unwrap()))?
    .run();

    tokio::spawn(run_scraper(checkin_, checkout_ , adults_, children_, infants_ , lat1,lat2,long1,long2,cursor_, app_state_clone));

    server.await
}*/





fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read line");
    input
}


fn extract_contextual_content(html_content: &str) -> String{
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("[contextualPicturesPageInfo]").unwrap();
    document.select(&selector)
        .map(|element| element.html())
        .collect::<Vec<_>>()
        .join("\n")


}

/*
43.947613
43.8520324685

12.5242224779
12.412739371


*/