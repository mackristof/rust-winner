
extern crate reqwest;
extern crate rustc_serialize;


use rustc_serialize::json;
use std::net::{TcpStream, TcpListener};
use std::io::{Read, Write};
use std::thread;
use std::env;




#[derive(RustcDecodable, RustcEncodable)]
pub struct LastRequest  {
    events: Vec<Event>
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Event  {
    id: String,
    start: Start,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Start {
    utc: String,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct EventDetail {
    pagination: Pagination,
    attendees: Vec<Attendee>,
}

#[derive(RustcDecodable, RustcEncodable)]
pub struct Pagination {
    page_number: i32,
    page_count: i32,
}

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct Attendee {
    id: String,
    profile: Profile,
}

#[derive(Debug)]
#[derive(RustcDecodable, RustcEncodable)]
pub struct Profile {
    first_name: String,
    last_name: String,
}


fn main() {
    let token = match env::var("TOKEN"){
        Ok(token) => (token),
        Err(err) => panic!("token not set in env : {}", err),
    };
    let event_id = match get_event_id(&token){
        Ok(event_id) => {
            event_id
        },
        Err(err) => panic!("can't get eventId: {}", err),
    };
    match get_attendees(&token, &event_id){
        Ok(attendees) => {
            println!("attendees {:?}", attendees.len());
            listen_http_simple()
        },
        Err(err) => panic!("can't get attendees: {}", err),
    }
    println!("last event id: {:?}", event_id);

}

fn listen_http_simple(){
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Listening for connections on port {}", 8080);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}

fn handle_read(mut stream: &TcpStream) {
    let mut buf = [0u8 ;4096];
    match stream.read(&mut buf) {
        Ok(_) => {
            let req_str = String::from_utf8_lossy(&buf);
            println!("{}", req_str);
        },
        Err(e) => println!("Unable to read stream: {}", e),
    }
}

fn handle_write(mut stream: TcpStream) {
    let response = b"HTTP/1.1 200 OK\r\nAccess-Control-Allow-Credentials: true\r\nAccess-Control-Allow-Headers: Accept, Accept-Encoding, Authorization, Content-Length, Content-Type, X-CSRF-Token\r\nAccess-Control-Allow-Methods: POST, GET, OPTIONS, PUT, DELETE, HEAD, PATCH\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Expose-Headers: Accept, Accept-Encoding, Authorization, Content-Length, Content-Type, X-CSRF-Token\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n<html><body>Hello world</body></html>\r\n";
    match stream.write(response) {
        Ok(_) => println!("Response sent"),
        Err(e) => println!("Failed sending response: {}", e),
    }
}

fn handle_client(stream: TcpStream) {
    handle_read(&stream);
    handle_write(stream);
}

fn get_attendees(token: &String, event_id: &String) -> Result <Vec<Attendee>, String> {
    let mut page: i32 = 1;
    let mut page_count: i32 = 1;
    let mut attendees: Vec<Attendee> = Vec::new() ;
    while page <= page_count {
        let url = format!("https://www.eventbriteapi.com/v3/events/{}/attendees/?page={}&token={}",
                          event_id,
                          page,
                          token.as_str() );
        println!("url: {:?}", url);
        let mut response = match reqwest::get(&url) {
            Ok(response) => response,
            Err(e) => return Err(e.to_string()),
        };
        let mut buf = String::new();
        match response.read_to_string(&mut buf) {
            Ok(read) => (read),
            Err(err) => return Err(err.to_string()),
        };

        let event_detail: EventDetail =  json::decode(&buf).unwrap();
        println!("page_count: {:?}", event_detail.pagination.page_count);
        page_count = event_detail.pagination.page_count;
        for attendee in event_detail.attendees {
            attendees.push(attendee);
        }
        page += 1 ;
    }
    return Ok(attendees);
}

fn get_event_id(token: &String) -> Result <String, String> {
    let url = format!("https://www.eventbriteapi.com/v3/events/search/?token={}&organizer.id=1464915124", token.as_str());
    let mut response = match reqwest::get(&url) {
        Ok(response) => response,
        Err(e) => return Err(e.to_string()),
    };
    let mut buf = String::new();
    match response.read_to_string(&mut buf) {
        Ok(read) => (read),
        Err(err) => return Err(err.to_string()),
    };

    let mut jsonstuct: LastRequest =  json::decode(&buf).unwrap();
    return match jsonstuct.events.pop(){
        Some(event) => Ok(event.id),
        None =>   Err(format!("no eventId")),
    };

}

