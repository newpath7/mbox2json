# mbox2json
## Introduction
* Outputs an mbox file in JSON format (an array of email objects)
* Only the following fields are included *To, From, Bcc, Cc, Date, Subject* and *Body*
* Can save to a CouchDB (just put credentials in a file named .env; see .env example in repo)

## Usage
cargo run &lt;path-to-mbox-file&gt; [*nbytes start*] [*-tcdb*] [*skip*]  
*nbytes start* to skip first nbytes of the mbox file   
Use *-tcdb* to send to a CouchDB instead. *skip* is the number of parsed emails to discard before actually saving to DB.
