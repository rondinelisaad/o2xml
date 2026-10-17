use std::{
    ffi::{c_char, c_int, c_void, CStr, CString},
    io::{self, BufRead, BufReader, Read, Write},
    process::ExitCode,
    ptr,
    sync::{Mutex, OnceLock},
};

use serde::{Deserialize, Serialize};

const XML_PARSE_NONET: c_int = 1 << 11;
const XML_CHAR_ENCODING_NONE: c_int = 0;

type XmlDoc = c_void;
type XmlDtd = c_void;
type XmlParserCtxt = c_void;
type XmlParserInput = c_void;
type XmlParserInputBuffer = c_void;
type XmlSchema = c_void;
type XmlSchemaParserCtxt = c_void;
type XmlSchemaValidCtxt = c_void;
type XmlValidCtxt = c_void;

type ExternalEntityLoader = Option<
    unsafe extern "C" fn(
        url: *const c_char,
        public_id: *const c_char,
        context: *mut XmlParserCtxt,
    ) -> *mut XmlParserInput,
>;

#[repr(C)]
struct ParserInputPrefix {
    buffer: *mut XmlParserInputBuffer,
    filename: *const c_char,
}

#[link(name = "xml2")]
extern "C" {
    fn xmlInitParser();
    fn xmlSetExternalEntityLoader(loader: ExternalEntityLoader);
    fn xmlParserInputBufferCreateMem(
        memory: *const c_char,
        size: c_int,
        encoding: c_int,
    ) -> *mut XmlParserInputBuffer;
    fn xmlFreeParserInputBuffer(buffer: *mut XmlParserInputBuffer);
    fn xmlNewIOInputStream(
        context: *mut XmlParserCtxt,
        buffer: *mut XmlParserInputBuffer,
        encoding: c_int,
    ) -> *mut XmlParserInput;
    fn xmlStrdup(value: *const u8) -> *mut u8;
    fn xmlReadMemory(
        buffer: *const c_char,
        size: c_int,
        url: *const c_char,
        encoding: *const c_char,
        options: c_int,
    ) -> *mut XmlDoc;
    fn xmlFreeDoc(document: *mut XmlDoc);
    fn xmlParseDTD(public_id: *const u8, system_id: *const u8) -> *mut XmlDtd;
    fn xmlFreeDtd(dtd: *mut XmlDtd);
    fn xmlNewValidCtxt() -> *mut XmlValidCtxt;
    fn xmlFreeValidCtxt(context: *mut XmlValidCtxt);
    fn xmlValidateDtd(context: *mut XmlValidCtxt, document: *mut XmlDoc, dtd: *mut XmlDtd)
        -> c_int;
    fn xmlSchemaNewParserCtxt(url: *const c_char) -> *mut XmlSchemaParserCtxt;
    fn xmlSchemaFreeParserCtxt(context: *mut XmlSchemaParserCtxt);
    fn xmlSchemaParse(context: *mut XmlSchemaParserCtxt) -> *mut XmlSchema;
    fn xmlSchemaFree(schema: *mut XmlSchema);
    fn xmlSchemaNewValidCtxt(schema: *mut XmlSchema) -> *mut XmlSchemaValidCtxt;
    fn xmlSchemaFreeValidCtxt(context: *mut XmlSchemaValidCtxt);
    fn xmlSchemaValidateDoc(context: *mut XmlSchemaValidCtxt, document: *mut XmlDoc) -> c_int;
}

#[derive(Deserialize)]
struct Request {
    mode: Mode,
    grammar_uri: String,
    public_id: Option<String>,
    document_bytes: usize,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Dtd,
    Xsd,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Message<'a> {
    Resource {
        url: &'a str,
        public_id: Option<&'a str>,
    },
    Result {
        valid: bool,
    },
}

#[derive(Deserialize)]
struct ResourceReply {
    allowed: bool,
    uri: Option<String>,
    bytes: usize,
}

