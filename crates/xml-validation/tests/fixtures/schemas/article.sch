<schema xmlns="http://purl.oclc.org/dsdl/schematron">
  <pattern id="article-metadata">
    <rule context="article">
      <assert id="article-type" role="error" test="@article-type">article-type is required</assert>
    </rule>
  </pattern>
</schema>
