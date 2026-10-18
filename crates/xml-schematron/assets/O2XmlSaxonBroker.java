import java.nio.file.Path;
import javax.xml.XMLConstants;
import javax.xml.transform.Source;
import javax.xml.transform.Templates;
import javax.xml.transform.Transformer;
import javax.xml.transform.TransformerException;
import javax.xml.transform.TransformerFactory;
import javax.xml.transform.URIResolver;
import javax.xml.transform.stream.StreamResult;
import javax.xml.transform.stream.StreamSource;

final class O2XmlSaxonBroker {
    private static final String SAXON_FACTORY = "net.sf.saxon.TransformerFactoryImpl";
    private static final String SAXON_ALLOW_EXTERNAL_FUNCTIONS =
            "http://saxon.sf.net/feature/allow-external-functions";
    private static final String SAXON_ALLOWED_PROTOCOLS =
            "http://saxon.sf.net/feature/allowedProtocols";
    private static final URIResolver DENY_EXTERNAL_RESOURCES = (href, base) -> {
        throw new TransformerException("O2XML denied external resource resolution");
    };

    private O2XmlSaxonBroker() {}

    public static void main(String[] arguments) throws Exception {
        if (arguments.length != 3 && arguments.length != 5) {
            throw new IllegalArgumentException("expected source, stylesheet, output, and optional parameter");
        }

        TransformerFactory factory = TransformerFactory.newInstance(
                SAXON_FACTORY,
                O2XmlSaxonBroker.class.getClassLoader());
        factory.setFeature(XMLConstants.FEATURE_SECURE_PROCESSING, true);
        factory.setAttribute(SAXON_ALLOW_EXTERNAL_FUNCTIONS, false);
        factory.setAttribute(SAXON_ALLOWED_PROTOCOLS, "");
        setExternalAccess(factory, XMLConstants.ACCESS_EXTERNAL_DTD);
        setExternalAccess(factory, XMLConstants.ACCESS_EXTERNAL_STYLESHEET);
        factory.setURIResolver(DENY_EXTERNAL_RESOURCES);

        Source stylesheet = fileSource(arguments[1]);
        Templates templates = factory.newTemplates(stylesheet);
        Transformer transformer = templates.newTransformer();
        transformer.setURIResolver(DENY_EXTERNAL_RESOURCES);
        if (arguments.length == 5) {
            transformer.setParameter(arguments[3], arguments[4]);
        }
        transformer.transform(fileSource(arguments[0]), new StreamResult(Path.of(arguments[2]).toFile()));
    }

    private static Source fileSource(String value) {
        Path path = Path.of(value).toAbsolutePath().normalize();
        StreamSource source = new StreamSource(path.toFile());
        source.setSystemId(path.toUri().toString());
        return source;
    }

    private static void setExternalAccess(TransformerFactory factory, String name) {
        try {
            factory.setAttribute(name, "");
        } catch (IllegalArgumentException ignored) {
            // Saxon's URIResolver remains fail-closed when a JAXP implementation
            // does not expose the optional access-control attribute.
        }
    }
}