struct Resource {
    uri: CString,
    bytes: Vec<u8>,
}

struct Broker {
    reader: BufReader<io::Stdin>,
    writer: io::Stdout,
}

impl Broker {
    fn request(&mut self, url: &str, public_id: Option<&str>) -> Result<Option<Resource>, String> {
        serde_json::to_writer(&mut self.writer, &Message::Resource { url, public_id })
            .map_err(|error| error.to_string())?;
        self.writer
            .write_all(b"\n")
            .and_then(|()| self.writer.flush())
            .map_err(|error| error.to_string())?;

        let mut line = String::new();
        self.reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        let reply: ResourceReply =
            serde_json::from_str(line.trim_end()).map_err(|error| error.to_string())?;
        let mut bytes = vec![0_u8; reply.bytes];
        self.reader
            .read_exact(&mut bytes)
            .map_err(|error| error.to_string())?;
        let mut delimiter = [0_u8; 1];
        self.reader
            .read_exact(&mut delimiter)
            .map_err(|error| error.to_string())?;
        if delimiter != *b"\n" {
            return Err("resource frame is missing its delimiter".into());
        }
        if !reply.allowed {
            return Ok(None);
        }
        let uri = reply
            .uri
            .ok_or_else(|| "allowed resource has no URI".to_owned())?;
        Ok(Some(Resource {
            uri: CString::new(uri).map_err(|_| "resource URI contains NUL".to_owned())?,
            bytes,
        }))
    }
}

static BROKER: OnceLock<Mutex<Broker>> = OnceLock::new();

unsafe extern "C" fn resource_loader(
    url: *const c_char,
    public_id: *const c_char,
    context: *mut XmlParserCtxt,
) -> *mut XmlParserInput {
    let Some(url) = c_string(url) else {
        return ptr::null_mut();
    };
    let public_id = c_string(public_id);
    let Some(broker) = BROKER.get() else {
        return ptr::null_mut();
    };
    let Ok(mut broker) = broker.lock() else {
        return ptr::null_mut();
    };
    let Ok(Some(resource)) = broker.request(&url, public_id.as_deref()) else {
        return ptr::null_mut();
    };
    let Ok(size) = c_int::try_from(resource.bytes.len()) else {
        return ptr::null_mut();
    };
    let buffer = unsafe {
        xmlParserInputBufferCreateMem(resource.bytes.as_ptr().cast(), size, XML_CHAR_ENCODING_NONE)
    };
    if buffer.is_null() {
        return ptr::null_mut();
    }
    let input = unsafe { xmlNewIOInputStream(context, buffer, XML_CHAR_ENCODING_NONE) };
    if input.is_null() {
        unsafe { xmlFreeParserInputBuffer(buffer) };
        return ptr::null_mut();
    }
    let filename = unsafe { xmlStrdup(resource.uri.as_ptr().cast()) };
    if filename.is_null() {
        return ptr::null_mut();
    }
    unsafe {
        (*input.cast::<ParserInputPrefix>()).filename = filename.cast();
    }
    input
}

fn c_string(pointer: *const c_char) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(pointer) }
            .to_string_lossy()
            .into_owned(),
    )
}

