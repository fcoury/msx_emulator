use std::env;
use std::fmt;
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

#[derive(Debug, Eq, PartialEq)]
pub struct InternalState {
    // 8-bit registers
    pub a: u8,
    pub f: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub h: u8,
    pub l: u8,

    // 16-bit registers
    pub sp: u16,
    pub pc: u16,
}

impl fmt::Display for InternalState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let flags = format!(
            "S: {} Z: {} H: {} P/V: {} N: {} C: {}",
            if self.f & 0b1000_0000 != 0 { "1" } else { "0" },
            if self.f & 0b0100_0000 != 0 { "1" } else { "0" },
            if self.f & 0b0001_0000 != 0 { "1" } else { "0" },
            if self.f & 0b0000_0100 != 0 { "1" } else { "0" },
            if self.f & 0b0000_0010 != 0 { "1" } else { "0" },
            if self.f & 0b0000_0001 != 0 { "1" } else { "0" },
        );
        // FIXME apparently the F3 and F5 registers are accounted for on the openMSX, we're skipping it for now
        // write!(
        //     f,
        //     "#{:04X} - A: #{:02X} B: #{:02X} C: #{:02X} D: #{:02X} E: #{:02X} F: #{:02X} H: #{:02X} L: #{:02X} - {}",
        //     self.pc, self.a, self.b, self.c, self.d, self.e, self.f, self.h, self.l, flags
        // )
        write!(
            f,
            "#{:04X} - A: #{:02X} B: #{:02X} C: #{:02X} D: #{:02X} E: #{:02X} H: #{:02X} L: #{:02X} - {}",
            self.pc, self.a, self.b, self.c, self.d, self.e, self.h, self.l, flags
        )
    }
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

    pub fn get_status(&mut self) -> anyhow::Result<InternalState> {
        let pc = self.get("reg pc")?.parse()?;
        let sp = self.get("reg sp")?.parse()?;
        let a = self.get("reg a")?.parse()?;
        let f = self.get("reg f")?.parse()?;
        let b = self.get("reg b")?.parse()?;
        let c = self.get("reg c")?.parse()?;
        let d = self.get("reg d")?.parse()?;
        let e = self.get("reg e")?.parse()?;
        let h = self.get("reg h")?.parse()?;
        let l = self.get("reg l")?.parse()?;

        Ok(InternalState {
            pc,
            sp,
            a,
            f,
            b,
            c,
            d,
            e,
            h,
            l,
        })
    }

    pub fn get(&mut self, command: &str) -> anyhow::Result<String> {
        match self.request(command) {
            Ok(Response::Ok(data)) => Ok(data),
            Ok(Response::Nok(data)) => Err(anyhow!("openMSX error: {}", data)),
            Err(e) => Err(e),
        }
    }

    pub fn request(&mut self, command: &str) -> Result<Response, Error> {
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
