use std::{
    collections::{BTreeMap, BTreeSet, HashSet},
    ffi::{c_char, c_int, c_long, c_uint, c_void, CStr, CString},
    io::{self, BufRead, BufReader, Read, Write},
    process::ExitCode,
    ptr,
    sync::{Mutex, OnceLock},
};

use serde::{Deserialize, Serialize};
use xml_dtd::{DtdContent, DtdElementDeclaration, DtdParticle};
use xml_schema::ContentParticle;
use xml_schema::{AttributeDeclaration, AttributeUse, ExpandedName, ValueConstraint};
use xml_xsd::{XsdContent, XsdElementDeclaration};

const XML_PARSE_NONET: c_int = 1 << 11;
const XML_CHAR_ENCODING_NONE: c_int = 0;

type XmlDoc = c_void;
type XmlDtd = c_void;
type XmlHashTable = c_void;
type XmlParserCtxt = c_void;
type XmlParserInput = c_void;
type XmlParserInputBuffer = c_void;
type XmlSchema = c_void;
type XmlSchemaParserCtxt = c_void;
type XmlSchemaValidCtxt = c_void;
type XmlValidCtxt = c_void;
type XmlXPathContext = c_void;
type XmlNode = c_void;
type XmlNs = c_void;

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

#[repr(C)]
struct XmlDtdLayout {
    private_data: *mut c_void,
    node_type: c_int,
    name: *const u8,
    children: *mut c_void,
    last: *mut c_void,
    parent: *mut c_void,
    next: *mut c_void,
    previous: *mut c_void,
    document: *mut c_void,
    notations: *mut c_void,
    elements: *mut XmlHashTable,
    attributes: *mut XmlHashTable,
    entities: *mut XmlHashTable,
    external_id: *const u8,
    system_id: *const u8,
    parameter_entities: *mut XmlHashTable,
}

#[repr(C)]
struct XmlSchemaLayout {
    name: *const u8,
    target_namespace: *const u8,
    version: *const u8,
    id: *const u8,
    document: *mut c_void,
    annotation: *mut c_void,
    flags: c_int,
    type_declarations: *mut XmlHashTable,
    attribute_declarations: *mut XmlHashTable,
    attribute_group_declarations: *mut XmlHashTable,
    element_declarations: *mut XmlHashTable,
}

#[repr(C)]
struct XmlSchemaElementLayout {
    component_type: c_int,
    next: *mut c_void,
    name: *const u8,
    id: *const u8,
    reference: *const u8,
    reference_namespace: *const u8,
    annotation: *mut c_void,
    subtype: *mut c_void,
    attributes: *mut c_void,
    node: *mut c_void,
    min_occurs: c_int,
    max_occurs: c_int,
    flags: c_int,
    target_namespace: *const u8,
    named_type: *const u8,
    named_type_namespace: *const u8,
    substitution_group: *const u8,
    substitution_group_namespace: *const u8,
    scope: *const u8,
    value: *const u8,
    reference_declaration: *mut c_void,
    content_model: *mut c_void,
    content_type: c_int,
}

#[repr(C)]
struct XmlSchemaTypeLayout {
    component_type: c_int,
    next: *mut c_void,
    name: *const u8,
    id: *const u8,
    reference: *const u8,
    reference_namespace: *const u8,
    annotation: *mut c_void,
    subtypes: *mut c_void,
    attributes: *mut c_void,
    node: *mut XmlNode,
}

#[repr(C)]
struct XmlNodeLayout {
    private_data: *mut c_void,
    node_type: c_int,
    name: *const u8,
    children: *mut XmlNode,
    last: *mut XmlNode,
    parent: *mut XmlNode,
    next: *mut XmlNode,
    previous: *mut XmlNode,
    document: *mut XmlDoc,
    namespace: *mut XmlNs,
}

#[repr(C)]
struct XmlNsLayout {
    next: *mut XmlNs,
    node_type: c_int,
    href: *const u8,
    prefix: *const u8,
}

#[repr(C)]
struct XmlElementLayout {
    private_data: *mut c_void,
    node_type: c_int,
    name: *const u8,
    children: *mut c_void,
    last: *mut c_void,
    parent: *mut c_void,
    next: *mut c_void,
    previous: *mut c_void,
    document: *mut c_void,
    element_type: c_int,
    content: *mut XmlElementContentLayout,
    attributes: *mut XmlAttributeLayout,
    prefix: *const u8,
    content_model: *mut c_void,
}

