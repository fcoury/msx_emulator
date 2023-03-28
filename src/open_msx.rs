use std::env;
use std::io::{BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use anyhow::{anyhow, Error, Result};
use tracing::{event, span, Level};
use walkdir::WalkDir;
use xml::reader::{EventReader, XmlEvent};

pub struct Client<'a> {
    pub reader: EventReader<&'a UnixStream>,
    pub writer: BufWriter<&'a UnixStream>,
}

pub enum Response {
    Ok(String),
    Nok(String),
}

impl<'a> Client<'a> {
    pub fn new(socket: &'a UnixStream) -> Result<Client<'a>, Error> {
        let span = span!(Level::DEBUG, "Client::new");
        let _enter = span.enter();

        let writer = BufWriter::new(socket);
        let mut reader = EventReader::new(socket);

        loop {
            match reader.next() {
                Ok(XmlEvent::StartElement { name, .. }) if name.local_name == "openmsx-output" => {
                    event!(Level::DEBUG, "openMSX is ready.");
                    return Ok(Client { reader, writer });
                }
                Ok(event) => {
                    event!(Level::TRACE, "xml event: {event:?}", event = event);
                }
                Err(err) => {
                    return Err(anyhow!(err));
                }
            };
        }
    }

    pub fn request(&mut self, command: &String) -> Result<Response, Error> {
        let span = span!(Level::DEBUG, "sending a command");
        let _enter = span.enter();

        self.writer.write_all(b"<command>")?;
        self.writer.write_all(command.as_bytes())?;
        self.writer.write_all(b"</command>\n")?;
        self.writer.flush()?;

        event!(Level::DEBUG, "sent command: {command}", command = command);

        let mut ok: String = String::from("nok");

        loop {
            match self.reader.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) if name.local_name == "reply" => {
                    ok = attributes
                        .iter()
                        .find(|attr| attr.name.local_name == "result")
                        .map(|attr| attr.value.to_owned())
                        .ok_or_else(|| anyhow!("result attribute is undefined"))?;
                    break;
                }
                Ok(event) => {
                    event!(Level::TRACE, "xml event: {event:?}", event = event);
                }
                Err(err) => {
                    return Err(anyhow!(err));
                }
            };
        }

        let mut data = String::new();

        loop {
            match self.reader.next() {
                Ok(XmlEvent::Characters(chunk)) => {
                    data.push_str(chunk.as_str());
                }
                Ok(XmlEvent::EndElement { name, .. }) if name.local_name == "reply" => {
                    event!(Level::DEBUG, "reply: {ok:?}. {data}", ok = ok, data = data);

                    return if ok == "ok" {
                        Ok(Response::Ok(data))
                    } else {
                        Ok(Response::Nok(data))
                    };
                }
                Ok(event) => {
                    event!(Level::TRACE, "xml event: {event:?}", event = event);
                }
                Err(err) => {
                    return Err(anyhow!(err));
                }
            }
        }
    }
}

pub fn find_socket() -> Result<PathBuf, Error> {
    let username = env::var("USER")?;
    let socket_folder_pattern = format!("openmsx-{}", username);

    for entry in WalkDir::new("/private/var/folders")
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if let Some(dir_name) = entry.file_name().to_str() {
            if dir_name.starts_with(&socket_folder_pattern) && entry.file_type().is_dir() {
                for subentry in entry.path().read_dir()? {
                    let subentry = subentry?;
                    if let Some(socket_name) = subentry.file_name().to_str() {
                        if socket_name.starts_with("socket.") {
                            return Ok(subentry.path());
                        }
                    }
                }
            }
        }
    }

    Err(anyhow!("Socket file not found."))
}
