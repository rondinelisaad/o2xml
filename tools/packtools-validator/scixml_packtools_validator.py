#!/usr/bin/env python3
"""Offline SciXML adapter for the Packtools 4.18.3 SPS validator.

The executable built from this module receives one JSON request on stdin and
returns one JSON response on stdout. It deliberately exposes only the SPS 1.10
content-validation API; JATS grammar validation remains an independent SciXML
stage.
"""

from __future__ import annotations

import json
import os
import re
import socket
import sys
from typing import Any, Iterable

from lxml import etree
from packtools.sps import i18n
from packtools.sps.pid_provider.xml_sps_lib import XMLWithPre
from packtools.sps.validation.xml_validator import get_validation_results
from packtools.version import __version__ as PACKTOOLS_VERSION


PROTOCOL_VERSION = 1
EXPECTED_PACKTOOLS_VERSION = "4.18.3"
MAX_REQUEST_BYTES = 100 * 1024 * 1024
MAX_DIAGNOSTICS = 5_000
MAX_MESSAGE_CHARS = 4_000
XML_NAME = re.compile(r"^[A-Za-z_][A-Za-z0-9_.:-]*$")


class ProtocolError(Exception):
    """A bounded, user-safe helper protocol failure."""


def _deny_network(*_args: Any, **_kwargs: Any) -> Any:
    raise PermissionError("network access is disabled for SPS validation")


def _disable_network() -> None:
    """Deny accidental requests made by validation callbacks or dependencies."""

    socket.create_connection = _deny_network  # type: ignore[assignment]
    socket.getaddrinfo = _deny_network  # type: ignore[assignment]
    os.environ["NO_PROXY"] = "*"
    os.environ["no_proxy"] = "*"
    os.environ["HTTP_PROXY"] = "http://127.0.0.1:9"
    os.environ["HTTPS_PROXY"] = "http://127.0.0.1:9"


def _read_request() -> dict[str, Any]:
    payload = sys.stdin.buffer.read(MAX_REQUEST_BYTES + 1)
    if len(payload) > MAX_REQUEST_BYTES:
        raise ProtocolError("request exceeds the 100 MiB validation limit")
    try:
        request = json.loads(payload)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ProtocolError("request is not valid UTF-8 JSON") from error
    if not isinstance(request, dict):
        raise ProtocolError("request must be a JSON object")
    if request.get("protocol") != PROTOCOL_VERSION:
        raise ProtocolError("unsupported helper protocol")
    source = request.get("source")
    if not isinstance(source, str) or not source.strip():
        raise ProtocolError("source must be a non-empty XML string")
    if request.get("sps_version") != "1.10":
        raise ProtocolError("only SPS 1.10 is supported by this helper")
    return request


def _format_template(template: Any, parameters: Any) -> str | None:
    if not isinstance(template, str) or not template.strip():
        return None
    if not isinstance(parameters, dict):
        parameters = {}
    try:
        return template.format(**parameters).strip()
    except (KeyError, IndexError, ValueError):
        return template.strip()


def _message(result: dict[str, Any]) -> str:
    message = _format_template(result.get("msg_text"), result.get("msg_params"))
    if not message:
        raw = result.get("message")
        message = raw.strip() if isinstance(raw, str) and raw.strip() else "SPS validation failed"
    advice = _format_template(result.get("adv_text"), result.get("adv_params"))
    if not advice:
        raw_advice = result.get("advice")
        advice = raw_advice.strip() if isinstance(raw_advice, str) and raw_advice.strip() else None
    if advice and advice not in message:
        message = f"{message}. {advice}"
    return " ".join(message.split())[:MAX_MESSAGE_CHARS]


def _xml_name(value: Any) -> str | None:
    if not isinstance(value, str):
        return None
    candidate = value.strip().removeprefix("@").split("=", 1)[0]
    return candidate if XML_NAME.fullmatch(candidate) else None


def _string_values(value: Any) -> Iterable[str]:
    if isinstance(value, str):
        cleaned = " ".join(value.split())
        if 2 <= len(cleaned) <= 256:
            yield cleaned
    elif isinstance(value, dict):
        for item in value.values():
            yield from _string_values(item)
    elif isinstance(value, list):
        for item in value:
            yield from _string_values(item)


def _context(root: etree._Element, result: dict[str, Any]) -> etree._Element:
    parent_id = result.get("parent_id")
    if isinstance(parent_id, str) and parent_id:
        matches = root.xpath(".//*[@id=$identifier]", identifier=parent_id)
        if matches:
            return matches[0]
    if result.get("parent") == "sub-article":
        matches = root.xpath(".//sub-article")
        if matches:
            return matches[0]
    return root