#[repr(C)]
struct XmlElementContentLayout {
    content_type: c_int,
    occurrence: c_int,
    name: *const u8,
    first: *mut XmlElementContentLayout,
    second: *mut XmlElementContentLayout,
    parent: *mut XmlElementContentLayout,
    prefix: *const u8,
}

#[repr(C)]
struct XmlAttributeLayout {
    private_data: *mut c_void,
    node_type: c_int,
    name: *const u8,
    children: *mut c_void,
    last: *mut c_void,
    parent: *mut c_void,
    next: *mut XmlAttributeLayout,
    previous: *mut XmlAttributeLayout,
    document: *mut c_void,
    next_in_hash: *mut XmlAttributeLayout,
    attribute_type: c_int,
    default_kind: c_int,
    default_value: *const u8,
    enumeration: *mut XmlEnumerationLayout,
    prefix: *const u8,
    element: *const u8,
}

#[repr(C)]
struct XmlEnumerationLayout {
    next: *mut XmlEnumerationLayout,
    name: *const u8,
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
    fn xmlHashScan(
        table: *mut XmlHashTable,
        scanner: Option<unsafe extern "C" fn(*mut c_void, *mut c_void, *const u8)>,
        data: *mut c_void,
    );
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
    fn xmlGetProp(node: *const XmlNode, name: *const u8) -> *mut u8;
    fn xmlSearchNs(document: *mut XmlDoc, node: *mut XmlNode, prefix: *const u8) -> *mut XmlNs;
    fn xmlDocGetRootElement(document: *const XmlDoc) -> *mut XmlNode;
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
    #[serde(rename = "dtd_index")]
    DtdIndex,
    Xsd,
    #[serde(rename = "xsd_index")]
    XsdIndex,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        dtd: Option<&'a [DtdElementDeclaration]>,
        #[serde(skip_serializing_if = "Option::is_none")]
        xsd: Option<&'a [XsdElementDeclaration]>,
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
            Mode::Dtd | Mode::DtdIndex | Mode::Xsd | Mode::XsdIndex => resource_loader,
        }));
    }

    if matches!(request.mode, Mode::DtdIndex) {
        let declarations = unsafe { extract_dtd(&grammar_uri, public_id.as_ref()) }?;
        let valid = declarations.is_some();
        emit_result(valid, None, declarations.as_deref(), None)?;
        return Ok(valid);
    }

    if matches!(request.mode, Mode::XsdIndex) {
        let declarations = unsafe { extract_xsd(&grammar_uri) }?;
        let valid = declarations.is_some();
        emit_result(valid, None, None, declarations.as_deref())?;
        return Ok(valid);
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
        emit_result(false, None, None, None)?;
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
        emit_result(true, Some(&value), None, None)?;
        return Ok(true);
    }

    let valid = unsafe {
        match request.mode {
            Mode::Dtd => validate_dtd(document_ptr, &grammar_uri, public_id.as_ref()),
            Mode::Xsd => validate_xsd(document_ptr, &grammar_uri),
            Mode::DtdIndex => Err("DTD index request reached the validation path".into()),
            Mode::XsdIndex => Err("XSD index request reached the validation path".into()),
            Mode::XPath => Err("XPath request reached the validation path".into()),
        }
    };
    unsafe { xmlFreeDoc(document_ptr) };
    let valid = valid?;
    emit_result(valid, None, None, None)?;
    Ok(valid)
}

struct XsdExtraction {
    declarations: Vec<XsdElementDeclaration>,
    error: Option<String>,
}

unsafe fn extract_xsd(grammar_uri: &CString) -> Result<Option<Vec<XsdElementDeclaration>>, String> {
    let parser = unsafe { xmlSchemaNewParserCtxt(grammar_uri.as_ptr()) };
    if parser.is_null() {
        return Err("cannot allocate XSD parser context".into());
    }
    let schema = unsafe { xmlSchemaParse(parser) };
    unsafe { xmlSchemaFreeParserCtxt(parser) };
    if schema.is_null() {
        return Ok(None);
    }
    let elements = unsafe { (*schema.cast::<XmlSchemaLayout>()).element_declarations };
    let mut extraction = XsdExtraction {
        declarations: Vec::new(),
        error: None,
    };
    if !elements.is_null() {
        unsafe {
            xmlHashScan(
                elements,
                Some(scan_xsd_element),
                (&mut extraction as *mut XsdExtraction).cast(),
            )
        };
    }
    unsafe { xmlSchemaFree(schema) };
    if let Some(error) = extraction.error {
        return Err(error);
    }
    extraction
        .declarations
        .sort_by(|left, right| left.name.cmp(&right.name));
    Ok(Some(extraction.declarations))
}

