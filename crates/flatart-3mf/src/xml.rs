//! A small XML writer over quick-xml.

use flatart_domain::{Error, Result, Stage};
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};

pub struct Xml {
    writer: Writer<Vec<u8>>,
}

fn failed(error: impl std::fmt::Display) -> Error {
    Error::StageFailed {
        stage: Stage::Export,
        message: format!("writing XML: {error}"),
    }
}

fn element<'a>(name: &'a str, attributes: &[(&'a str, &'a str)]) -> BytesStart<'a> {
    BytesStart::new(name).with_attributes(attributes.iter().copied())
}

impl Xml {
    /// A document with the XML declaration, one element per line.
    pub fn new() -> Result<Self> {
        let mut xml = Self {
            writer: Writer::new_with_indent(Vec::new(), b' ', 1),
        };
        xml.event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
        Ok(xml)
    }

    fn event(&mut self, event: Event) -> Result<()> {
        self.writer.write_event(event).map_err(failed)
    }

    pub fn open(&mut self, name: &str, attributes: &[(&str, &str)]) -> Result<()> {
        self.event(Event::Start(element(name, attributes)))
    }

    pub fn close(&mut self, name: &str) -> Result<()> {
        self.event(Event::End(BytesEnd::new(name)))
    }

    pub fn empty(&mut self, name: &str, attributes: &[(&str, &str)]) -> Result<()> {
        self.event(Event::Empty(element(name, attributes)))
    }

    /// `<name …>text</name>`.
    pub fn text(&mut self, name: &str, attributes: &[(&str, &str)], text: &str) -> Result<()> {
        self.writer
            .create_element(name)
            .with_attributes(attributes.iter().copied())
            .write_text_content(BytesText::new(text))
            .map_err(failed)?;
        Ok(())
    }

    pub fn finish(self) -> Vec<u8> {
        let mut bytes = self.writer.into_inner();
        bytes.push(b'\n');
        bytes
    }
}

/// A coordinate with up to 5 decimals (10 nm) and no trailing zeros.
pub fn number(v: f64) -> String {
    let s = format!("{v:.5}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    match s {
        "-0" | "" => "0".into(),
        s => s.into(),
    }
}
