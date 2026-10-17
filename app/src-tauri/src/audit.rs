use xml_domain::Revision;

pub fn event(
    action: &'static str,
    result: &'static str,
    resource: &'static str,
    resource_id: u64,
    revision: Option<Revision>,
    engine: Option<&'static str>,
) {
    tracing::info!(
        audit = true,
        action,
        result,
        resource,
        resource_id,
        revision = revision.map(|value| value.0),
        engine,
    );
}
