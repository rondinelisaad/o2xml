#include <libxml/tree.h>
#include <libxml/valid.h>
#include <libxml/xmlerror.h>

#include <stdarg.h>
#include <stdio.h>
#include <string.h>

#define O2XML_DIAGNOSTIC_LIMIT 8192

static void o2xml_write_validity_message(void *context, const char *severity,
                                         const char *format, va_list arguments) {
    char message[O2XML_DIAGNOSTIC_LIMIT];
    xmlValidCtxtPtr validity_context = (xmlValidCtxtPtr)context;
    long line = 1;
    int written = vsnprintf(message, sizeof(message), format, arguments);

    if (written < 0) {
        strncpy(message, "DTD validation failed", sizeof(message));
        message[sizeof(message) - 1] = '\0';
    }

    for (size_t index = 0; message[index] != '\0'; index++) {
        if (message[index] == '\n' || message[index] == '\r' ||
            message[index] == '\t') {
            message[index] = ' ';
        }
    }

    if (validity_context != NULL && validity_context->node != NULL) {
        long node_line = xmlGetLineNo(validity_context->node);
        if (node_line > 0) {
            line = node_line;
        }
    }

    fprintf(stderr, "document.xml:%ld: validity %s : %s\n", line, severity,
            message);
}

static void o2xml_validity_error(void *context, const char *format, ...) {
    va_list arguments;
    va_start(arguments, format);
    o2xml_write_validity_message(context, "error", format, arguments);
    va_end(arguments);
}

static void o2xml_validity_warning(void *context, const char *format, ...) {
    va_list arguments;
    va_start(arguments, format);
    o2xml_write_validity_message(context, "warning", format, arguments);
    va_end(arguments);
}

static void o2xml_ignore_validity_message(void *context, const char *format, ...) {
    (void)context;
    (void)format;
}

static void o2xml_structured_error(void *context, xmlErrorPtr error) {
    char message[O2XML_DIAGNOSTIC_LIMIT];
    long line = 1;
    const char *severity = "error";

    (void)context;
    if (error == NULL) {
        return;
    }
    if (error->level == XML_ERR_WARNING) {
        severity = "warning";
    }
    if (error->line > 0) {
        line = error->line;
    } else if (error->node != NULL) {
        long node_line = xmlGetLineNo((xmlNodePtr)error->node);
        if (node_line > 0) {
            line = node_line;
        }
    }

    snprintf(message, sizeof(message), "%s",
             error->message != NULL ? error->message : "DTD validation failed");
    for (size_t index = 0; message[index] != '\0'; index++) {
        if (message[index] == '\n' || message[index] == '\r' ||
            message[index] == '\t') {
            message[index] = ' ';
        }
    }
    fprintf(stderr, "document.xml:%ld: validity %s : %s\n", line, severity,
            message);
}

static void o2xml_configure_dtd_valid_context(xmlValidCtxtPtr context) {
    if (context == NULL) {
        return;
    }
    context->userData = context;
    context->error = o2xml_validity_error;
    context->warning = o2xml_validity_warning;
    xmlSetStructuredErrorFunc(context, o2xml_structured_error);
}

static int o2xml_validate_element_tree(xmlValidCtxtPtr context, xmlDocPtr document,
                                       xmlNodePtr node) {
    int valid = 1;
    for (xmlNodePtr current = node; current != NULL; current = current->next) {
        if (current->type != XML_ELEMENT_NODE) {
            continue;
        }
        context->node = current;
        if (xmlValidateOneElement(context, document, current) != 1) {
            valid = 0;
        }
        for (xmlAttrPtr attribute = current->properties; attribute != NULL;
             attribute = attribute->next) {
            xmlChar *value = xmlNodeListGetString(document, attribute->children, 1);
            if (value == NULL) {
                continue;
            }
            context->node = current;
            if (xmlValidateOneAttribute(context, document, current, attribute,
                                        value) != 1) {
                valid = 0;
            }
            xmlFree(value);
        }
        if (o2xml_validate_element_tree(context, document, current->children) != 1) {
            valid = 0;
        }
    }
    return valid;
}

int o2xmlValidateDtdWithLocations(xmlDocPtr document, xmlDtdPtr dtd) {
    xmlValidCtxtPtr silent_context = NULL;
    xmlValidCtxtPtr detailed_context = NULL;
    xmlDtdPtr previous_external_subset = NULL;
    xmlDocPtr previous_dtd_document = NULL;
    int valid = 0;

    if (document == NULL || dtd == NULL) {
        return 0;
    }

    silent_context = xmlNewValidCtxt();
    if (silent_context == NULL) {
        return 0;
    }
    silent_context->error = o2xml_ignore_validity_message;
    silent_context->warning = o2xml_ignore_validity_message;
    valid = xmlValidateDtd(silent_context, document, dtd);
    xmlFreeValidCtxt(silent_context);

    previous_external_subset = document->extSubset;
    previous_dtd_document = dtd->doc;
    document->extSubset = dtd;
    dtd->doc = document;

    detailed_context = xmlNewValidCtxt();
    if (detailed_context != NULL) {
        o2xml_configure_dtd_valid_context(detailed_context);
        detailed_context->node = xmlDocGetRootElement(document);
        (void)xmlValidateRoot(detailed_context, document);
        (void)o2xml_validate_element_tree(detailed_context, document,
                                          xmlDocGetRootElement(document));
        xmlFreeValidCtxt(detailed_context);
    }

    document->extSubset = previous_external_subset;
    dtd->doc = previous_dtd_document;
    xmlSetStructuredErrorFunc(NULL, NULL);
    return valid;
}
