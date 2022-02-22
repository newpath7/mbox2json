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

use mailbox;
use regex::Regex;
use serde::Serialize;
use serde_json;
use std::{fs::File, env, str};
use std::vec::Vec;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

#[derive(Serialize)]
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

#[derive(Serialize)]
struct MyMail {
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

fn main() {
	let mbox = mailbox::read(File::open(env::args()
		.nth(1).expect("no file given")).unwrap());
    print!("[");
    let mut i = 0;

	for mail in mbox {
        let mut mymailito = MyMail {
			to: AddInfo { val: "".to_string(), ads: Vec::new(),},
            from: AddInfo { val: "".to_string(), ads: Vec::new(), },
            bcc: AddInfo { val: "".to_string(), ads: Vec::new(), },
            cc: AddInfo { val: "".to_string(), ads: Vec::new(), },
            date: "".to_string(), cdate: 0,
            subject: "".to_string(),  body: "".to_string(),
         };
		let mref = mail.as_ref();

		let mref = match mref {
			Ok(mr) => {
        		if i > 0 { print!(",\n"); }
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
        		print!("{}", serde_json::to_string(&mymailito).unwrap());
			},
			_ => continue,
		};
	}
	println!("]");
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