unsafe extern "C" fn scan_xsd_element(payload: *mut c_void, data: *mut c_void, _name: *const u8) {
    if payload.is_null() || data.is_null() {
        return;
    }
    let extraction = unsafe { &mut *data.cast::<XsdExtraction>() };
    if extraction.error.is_some() {
        return;
    }
    if extraction.declarations.len() >= 100_000 {
        extraction.error = Some("XSD exceeds the 100000-element extraction limit".into());
        return;
    }
    let element = unsafe { &*payload.cast::<XmlSchemaElementLayout>() };
    let Some(local_name) = (unsafe { optional_xml_string(element.name) }) else {
        extraction.error = Some("XSD element declaration has no name".into());
        return;
    };
    let namespace_uri = unsafe { optional_xml_string(element.target_namespace) };
    let name = match ExpandedName::new(namespace_uri.as_deref(), &local_name) {
        Ok(name) => name,
        Err(error) => {
            extraction.error = Some(error.to_string());
            return;
        }
    };
    let type_node = if element.subtype.is_null() {
        ptr::null_mut()
    } else {
        unsafe { (*element.subtype.cast::<XmlSchemaTypeLayout>()).node }
    };
    let content = match unsafe { xsd_type_particle(type_node, 0) } {
        Ok(Some(particle)) => XsdContent::Particle(particle),
        Ok(None) if matches!(element.content_type, 1 | 4 | 6) => XsdContent::Empty,
        Ok(None) => XsdContent::Any,
        Err(error) => {
            extraction.error = Some(error);
            return;
        }
    };
    let attributes = match unsafe { xsd_type_attributes(type_node) } {
        Ok(attributes) => attributes,
        Err(error) => {
            extraction.error = Some(error);
            return;
        }
    };
    extraction.declarations.push(XsdElementDeclaration {
        name,
        content,
        attributes,
    });
}

