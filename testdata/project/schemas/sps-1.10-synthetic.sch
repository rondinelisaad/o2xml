<schema xmlns="http://purl.oclc.org/dsdl/schematron">
  <pattern id="sps-identification">
    <rule context="article">
      <assert id="sps-specific-use" role="error" test="@specific-use = 'sps-1.10'">specific-use must identify SPS 1.10</assert>
    </rule>
  </pattern>
</schema>