fn main() -> ExitCode {
    match run() {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(3),
        Err(error) => {
            eprintln!("validation helper error: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<bool, String> {
    let mut broker = Broker {
        reader: BufReader::new(io::stdin()),
        writer: io::stdout(),
    };
    let mut header = String::new();
    broker
        .reader
        .read_line(&mut header)
        .map_err(|error| error.to_string())?;
    let request: Request =
        serde_json::from_str(header.trim_end()).map_err(|error| error.to_string())?;
    let mut document = vec![0_u8; request.document_bytes];
    broker
        .reader
        .read_exact(&mut document)
        .map_err(|error| error.to_string())?;
    let mut delimiter = [0_u8; 1];
    broker
        .reader
        .read_exact(&mut delimiter)
        .map_err(|error| error.to_string())?;
    if delimiter != *b"\n" {
        return Err("document frame is missing its delimiter".into());
    }
    BROKER
        .set(Mutex::new(broker))
        .map_err(|_| "resource broker was already initialized".to_owned())?;

    let document_url = CString::new("document.xml").map_err(|error| error.to_string())?;
    let document_size = c_int::try_from(document.len())
        .map_err(|_| "document is too large for libxml2".to_owned())?;
    let grammar_uri =
        CString::new(request.grammar_uri).map_err(|_| "grammar URI contains NUL".to_owned())?;
    let public_id = request
        .public_id
        .map(CString::new)
        .transpose()
        .map_err(|_| "public identifier contains NUL".to_owned())?;

    unsafe {
        xmlInitParser();
        xmlSetExternalEntityLoader(Some(resource_loader));
    }
    let document_ptr = unsafe {
        xmlReadMemory(
            document.as_ptr().cast(),
            document_size,
            document_url.as_ptr(),
            ptr::null(),
            XML_PARSE_NONET,
        )
    };
    if document_ptr.is_null() {
        emit_result(false)?;
        return Ok(false);
    }

    let valid = unsafe {
        match request.mode {
            Mode::Dtd => validate_dtd(document_ptr, &grammar_uri, public_id.as_ref()),
            Mode::Xsd => validate_xsd(document_ptr, &grammar_uri),
        }
    };
    unsafe { xmlFreeDoc(document_ptr) };
    let valid = valid?;
    emit_result(valid)?;
    Ok(valid)
}

unsafe fn validate_dtd(
    document: *mut XmlDoc,
    grammar_uri: &CString,
    public_id: Option<&CString>,
) -> Result<bool, String> {
    let public_id = public_id.map_or(ptr::null(), |value| value.as_ptr().cast());
    let dtd = unsafe { xmlParseDTD(public_id, grammar_uri.as_ptr().cast()) };
    if dtd.is_null() {
        return Ok(false);
    }
    let context = unsafe { xmlNewValidCtxt() };
    if context.is_null() {
        unsafe { xmlFreeDtd(dtd) };
        return Err("cannot allocate DTD validation context".into());
    }
    let result = unsafe { xmlValidateDtd(context, document, dtd) };
    unsafe {
        xmlFreeValidCtxt(context);
        xmlFreeDtd(dtd);
    }
    Ok(result == 1)
}

unsafe fn validate_xsd(document: *mut XmlDoc, grammar_uri: &CString) -> Result<bool, String> {
    let parser = unsafe { xmlSchemaNewParserCtxt(grammar_uri.as_ptr()) };
    if parser.is_null() {
        return Err("cannot allocate XSD parser context".into());
    }
    let schema = unsafe { xmlSchemaParse(parser) };
    unsafe { xmlSchemaFreeParserCtxt(parser) };
    if schema.is_null() {
        return Ok(false);
    }
    let context = unsafe { xmlSchemaNewValidCtxt(schema) };
    if context.is_null() {
        unsafe { xmlSchemaFree(schema) };
        return Err("cannot allocate XSD validation context".into());
    }
    let result = unsafe { xmlSchemaValidateDoc(context, document) };
    unsafe {
        xmlSchemaFreeValidCtxt(context);
        xmlSchemaFree(schema);
    }
    Ok(result == 0)
}

fn emit_result(valid: bool) -> Result<(), String> {
    let Some(broker) = BROKER.get() else {
        return Err("resource broker is unavailable".into());
    };
    let mut broker = broker
        .lock()
        .map_err(|_| "resource broker failed".to_owned())?;
    serde_json::to_writer(&mut broker.writer, &Message::Result { valid })
        .map_err(|error| error.to_string())?;
    broker
        .writer
        .write_all(b"\n")
        .and_then(|()| broker.writer.flush())
        .map_err(|error| error.to_string())
}