unsafe fn xsd_type_particle(
    type_node: *mut XmlNode,
    depth: usize,
) -> Result<Option<ContentParticle>, String> {
    if type_node.is_null() {
        return Ok(None);
    }
    if depth > 128 {
        return Err("XSD content particle exceeds the depth limit".into());
    }
    for child in unsafe { xsd_children(type_node) }? {
        match unsafe { node_local_name(child) }.as_deref() {
            Some("sequence" | "choice") if unsafe { is_xsd_node(child) } => {
                return unsafe { xsd_particle(child, depth + 1) };
            }
            Some("complexContent" | "simpleContent") if unsafe { is_xsd_node(child) } => {
                for derivation in unsafe { xsd_children(child) }? {
                    if matches!(
                        unsafe { node_local_name(derivation) }.as_deref(),
                        Some("extension" | "restriction")
                    ) && unsafe { is_xsd_node(derivation) }
                    {
                        if let Some(particle) = unsafe { xsd_type_particle(derivation, depth + 1) }?
                        {
                            return Ok(Some(particle));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(None)
}

unsafe fn xsd_particle(
    node: *mut XmlNode,
    depth: usize,
) -> Result<Option<ContentParticle>, String> {
    if depth > 128 {
        return Err("XSD content particle exceeds the depth limit".into());
    }
    let local = unsafe { node_local_name(node) }
        .ok_or_else(|| "XSD content particle has no name".to_owned())?;
    let particle = match local.as_str() {
        "element" => ContentParticle::Element(unsafe { xsd_element_name(node) }?),
        "sequence" | "choice" => {
            let mut particles = Vec::new();
            for child in unsafe { xsd_children(node) }? {
                if !unsafe { is_xsd_node(child) } {
                    continue;
                }
                let child_name = unsafe { node_local_name(child) };
                if matches!(
                    child_name.as_deref(),
                    Some("element" | "sequence" | "choice")
                ) {
                    if let Some(particle) = unsafe { xsd_particle(child, depth + 1) }? {
                        particles.push(particle);
                    }
                } else if matches!(child_name.as_deref(), Some("any" | "group" | "all")) {
                    return Ok(None);
                }
            }
            if local == "sequence" {
                ContentParticle::Sequence(particles)
            } else {
                if particles.is_empty() {
                    return Ok(None);
                }
                ContentParticle::Choice(particles)
            }
        }
        _ => return Ok(None),
    };
    let min_occurs = unsafe { node_property(node, "minOccurs") }?
        .map(|value| parse_occurs(&value))
        .transpose()?
        .unwrap_or(1);
    let max_occurs = match unsafe { node_property(node, "maxOccurs") }? {
        Some(value) if value == "unbounded" => None,
        Some(value) => Some(parse_occurs(&value)?),
        None => Some(1),
    };
    if min_occurs == 1 && max_occurs == Some(1) {
        Ok(Some(particle))
    } else {
        Ok(Some(ContentParticle::Repeat {
            particle: Box::new(particle),
            min_occurs,
            max_occurs,
        }))
    }
}

fn parse_occurs(value: &str) -> Result<u32, String> {
    value
        .parse::<u32>()
        .map_err(|_| "XSD occurrence is not a valid non-negative integer".to_owned())
}

unsafe fn xsd_element_name(node: *mut XmlNode) -> Result<ExpandedName, String> {
    if let Some(reference) = unsafe { node_property(node, "ref") }? {
        return unsafe { resolve_qname(node, &reference) };
    }
    let local = unsafe { node_property(node, "name") }?
        .ok_or_else(|| "local XSD element has neither name nor ref".to_owned())?;
    let qualified = unsafe { node_property(node, "form") }?.is_some_and(|form| form == "qualified")
        || (unsafe { node_property(node, "form") }?.is_none()
            && unsafe { schema_property(node, "elementFormDefault") }?
                .is_some_and(|form| form == "qualified"));
    let namespace = if qualified {
        unsafe { schema_property(node, "targetNamespace") }?
    } else {
        None
    };
    ExpandedName::new(namespace.as_deref(), &local).map_err(|error| error.to_string())
}

unsafe fn xsd_type_attributes(
    type_node: *mut XmlNode,
) -> Result<Vec<AttributeDeclaration>, String> {
    let mut attributes = Vec::new();
    if type_node.is_null() {
        return Ok(attributes);
    }
    unsafe { collect_xsd_attributes(type_node, &mut attributes, 0) }?;
    Ok(attributes)
}

unsafe fn collect_xsd_attributes(
    container: *mut XmlNode,
    output: &mut Vec<AttributeDeclaration>,
    depth: usize,
) -> Result<(), String> {
    if depth > 8 {
        return Err("XSD attribute container exceeds the depth limit".into());
    }
    for child in unsafe { xsd_children(container) }? {
        if !unsafe { is_xsd_node(child) } {
            continue;
        }
        match unsafe { node_local_name(child) }.as_deref() {
            Some("attribute") => {
                if output.len() >= 4_096 {
                    return Err("XSD type exceeds the 4096-attribute extraction limit".into());
                }
                output.push(unsafe { xsd_attribute(child) }?);
            }
            Some("complexContent" | "simpleContent" | "extension" | "restriction") => {
                unsafe { collect_xsd_attributes(child, output, depth + 1) }?;
            }
            _ => {}
        }
    }
    Ok(())
}

unsafe fn xsd_attribute(node: *mut XmlNode) -> Result<AttributeDeclaration, String> {
    let name = if let Some(reference) = unsafe { node_property(node, "ref") }? {
        unsafe { resolve_qname(node, &reference) }?
    } else {
        let local = unsafe { node_property(node, "name") }?
            .ok_or_else(|| "XSD attribute has neither name nor ref".to_owned())?;
        let qualified = unsafe { node_property(node, "form") }?
            .is_some_and(|form| form == "qualified")
            || (unsafe { node_property(node, "form") }?.is_none()
                && unsafe { schema_property(node, "attributeFormDefault") }?
                    .is_some_and(|form| form == "qualified"));
        let namespace = if qualified {
            unsafe { schema_property(node, "targetNamespace") }?
        } else {
            None
        };
        ExpandedName::new(namespace.as_deref(), &local).map_err(|error| error.to_string())?
    };
    let use_kind = match unsafe { node_property(node, "use") }?.as_deref() {
        Some("required") => AttributeUse::Required,
        Some("prohibited") => AttributeUse::Prohibited,
        _ => AttributeUse::Optional,
    };
    let value_constraint = if let Some(value) = unsafe { node_property(node, "fixed") }? {
        Some(ValueConstraint::Fixed(value))
    } else {
        unsafe { node_property(node, "default") }?.map(ValueConstraint::Default)
    };
    Ok(AttributeDeclaration {
        name,
        use_kind,
        allowed_values: Vec::new(),
        value_constraint,
    })
}

unsafe fn resolve_qname(node: *mut XmlNode, lexical: &str) -> Result<ExpandedName, String> {
    let (prefix, local) = lexical
        .split_once(':')
        .map_or((None, lexical), |(prefix, local)| (Some(prefix), local));
    let prefix = prefix
        .map(CString::new)
        .transpose()
        .map_err(|_| "XSD QName prefix contains NUL".to_owned())?;
    let layout = unsafe { &*node.cast::<XmlNodeLayout>() };
    let namespace = unsafe {
        xmlSearchNs(
            layout.document,
            node,
            prefix
                .as_ref()
                .map_or(ptr::null(), |value| value.as_ptr().cast()),
        )
    };
    let namespace_uri = if namespace.is_null() {
        if prefix.is_some() {
            return Err("XSD QName uses an unbound prefix".into());
        }
        None
    } else {
        unsafe { optional_xml_string((*namespace.cast::<XmlNsLayout>()).href) }
    };
    ExpandedName::new(namespace_uri.as_deref(), local).map_err(|error| error.to_string())
}

unsafe fn schema_property(node: *mut XmlNode, name: &str) -> Result<Option<String>, String> {
    let document = unsafe { (*node.cast::<XmlNodeLayout>()).document };
    if document.is_null() {
        return Err("XSD node has no document".into());
    }
    let root = unsafe { xmlDocGetRootElement(document) };
    if root.is_null() {
        return Err("XSD document has no root element".into());
    }
    unsafe { node_property(root, name) }
}

unsafe fn node_property(node: *mut XmlNode, name: &str) -> Result<Option<String>, String> {
    let name = CString::new(name).map_err(|_| "XSD property name contains NUL".to_owned())?;
    let value = unsafe { xmlGetProp(node, name.as_ptr().cast()) };
    if value.is_null() {
        return Ok(None);
    }
    let text = unsafe { CStr::from_ptr(value.cast()) }
        .to_string_lossy()
        .into_owned();
    unsafe { xmlMemFree(value.cast()) };
    Ok(Some(text))
}

unsafe fn xsd_children(node: *mut XmlNode) -> Result<Vec<*mut XmlNode>, String> {
    let mut output = Vec::new();
    let mut visited = HashSet::new();
    let mut child = unsafe { (*node.cast::<XmlNodeLayout>()).children };
    while !child.is_null() {
        if output.len() >= 100_000 {
            return Err("XSD node exceeds the 100000-child traversal limit".into());
        }
        if !visited.insert(child as usize) {
            return Err("XSD child list contains a cycle".into());
        }
        output.push(child);
        child = unsafe { (*child.cast::<XmlNodeLayout>()).next };
    }
    Ok(output)
}

unsafe fn node_local_name(node: *mut XmlNode) -> Option<String> {
    unsafe { optional_xml_string((*node.cast::<XmlNodeLayout>()).name) }
}

unsafe fn is_xsd_node(node: *mut XmlNode) -> bool {
    let namespace = unsafe { (*node.cast::<XmlNodeLayout>()).namespace };
    !namespace.is_null()
        && unsafe { optional_xml_string((*namespace.cast::<XmlNsLayout>()).href) }
            .is_some_and(|href| href == "http://www.w3.org/2001/XMLSchema")
}

struct DtdExtraction {
    declarations: Vec<DtdElementDeclaration>,
    error: Option<String>,
}

unsafe fn extract_dtd(
    grammar_uri: &CString,
    public_id: Option<&CString>,
) -> Result<Option<Vec<DtdElementDeclaration>>, String> {
    let public_id = public_id.map_or(ptr::null(), |value| value.as_ptr().cast());
    let dtd = unsafe { xmlParseDTD(public_id, grammar_uri.as_ptr().cast()) };
    if dtd.is_null() {
        return Ok(None);
    }
    let elements = unsafe { (*dtd.cast::<XmlDtdLayout>()).elements };
    let mut extraction = DtdExtraction {
        declarations: Vec::new(),
        error: None,
    };
    if !elements.is_null() {
        unsafe {
            xmlHashScan(
                elements,
                Some(scan_dtd_element),
                (&mut extraction as *mut DtdExtraction).cast(),
            )
        };
    }
    unsafe { xmlFreeDtd(dtd) };
    if let Some(error) = extraction.error {
        return Err(error);
    }
    extraction
        .declarations
        .sort_by(|left, right| left.name.cmp(&right.name));
    Ok(Some(extraction.declarations))
}

unsafe extern "C" fn scan_dtd_element(payload: *mut c_void, data: *mut c_void, _name: *const u8) {
    if payload.is_null() || data.is_null() {
        return;
    }
    let extraction = unsafe { &mut *data.cast::<DtdExtraction>() };
    if extraction.error.is_some() {
        return;
    }
    if extraction.declarations.len() >= 100_000 {
        extraction.error = Some("DTD exceeds the 100000-element extraction limit".into());
        return;
    }
    match unsafe { extract_dtd_element(&*payload.cast::<XmlElementLayout>()) } {
        Ok(declaration) => extraction.declarations.push(declaration),
        Err(error) => extraction.error = Some(error),
    }
}

unsafe fn extract_dtd_element(element: &XmlElementLayout) -> Result<DtdElementDeclaration, String> {
    let name = unsafe { expanded_lexical_name(element.name, element.prefix) }?;
    let content = match element.element_type {
        1 => DtdContent::Empty,
        2 => DtdContent::Any,
        3 => {
            let mut children = BTreeSet::new();
            if !element.content.is_null() {
                unsafe { collect_mixed_names(element.content, 0, &mut children) }?;
            }
            DtdContent::Mixed(children)
        }
        4 => {
            if element.content.is_null() {
                return Err("DTD children declaration has no content tree".into());
            }
            DtdContent::Children(unsafe { extract_particle(element.content, 0) }?)
        }
        value => return Err(format!("unsupported libxml2 DTD element type {value}")),
    };
    let attributes = unsafe { extract_attributes(element.attributes) }?;
    Ok(DtdElementDeclaration {
        name,
        content,
        attributes,
    })
}

unsafe fn collect_mixed_names(
    content: *mut XmlElementContentLayout,
    depth: usize,
    names: &mut BTreeSet<ExpandedName>,
) -> Result<(), String> {
    if content.is_null() || depth > 128 {
        return if content.is_null() {
            Ok(())
        } else {
            Err("DTD mixed content exceeds the depth limit".into())
        };
    }
    let node = unsafe { &*content };
    match node.content_type {
        1 => Ok(()),
        2 => {
            names.insert(unsafe { expanded_lexical_name(node.name, node.prefix) }?);
            Ok(())
        }
        3 | 4 => {
            unsafe { collect_mixed_names(node.first, depth + 1, names) }?;
            unsafe { collect_mixed_names(node.second, depth + 1, names) }
        }
        value => Err(format!("unsupported libxml2 mixed content type {value}")),
    }
}

unsafe fn extract_particle(
    content: *mut XmlElementContentLayout,
    depth: usize,
) -> Result<DtdParticle, String> {
    if content.is_null() {
        return Err("DTD content particle is null".into());
    }
    if depth > 128 {
        return Err("DTD content particle exceeds the depth limit".into());
    }
    let node = unsafe { &*content };
    let particle = match node.content_type {
        2 => DtdParticle::Element(unsafe { expanded_lexical_name(node.name, node.prefix) }?),
        3 => DtdParticle::Sequence(vec![
            unsafe { extract_particle(node.first, depth + 1) }?,
            unsafe { extract_particle(node.second, depth + 1) }?,
        ]),
        4 => DtdParticle::Choice(vec![
            unsafe { extract_particle(node.first, depth + 1) }?,
            unsafe { extract_particle(node.second, depth + 1) }?,
        ]),
        1 => return Err("PCDATA is only valid in a mixed DTD declaration".into()),
        value => return Err(format!("unsupported libxml2 DTD content type {value}")),
    };
    match node.occurrence {
        1 => Ok(particle),
        2 => Ok(DtdParticle::Optional(Box::new(particle))),
        3 => Ok(DtdParticle::ZeroOrMore(Box::new(particle))),
        4 => Ok(DtdParticle::OneOrMore(Box::new(particle))),
        value => Err(format!("unsupported libxml2 DTD occurrence {value}")),
    }
}

unsafe fn extract_attributes(
    mut attribute: *mut XmlAttributeLayout,
) -> Result<Vec<AttributeDeclaration>, String> {
    let mut attributes = Vec::new();
    let mut visited = HashSet::new();
    while !attribute.is_null() {
        if attributes.len() >= 4_096 {
            return Err("DTD element exceeds the 4096-attribute extraction limit".into());
        }
        if !visited.insert(attribute as usize) {
            return Err("DTD attribute list contains a cycle".into());
        }
        let item = unsafe { &*attribute };
        let name = unsafe { expanded_lexical_name(item.name, item.prefix) }?;
        let default_value = unsafe { optional_xml_string(item.default_value) };
        let use_kind = match item.default_kind {
            2 => AttributeUse::Required,
            1 | 3 | 4 => AttributeUse::Optional,
            value => return Err(format!("unsupported libxml2 attribute default {value}")),
        };
        let value_constraint = match (item.default_kind, default_value) {
            (4, Some(value)) => Some(ValueConstraint::Fixed(value)),
            (1, Some(value)) => Some(ValueConstraint::Default(value)),
            (4, None) => return Err("fixed DTD attribute has no value".into()),
            _ => None,
        };
        attributes.push(AttributeDeclaration {
            name,
            use_kind,
            allowed_values: unsafe { extract_enumeration(item.enumeration) }?,
            value_constraint,
        });
        attribute = item.next_in_hash;
    }
    Ok(attributes)
}

unsafe fn extract_enumeration(mut item: *mut XmlEnumerationLayout) -> Result<Vec<String>, String> {
    let mut values = Vec::new();
    let mut visited = HashSet::new();
    while !item.is_null() {
        if values.len() >= 4_096 {
            return Err("DTD attribute exceeds the 4096-value enumeration limit".into());
        }
        if !visited.insert(item as usize) {
            return Err("DTD attribute enumeration contains a cycle".into());
        }
        let value = unsafe { &*item };
        values.push(
            unsafe { optional_xml_string(value.name) }
                .ok_or_else(|| "DTD enumeration value is null".to_owned())?,
        );
        item = value.next;
    }
    Ok(values)
}

unsafe fn expanded_lexical_name(
    name: *const u8,
    prefix: *const u8,
) -> Result<ExpandedName, String> {
    let name = unsafe { optional_xml_string(name) }
        .ok_or_else(|| "DTD declaration name is null".to_owned())?;
    let local_name = match unsafe { optional_xml_string(prefix) } {
        Some(prefix) => format!("{prefix}:{name}"),
        None => name,
    };
    ExpandedName::new(None, &local_name).map_err(|error| error.to_string())
}

unsafe fn optional_xml_string(pointer: *const u8) -> Option<String> {
    if pointer.is_null() {
        return None;
    }
    Some(
        unsafe { CStr::from_ptr(pointer.cast()) }
            .to_string_lossy()
            .into_owned(),
    )
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

fn emit_result(
    valid: bool,
    xpath: Option<&XPathValue>,
    dtd: Option<&[DtdElementDeclaration]>,
    xsd: Option<&[XsdElementDeclaration]>,
) -> Result<(), String> {
    let Some(broker) = BROKER.get() else {
        return Err("resource broker is unavailable".into());
    };
    let mut broker = broker
        .lock()
        .map_err(|_| "resource broker failed".to_owned())?;
    serde_json::to_writer(
        &mut broker.writer,
        &Message::Result {
            valid,
            xpath,
            dtd,
            xsd,
        },
    )
    .map_err(|error| error.to_string())?;
    broker
        .writer
        .write_all(b"\n")
        .and_then(|()| broker.writer.flush())
        .map_err(|error| error.to_string())
}
