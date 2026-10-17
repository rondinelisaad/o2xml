use std::{
    collections::BTreeMap,
    ffi::{c_char, c_int, c_long, c_uint, c_void, CStr, CString},
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
type XmlXPathContext = c_void;

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

#[repr(C)]
struct XmlNodeSet {
    node_count: c_int,
    node_capacity: c_int,
    nodes: *mut *mut c_void,
}

#[repr(C)]
struct XmlXPathObject {
    object_type: c_uint,
    nodes: *mut XmlNodeSet,
    boolean: c_int,
    number: f64,
    string: *mut u8,
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
    fn xmlXPathNewContext(document: *mut XmlDoc) -> *mut XmlXPathContext;
    fn xmlXPathFreeContext(context: *mut XmlXPathContext);
    fn xmlXPathRegisterNs(
        context: *mut XmlXPathContext,
        prefix: *const u8,
        namespace_uri: *const u8,
    ) -> c_int;
    fn xmlXPathEvalExpression(
        expression: *const u8,
        context: *mut XmlXPathContext,
    ) -> *mut XmlXPathObject;
    fn xmlXPathFreeObject(object: *mut XmlXPathObject);
    fn xmlXPathCastToString(object: *mut XmlXPathObject) -> *mut u8;
    fn xmlGetNodePath(node: *const c_void) -> *mut u8;
    fn xmlGetLineNo(node: *const c_void) -> c_long;
    fn xmlMemFree(pointer: *mut c_void);
}

#[derive(Deserialize)]
struct Request {
    mode: Mode,
    grammar_uri: String,
    public_id: Option<String>,
    document_bytes: usize,
    expression: Option<String>,
    #[serde(default)]
    namespaces: BTreeMap<String, String>,
    #[serde(default = "default_max_results")]
    max_results: usize,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Mode {
    Dtd,
    Xsd,
    XPath,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        xpath: Option<&'a XPathValue>,
    },
}

#[derive(Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
enum XPathValue {
    Nodes {
        items: Vec<XPathNode>,
        truncated: bool,
    },
    Boolean(bool),
    Number(String),
    String(String),
}

#[derive(Serialize)]
struct XPathNode {
    path: String,
    line: usize,
}

const fn default_max_results() -> usize {
    10_000
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

unsafe extern "C" fn deny_resource_loader(
    _url: *const c_char,
    _public_id: *const c_char,
    _context: *mut XmlParserCtxt,
) -> *mut XmlParserInput {
    ptr::null_mut()
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
        xmlSetExternalEntityLoader(Some(match request.mode {
            Mode::XPath => deny_resource_loader,
            Mode::Dtd | Mode::Xsd => resource_loader,
        }));
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
        emit_result(false, None)?;
        return Ok(false);
    }

    if matches!(request.mode, Mode::XPath) {
        let expression = request
            .expression
            .ok_or_else(|| "XPath request has no expression".to_owned())?;
        let value = unsafe {
            evaluate_xpath(
                document_ptr,
                &expression,
                &request.namespaces,
                request.max_results,
            )
        };
        unsafe { xmlFreeDoc(document_ptr) };
        let value = value?;
        emit_result(true, Some(&value))?;
        return Ok(true);
    }

    let valid = unsafe {
        match request.mode {
            Mode::Dtd => validate_dtd(document_ptr, &grammar_uri, public_id.as_ref()),
            Mode::Xsd => validate_xsd(document_ptr, &grammar_uri),
            Mode::XPath => Err("XPath request reached the validation path".into()),
        }
    };
    unsafe { xmlFreeDoc(document_ptr) };
    let valid = valid?;
    emit_result(valid, None)?;
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

unsafe fn evaluate_xpath(
    document: *mut XmlDoc,
    expression: &str,
    namespaces: &BTreeMap<String, String>,
    max_results: usize,
) -> Result<XPathValue, String> {
    let expression = CString::new(expression).map_err(|_| "XPath expression contains NUL")?;
    let namespace_bindings = namespaces
        .iter()
        .map(|(prefix, namespace_uri)| {
            Ok((
                CString::new(prefix.as_str()).map_err(|_| "XPath prefix contains NUL")?,
                CString::new(namespace_uri.as_str()).map_err(|_| "namespace URI contains NUL")?,
            ))
        })
        .collect::<Result<Vec<_>, &str>>()?;
    let context = unsafe { xmlXPathNewContext(document) };
    if context.is_null() {
        return Err("cannot allocate XPath context".into());
    }
    for (prefix, namespace_uri) in &namespace_bindings {
        if unsafe {
            xmlXPathRegisterNs(
                context,
                prefix.as_ptr().cast(),
                namespace_uri.as_ptr().cast(),
            )
        } != 0
        {
            unsafe { xmlXPathFreeContext(context) };
            return Err("cannot register XPath namespace".into());
        }
    }
    let object = unsafe { xmlXPathEvalExpression(expression.as_ptr().cast(), context) };
    if object.is_null() {
        unsafe { xmlXPathFreeContext(context) };
        return Err("invalid XPath expression".into());
    }
    let value = unsafe { xpath_value(object, max_results) };
    unsafe {
        xmlXPathFreeObject(object);
        xmlXPathFreeContext(context);
    }
    value
}

unsafe fn xpath_value(
    object: *mut XmlXPathObject,
    max_results: usize,
) -> Result<XPathValue, String> {
    match unsafe { (*object).object_type } {
        1 => {
            let nodes = unsafe { (*object).nodes };
            if nodes.is_null() {
                return Ok(XPathValue::Nodes {
                    items: Vec::new(),
                    truncated: false,
                });
            }
            let count = usize::try_from(unsafe { (*nodes).node_count })
                .map_err(|_| "XPath returned an invalid node count")?;
            let table = unsafe { (*nodes).nodes };
            if count > 0 && table.is_null() {
                return Err("XPath returned a node set without storage".into());
            }
            let retained = count.min(max_results);
            let mut matches = Vec::with_capacity(retained);
            for index in 0..retained {
                let node = unsafe { *table.add(index) };
                if node.is_null() {
                    continue;
                }
                let path = unsafe { xmlGetNodePath(node) };
                if path.is_null() {
                    continue;
                }
                let path_text = unsafe { CStr::from_ptr(path.cast()) }
                    .to_string_lossy()
                    .into_owned();
                unsafe { xmlMemFree(path.cast()) };
                let line = usize::try_from(unsafe { xmlGetLineNo(node) }.max(1)).unwrap_or(1);
                matches.push(XPathNode {
                    path: path_text,
                    line,
                });
            }
            Ok(XPathValue::Nodes {
                items: matches,
                truncated: retained < count,
            })
        }
        2 => Ok(XPathValue::Boolean(unsafe { (*object).boolean } != 0)),
        3 => Ok(XPathValue::Number(unsafe { (*object).number }.to_string())),
        4 => {
            let value = unsafe { xmlXPathCastToString(object) };
            if value.is_null() {
                return Err("cannot convert XPath result to string".into());
            }
            let text = unsafe { CStr::from_ptr(value.cast()) }
                .to_string_lossy()
                .into_owned();
            unsafe { xmlMemFree(value.cast()) };
            Ok(XPathValue::String(text))
        }
        _ => Err("unsupported XPath result type".into()),
    }
}

fn emit_result(valid: bool, xpath: Option<&XPathValue>) -> Result<(), String> {
    let Some(broker) = BROKER.get() else {
        return Err("resource broker is unavailable".into());
    };
    let mut broker = broker
        .lock()
        .map_err(|_| "resource broker failed".to_owned())?;
    serde_json::to_writer(&mut broker.writer, &Message::Result { valid, xpath })
        .map_err(|error| error.to_string())?;
    broker
        .writer
        .write_all(b"\n")
        .and_then(|()| broker.writer.flush())
        .map_err(|error| error.to_string())
}
