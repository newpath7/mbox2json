//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyright (C) 2022 Jungle Moon <newpath7@gmail.com>
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

// usage: mbox2json <mboxfile>
// In JSON format, outputs some fields from each email from mbox
// MBOX file format requirement(s):
//  * Date in the From header line should be 24 characters long 
//          - GMail MBox exports include timezones which break this requirement
use mailbox;
use regex::Regex;
use serde::{Serialize, Deserialize};
use serde_json;
use serde_json::to_value;
use std::{fs::File, env, str, 
    io::Seek, io::SeekFrom};
use std::vec::Vec;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

use dotenv::dotenv;
use couch_rs::CouchDocument;
use couch_rs::types::document::DocumentId;
use couch_rs::document::TypedCouchDocument;
use futures::executor::block_on;

use ansi_escapes;

#[derive(Serialize, Deserialize)]
struct AddInfo {
    val: String,
    ads: Vec<String>,  // addresses
}

impl AddInfo {
    fn setads(&mut self) {
// From RFC5322 (3.4 Address Specification:)
// address-list    =   (address *("," address)) / obs-addr-list
        self.ads = Vec::new();
        for c in Regex::new(r"([a-zA-Z0-9._-]+@[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+)").unwrap().captures_iter(self.val.as_str()) {
            self.ads.push(c.get(1).map_or("", |m| m.as_str()).to_string());
       }
    }
}

#[derive(Serialize, Deserialize, CouchDocument)]
struct MyMail {
    #[serde(skip_serializing_if = "String::is_empty")]
    _id: DocumentId,
    #[serde(skip_serializing_if = "String::is_empty")]
     _rev: String,
    to: AddInfo,
	from: AddInfo,
	bcc: AddInfo,
	cc: AddInfo,
	date: String,
    cdate: i64,
	subject: String,
	body: String,
}

impl MyMail {
    fn setcdate(&mut self) {
        self.cdate = NaiveDateTime::parse_from_str(self.date.as_str(),
                "%a, %d %b %Y %H:%M:%S %z")
            .unwrap_or(NaiveDateTime::new(NaiveDate::from_ymd(1970, 1, 1),
            NaiveTime::from_hms_milli(0, 0, 0, 0))).timestamp();
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let tcdb = match args.get(3) {
        Some(argu) => if argu.as_str() == "-tcdb" { true } else { false },
        None => false
    };

    if args.len() == 0 { 
        println!("usage: mbox2json <mbox-file> [nbytes start] [-tcdb] [skip]"); 
       std:: process::exit(0);
    }
    let mut mboxfile = File::open(&args[1]).expect("could not open file");

    if args.len() > 1 {
        mboxfile.seek(SeekFrom::Start(args[2].parse::<u64>()
            .unwrap_or(0))).unwrap();
    }
    let mut i = 0;
    let iskip: usize = match args.get(4) {
        Some(skip) => skip.parse::<usize>().unwrap(),
        None => 0,
    };

    let db = if tcdb {
        dotenv().ok(); 
        println!("");
        let client = couch_rs::Client::new_with_timeout(&env::var("host").unwrap(),
            Some(&env::var("username").unwrap()),
            Some(&env::var("password").unwrap()),
            Some(env::var("timeout").unwrap().parse::<u64>().unwrap())).unwrap();
        block_on(client.db(&env::var("database").unwrap_or("".to_string())))
    } else { 
        let client = couch_rs::Client::new_no_auth("http://172.17.0.2:5984").unwrap();
        block_on(client.db("database"))
    };

    if iskip > 0 { println!("Skipping first {} emails\n", iskip); }
	let mbox = mailbox::read(mboxfile);
    if !tcdb { print!("[");  }

	for mail in mbox {
        let mut mymailito = MyMail {
            _id: "".to_string(), 
            _rev: "".to_string(),
			to: AddInfo { val: "".to_string(), ads: Vec::new(),},
            from: AddInfo { val: "".to_string(), ads: Vec::new(), },
            bcc: AddInfo { val: "".to_string(), ads: Vec::new(), },
            cc: AddInfo { val: "".to_string(), ads: Vec::new(), },
            date: "".to_string(), cdate: 0,
            subject: "".to_string(),  body: "".to_string(),
         };
		let mref = mail.as_ref();

		let _mref = match mref {
			Ok(mr) => {
        		if i > 0 && !tcdb { print!(",\n"); }
				i += 1;
	           
				for hkey in mr.headers().keys() {
            		match hkey.as_ref().to_string().as_str() {
			    		"To" => { mymailito.to.val = gethfield(hkey.owner(), "To");
                          mymailito.to.setads() },
			    		"From" => { mymailito.from.val = gethfield(hkey.owner(), "From");
                          mymailito.from.setads() },
			    		"Cc" => { mymailito.cc.val = gethfield(hkey.owner(), "Cc");
                          mymailito.cc.setads() },
			    		"Bcc" => { mymailito.bcc.val = gethfield(hkey.owner(), "Bcc");
                          mymailito.bcc.setads() },
                		"Date" => mymailito.date = gethfield(hkey.owner(), "Date"),
                		"Subject" => mymailito.subject = gethfield(hkey.owner(), "Subject"),
                		_ => (),
            		}
				}
        		mymailito.setcdate();

        		for bod in mail.unwrap().body().iter() {
	        		mymailito.body.push(char::from(bod));
	    		}
        		if !tcdb { 
                    print!("{}", serde_json::to_string(&mymailito).unwrap());  
                } else {
                    if i > iskip {  
                    let mut value = to_value(mymailito).unwrap();
                    match  db.as_ref().unwrap().create(&mut value).await {
                        Ok(_v) => println!("{}Added: {}", ansi_escapes::EraseLines(2), i),
                        Err(e) => println!("{:?}", e),
                    };
                    }
                }
			},
			_ => continue,
		};
	}
	if !tcdb { println!("]");  }
}

fn gethfield(s: &String, misc: &str) -> String {
    let caps = Regex::new(&format!("{}:{}", misc, " (.*)"))
        .unwrap().captures(s);

    return match caps {
        Some(w) => String::from(w.get(1)
                        .map_or("", |m| m.as_str())),
        None => "".to_string(),
    };
}