def _candidate_nodes(
    root: etree._Element, result: dict[str, Any]
) -> list[etree._Element]:
    context = _context(root, result)
    item = _xml_name(result.get("item"))
    sub_item_raw = result.get("sub_item")
    sub_item = _xml_name(sub_item_raw)
    attribute = (
        sub_item if isinstance(sub_item_raw, str) and sub_item_raw.strip().startswith("@") else None
    )

    expressions: list[str] = []
    if attribute and item:
        expressions.extend((f".//{item}[@{attribute}]", f".//{item}"))
    elif attribute:
        expressions.append(f".//*[@{attribute}]")
    elif sub_item:
        expressions.append(f".//{sub_item}")
    if item:
        expressions.append(f".//{item}")

    nodes: list[etree._Element] = []
    seen: set[int] = set()
    for expression in expressions:
        try:
            matches = context.xpath(expression)
        except etree.XPathError:
            continue
        for match in matches:
            if not isinstance(match, etree._Element) or id(match) in seen:
                continue
            seen.add(id(match))
            nodes.append(match)
    return nodes or [context]


def _locate(root: etree._Element, result: dict[str, Any]) -> tuple[int, int, bool]:
    candidates = _candidate_nodes(root, result)
    evidence = set(_string_values(result.get("data")))
    evidence.update(_string_values(result.get("got_value")))
    attribute = _xml_name(result.get("sub_item"))

    def score(node: etree._Element) -> tuple[int, int]:
        node_text = " ".join("".join(node.itertext()).split())[:2_000]
        serialized = etree.tostring(node, encoding="unicode")[:4_000]
        matches = sum(1 for value in evidence if value in node_text or value in serialized)
        if attribute and node.get(attribute) is not None:
            matches += 3
        return matches, -(node.sourceline or 1)

    node = max(candidates, key=score)
    has_specific_name = bool(
        _xml_name(result.get("item")) or _xml_name(result.get("sub_item"))
    )
    return (
        max(node.sourceline or 1, 1),
        1,
        not has_specific_name or len(candidates) != 1,
    )


def _severity(level: Any) -> str:
    normalized = str(level or "").upper()
    return "error" if normalized in {"CRITICAL", "ERROR"} else "warning"


def _diagnostic(root: etree._Element, result: dict[str, Any]) -> dict[str, Any]:
    line, column, approximate = _locate(root, result)
    group = str(result.get("group") or "sps")[:120]
    title = str(result.get("title") or result.get("validation_type") or "rule")[:160]
    level = str(result.get("response") or "WARNING").upper()
    return {
        "severity": _severity(level),
        "level": level,
        "message": _message(result),
        "rule": f"packtools:{group}:{title}",
        "line": line,
        "column": column,
        "approximate": approximate,
    }


def _validate(request: dict[str, Any]) -> dict[str, Any]:
    if PACKTOOLS_VERSION != EXPECTED_PACKTOOLS_VERSION:
        raise ProtocolError(
            f"expected Packtools {EXPECTED_PACKTOOLS_VERSION}, found {PACKTOOLS_VERSION}"
        )
    locale = request.get("locale")
    i18n.set_locale(locale if locale in {"pt_BR", "en", "es"} else "pt_BR")
    source = request["source"]
    try:
        document = next(XMLWithPre.create(xml_content=source))
    except Exception as error:
        raise ProtocolError(f"Packtools could not parse the XML: {error}") from error

    diagnostics: list[dict[str, Any]] = []
    engine_errors: list[str] = []
    truncated = False
    for result in get_validation_results(document.xmltree, {}):
        if not isinstance(result, dict):
            continue
        response = result.get("response")
        if response == "OK" or response is None:
            continue
        if response == "exception":
            group = str(result.get("group") or "unknown")[:120]
            detail = str(result.get("error") or "unknown Packtools exception")[:500]
            engine_errors.append(f"{group}: {detail}")
            continue
        if len(diagnostics) >= MAX_DIAGNOSTICS:
            truncated = True
            continue
        diagnostics.append(_diagnostic(document.xmltree, result))

    return {
        "protocol": PROTOCOL_VERSION,
        "packtools_version": PACKTOOLS_VERSION,
        "diagnostics": diagnostics,
        "engine_errors": engine_errors[:100],
        "truncated": truncated,
    }


def main() -> int:
    _disable_network()
    try:
        response = _validate(_read_request())
        json.dump(response, sys.stdout, ensure_ascii=False, separators=(",", ":"))
        sys.stdout.write("\n")
        return 0
    except ProtocolError as error:
        json.dump(
            {
                "protocol": PROTOCOL_VERSION,
                "packtools_version": PACKTOOLS_VERSION,
                "error": str(error)[:1_000],
            },
            sys.stdout,
            ensure_ascii=False,
            separators=(",", ":"),
        )
        sys.stdout.write("\n")
        return 2
    except Exception as error:  # defensive process boundary
        json.dump(
            {
                "protocol": PROTOCOL_VERSION,
                "packtools_version": PACKTOOLS_VERSION,
                "error": f"unexpected Packtools failure: {type(error).__name__}: {error}"[:1_000],
            },
            sys.stdout,
            ensure_ascii=False,
            separators=(",", ":"),
        )
        sys.stdout.write("\n")
        return 3


if __name__ == "__main__":
    raise SystemExit(main())
