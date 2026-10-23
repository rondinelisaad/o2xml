<?xml version='1.0' encoding='UTF-8'?>
<!--This XSLT was automatically generated from a Schematron schema.--><axsl:stylesheet xmlns:date="http://exslt.org/dates-and-times" xmlns:dyn="http://exslt.org/dynamic" xmlns:exsl="http://exslt.org/common" xmlns:math="http://exslt.org/math" xmlns:random="http://exslt.org/random" xmlns:regexp="http://exslt.org/regular-expressions" xmlns:set="http://exslt.org/sets" xmlns:str="http://exslt.org/strings" xmlns:axsl="http://www.w3.org/1999/XSL/Transform" xmlns:sch="http://www.ascc.net/xml/schematron" xmlns:iso="http://purl.oclc.org/dsdl/schematron" xmlns:xlink="http://www.w3.org/1999/xlink" extension-element-prefixes="date dyn exsl math random regexp set str" version="1.0"><!--Implementers: please note that overriding process-prolog or process-root is 
    the preferred method for meta-stylesheets to use where possible. -->
<axsl:param name="archiveDirParameter"/><axsl:param name="archiveNameParameter"/><axsl:param name="fileNameParameter"/><axsl:param name="fileDirParameter"/>

<!--PHASES-->


<!--PROLOG-->
<axsl:output xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" xmlns:svrl="http://purl.oclc.org/dsdl/svrl" method="xml" omit-xml-declaration="no" standalone="yes" indent="yes"/>

<!--KEYS-->


<!--DEFAULT RULES-->


<!--MODE: SCHEMATRON-SELECT-FULL-PATH-->
<!--This mode can be used to generate an ugly though full XPath for locators-->
<axsl:template match="*" mode="schematron-select-full-path"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:template>

<!--MODE: SCHEMATRON-FULL-PATH-->
<!--This mode can be used to generate an ugly though full XPath for locators-->
<axsl:template match="*" mode="schematron-get-full-path"><axsl:apply-templates select="parent::*" mode="schematron-get-full-path"/><axsl:text>/</axsl:text><axsl:choose><axsl:when test="namespace-uri()=''"><axsl:value-of select="name()"/><axsl:variable name="p_1" select="1+    count(preceding-sibling::*[name()=name(current())])"/><axsl:if test="$p_1&gt;1 or following-sibling::*[name()=name(current())]">[<axsl:value-of select="$p_1"/>]</axsl:if></axsl:when><axsl:otherwise><axsl:text>*[local-name()='</axsl:text><axsl:value-of select="local-name()"/><axsl:text>' and namespace-uri()='</axsl:text><axsl:value-of select="namespace-uri()"/><axsl:text>']</axsl:text><axsl:variable name="p_2" select="1+   count(preceding-sibling::*[local-name()=local-name(current())])"/><axsl:if test="$p_2&gt;1 or following-sibling::*[local-name()=local-name(current())]">[<axsl:value-of select="$p_2"/>]</axsl:if></axsl:otherwise></axsl:choose></axsl:template><axsl:template match="@*" mode="schematron-get-full-path"><axsl:text>/</axsl:text><axsl:choose><axsl:when test="namespace-uri()=''">@<axsl:value-of select="name()"/></axsl:when><axsl:otherwise><axsl:text>@*[local-name()='</axsl:text><axsl:value-of select="local-name()"/><axsl:text>' and namespace-uri()='</axsl:text><axsl:value-of select="namespace-uri()"/><axsl:text>']</axsl:text></axsl:otherwise></axsl:choose></axsl:template>

<!--MODE: SCHEMATRON-FULL-PATH-2-->
<!--This mode can be used to generate prefixed XPath for humans-->
<axsl:template match="node() | @*" mode="schematron-get-full-path-2"><axsl:for-each select="ancestor-or-self::*"><axsl:text>/</axsl:text><axsl:value-of select="name(.)"/><axsl:if test="preceding-sibling::*[name(.)=name(current())]"><axsl:text>[</axsl:text><axsl:value-of select="count(preceding-sibling::*[name(.)=name(current())])+1"/><axsl:text>]</axsl:text></axsl:if></axsl:for-each><axsl:if test="not(self::*)"><axsl:text/>/@<axsl:value-of select="name(.)"/></axsl:if></axsl:template>

<!--MODE: GENERATE-ID-FROM-PATH -->
<axsl:template match="/" mode="generate-id-from-path"/><axsl:template match="text()" mode="generate-id-from-path"><axsl:apply-templates select="parent::*" mode="generate-id-from-path"/><axsl:value-of select="concat('.text-', 1+count(preceding-sibling::text()), '-')"/></axsl:template><axsl:template match="comment()" mode="generate-id-from-path"><axsl:apply-templates select="parent::*" mode="generate-id-from-path"/><axsl:value-of select="concat('.comment-', 1+count(preceding-sibling::comment()), '-')"/></axsl:template><axsl:template match="processing-instruction()" mode="generate-id-from-path"><axsl:apply-templates select="parent::*" mode="generate-id-from-path"/><axsl:value-of select="concat('.processing-instruction-', 1+count(preceding-sibling::processing-instruction()), '-')"/></axsl:template><axsl:template match="@*" mode="generate-id-from-path"><axsl:apply-templates select="parent::*" mode="generate-id-from-path"/><axsl:value-of select="concat('.@', name())"/></axsl:template><axsl:template match="*" mode="generate-id-from-path" priority="-0.5"><axsl:apply-templates select="parent::*" mode="generate-id-from-path"/><axsl:text>.</axsl:text><axsl:value-of select="concat('.',name(),'-',1+count(preceding-sibling::*[name()=name(current())]),'-')"/></axsl:template><!--MODE: SCHEMATRON-FULL-PATH-3-->
<!--This mode can be used to generate prefixed XPath for humans 
	(Top-level element has index)-->
<axsl:template match="node() | @*" mode="schematron-get-full-path-3"><axsl:for-each select="ancestor-or-self::*"><axsl:text>/</axsl:text><axsl:value-of select="name(.)"/><axsl:if test="parent::*"><axsl:text>[</axsl:text><axsl:value-of select="count(preceding-sibling::*[name(.)=name(current())])+1"/><axsl:text>]</axsl:text></axsl:if></axsl:for-each><axsl:if test="not(self::*)"><axsl:text/>/@<axsl:value-of select="name(.)"/></axsl:if></axsl:template>

<!--MODE: GENERATE-ID-2 -->
<axsl:template match="/" mode="generate-id-2">U</axsl:template><axsl:template match="*" mode="generate-id-2" priority="2"><axsl:text>U</axsl:text><axsl:number level="multiple" count="*"/></axsl:template><axsl:template match="node()" mode="generate-id-2"><axsl:text>U.</axsl:text><axsl:number level="multiple" count="*"/><axsl:text>n</axsl:text><axsl:number count="node()"/></axsl:template><axsl:template match="@*" mode="generate-id-2"><axsl:text>U.</axsl:text><axsl:number level="multiple" count="*"/><axsl:text>_</axsl:text><axsl:value-of select="string-length(local-name(.))"/><axsl:text>_</axsl:text><axsl:value-of select="translate(name(),':','.')"/></axsl:template><!--Strip characters--><axsl:template match="text()" priority="-1"/>

<!--SCHEMA METADATA-->
<axsl:template match="/"><svrl:schematron-output xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" title="" schemaVersion=""><axsl:comment><axsl:value-of select="$archiveDirParameter"/>   
		 <axsl:value-of select="$archiveNameParameter"/>  
		 <axsl:value-of select="$fileNameParameter"/>  
		 <axsl:value-of select="$fileDirParameter"/></axsl:comment><svrl:text>
  *******************************************************************************
   THINGS TO BE SURE BEFORE EDITING THIS FILE!

   The spec used is ISO-Schematron. 
   
   Some useful info:
     - The query language used is the extended version of XPath specified in XSLT.
     - The rule context is interpreted according to the Production 1 of XSLT. 
       The rule context may be the root node, elements, attributes, comments and 
       processing instructions. 
     - The assertion test is interpreted according to Production 14 of XPath, as 
       returning a Boolean value.

   For more info, refer to the official ISO/IEC 19757-3:2006(E) standard.
  
   The implementation of the schematron patterns comes with the idea of SPS as a
   set of constraints on top of JATS' Publishing Tag Set v1.0 (JPTS)[1]. To keep
   consistency, please make sure:
  
     - DTD/XSD constraints are not duplicated here
     - There is an issue at http://git.io/5EcR4Q with status `Aprovada`
     - PMC-Style compatibility is desired[2]
  
   Always double-check the JPTS and PMC-Style before editing.
   [1] http://jats.nlm.nih.gov/publishing/tag-library/1.0/
   [2] https://www.ncbi.nlm.nih.gov/pmc/pmcdoc/tagging-guidelines/article/tags.html
  *******************************************************************************
  </svrl:text><svrl:ns-prefix-in-attribute-values uri="http://www.w3.org/1999/xlink" prefix="xlink"/><svrl:ns-prefix-in-attribute-values uri="http://exslt.org/regular-expressions" prefix="regexp"/><svrl:active-pattern><axsl:attribute name="id">journal-id_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M54"/><svrl:active-pattern><axsl:attribute name="id">journal-id_has_publisher-id</axsl:attribute><axsl:attribute name="name">
      There exists one journal-id[@journal-id-type='publisher-id'].
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M55"/><svrl:active-pattern><axsl:attribute name="id">journal-id_values</axsl:attribute><axsl:attribute name="name">journal-id_values</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M56"/><svrl:active-pattern><axsl:attribute name="id">has_journal-title_and_abbrev-journal-title</axsl:attribute><axsl:attribute name="name">has_journal-title_and_abbrev-journal-title</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M57"/><svrl:active-pattern><axsl:attribute name="id">journal-title_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M58"/><svrl:active-pattern><axsl:attribute name="id">abbrev-journal-title_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M59"/><svrl:active-pattern><axsl:attribute name="id">publisher</axsl:attribute><axsl:attribute name="name">publisher</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M60"/><svrl:active-pattern><axsl:attribute name="id">publisher_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M61"/><svrl:active-pattern><axsl:attribute name="id">article_categories</axsl:attribute><axsl:attribute name="name">article_categories</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M62"/><svrl:active-pattern><axsl:attribute name="id">fpage_or_elocation-id</axsl:attribute><axsl:attribute name="name">fpage_or_elocation-id</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M63"/><svrl:active-pattern><axsl:attribute name="id">fpage_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M64"/><svrl:active-pattern><axsl:attribute name="id">elocation-id_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M65"/><svrl:active-pattern><axsl:attribute name="id">issn_pub_type_epub_or_ppub</axsl:attribute><axsl:attribute name="name">issn_pub_type_epub_or_ppub</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M66"/><svrl:active-pattern><axsl:attribute name="id">issn_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M67"/><svrl:active-pattern><axsl:attribute name="id">article-id_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M68"/><svrl:active-pattern><axsl:attribute name="id">article-id_attributes</axsl:attribute><axsl:attribute name="name">
      Mandatory attributes are present.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M69"/><svrl:active-pattern><axsl:attribute name="id">article-id_values</axsl:attribute><axsl:attribute name="name">
      Values are known.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M70"/><svrl:active-pattern><axsl:attribute name="id">subj_group</axsl:attribute><axsl:attribute name="name">
      Make sure only one heading is provided per language, in subj-group.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M71"/><svrl:active-pattern><axsl:attribute name="id">subj_group_subarticle_pt</axsl:attribute><axsl:attribute name="name">
      Make sure only one heading is provided per language, in subj-group.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M72"/><svrl:active-pattern><axsl:attribute name="id">subj_group_subarticle_es</axsl:attribute><axsl:attribute name="name">
      Make sure only one heading is provided per language, in subj-group.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M73"/><svrl:active-pattern><axsl:attribute name="id">subj_group_subarticle_en</axsl:attribute><axsl:attribute name="name">
      Make sure only one heading is provided per language, in subj-group.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M74"/><svrl:active-pattern><axsl:attribute name="id">subj_group_subarticle_fr</axsl:attribute><axsl:attribute name="name">
      Make sure only one heading is provided per language, in subj-group.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M75"/><svrl:active-pattern><axsl:attribute name="id">abstract</axsl:attribute><axsl:attribute name="name">abstract</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M76"/><svrl:active-pattern><axsl:attribute name="id">abstract_abstract-type_values</axsl:attribute><axsl:attribute name="name">abstract_abstract-type_values</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M77"/><svrl:active-pattern><axsl:attribute name="id">article-title</axsl:attribute><axsl:attribute name="name">article-title</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M78"/><svrl:active-pattern><axsl:attribute name="id">aff_contenttypes</axsl:attribute><axsl:attribute name="name">aff_contenttypes</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M79"/><svrl:active-pattern><axsl:attribute name="id">aff_contenttypes_contribgroup</axsl:attribute><axsl:attribute name="name">aff_contenttypes_contribgroup</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M80"/><svrl:active-pattern><axsl:attribute name="id">kwdgroup_lang</axsl:attribute><axsl:attribute name="name">
      Make sure all kwd-group elements have xml:lang attribute.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M81"/><svrl:active-pattern><axsl:attribute name="id">counts_tables</axsl:attribute><axsl:attribute name="name">
      Make sure the total number of tables are correct.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M82"/><svrl:active-pattern><axsl:attribute name="id">counts_refs</axsl:attribute><axsl:attribute name="name">
      Make sure the total number of refs are correct.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M83"/><svrl:active-pattern><axsl:attribute name="id">counts_figs</axsl:attribute><axsl:attribute name="name">
      Make sure the total number of figures are correct.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M84"/><svrl:active-pattern><axsl:attribute name="id">counts_equations</axsl:attribute><axsl:attribute name="name">
      Make sure the total number of equations are correct.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M85"/><svrl:active-pattern><axsl:attribute name="id">counts_pages</axsl:attribute><axsl:attribute name="name">
      Make sure the total number of pages are correct.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M86"/><svrl:active-pattern><axsl:attribute name="id">pub-date_date_type</axsl:attribute><axsl:attribute name="name">
      Restrict the valid values of @date-type.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M87"/><svrl:active-pattern><axsl:attribute name="id">pub-date_type_pub</axsl:attribute><axsl:attribute name="name">
      Make sure there exists at least one pub-date of type pub.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M88"/><svrl:active-pattern><axsl:attribute name="id">pub-date_publication_format</axsl:attribute><axsl:attribute name="name">
      Restrict the valid values of @publication-format.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M89"/><svrl:active-pattern><axsl:attribute name="id">pub-date_type_collection</axsl:attribute><axsl:attribute name="name">
      Assert rules on pub-date[@date-type="collection"] child elements.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M90"/><svrl:active-pattern><axsl:attribute name="id">pub-date_type_not_collection</axsl:attribute><axsl:attribute name="name">
      Assert rules on pub-date[@date-type != "pub"] child elements.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M91"/><svrl:active-pattern><axsl:attribute name="id">volume_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M92"/><svrl:active-pattern><axsl:attribute name="id">issue_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M93"/><svrl:active-pattern><axsl:attribute name="id">supplement</axsl:attribute><axsl:attribute name="name">
      Make sure the supplement is not present.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M94"/><svrl:active-pattern><axsl:attribute name="id">elocation-id</axsl:attribute><axsl:attribute name="name">
      Allow elocation-id to be present only when fpage is absent.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M95"/><svrl:active-pattern><axsl:attribute name="id">history</axsl:attribute><axsl:attribute name="name">
      Restrict the valid values of history/date/[@date-type].
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M96"/><svrl:active-pattern><axsl:attribute name="id">product</axsl:attribute><axsl:attribute name="name">
      Allow product to be present only when article-type is book-review or product-review.
      Also, make sure product[@product-type='book'].
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M97"/><svrl:active-pattern><axsl:attribute name="id">product_product-type_values</axsl:attribute><axsl:attribute name="name">
      Make sure the supplied values are valid.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M98"/><svrl:active-pattern><axsl:attribute name="id">sectitle</axsl:attribute><axsl:attribute name="name">
      Make sure all sections have a title element.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M99"/><svrl:active-pattern><axsl:attribute name="id">paragraph</axsl:attribute><axsl:attribute name="name">
      Make sure paragraphs have no id attr.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M100"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-aff</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M101"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-app</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M102"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-author-notes</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M103"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-bibr</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M104"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-contrib</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M105"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-corresp</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M106"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-disp-formula</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M107"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-fig</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M108"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-fn</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M109"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-sec</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M110"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-supplementary-material</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M111"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-table</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M112"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-integrity-table-fn</axsl:attribute><axsl:attribute name="name">
      Make sure all references to are reachable.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M113"/><svrl:active-pattern><axsl:attribute name="id">xref-reftype-values</axsl:attribute><axsl:attribute name="name">
      Validate the ref-type value against a list.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M114"/><svrl:active-pattern><axsl:attribute name="id">caption_title</axsl:attribute><axsl:attribute name="name">
      Make sure all captions have a title element.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M115"/><svrl:active-pattern><axsl:attribute name="id">license_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure all mandatory attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M116"/><svrl:active-pattern><axsl:attribute name="id">license</axsl:attribute><axsl:attribute name="name">
      Make sure the document has a permissions element, and a valid
      license (represented as a known href).

      Valid licenses are:
        - http://creativecommons.org/licenses/by-nc/4.0/
        - http://creativecommons.org/licenses/by-nc/3.0/
        - http://creativecommons.org/licenses/by/4.0/
        - http://creativecommons.org/licenses/by/3.0/
        - http://creativecommons.org/licenses/by-nc-nd/4.0/
        - http://creativecommons.org/licenses/by-nc-nd/3.0/
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M117"/><svrl:active-pattern><axsl:attribute name="id">ack</axsl:attribute><axsl:attribute name="name">
      Ack elements cannot be organized as sections (sec).
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M118"/><svrl:active-pattern><axsl:attribute name="id">element-citation</axsl:attribute><axsl:attribute name="name">
      - Make sure name, etal and collab are not child of element-citation.
      - element-citation can be only child of ref elements.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M119"/><svrl:active-pattern><axsl:attribute name="id">person-group</axsl:attribute><axsl:attribute name="name">
      Make sure person-group-type is present.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M120"/><svrl:active-pattern><axsl:attribute name="id">person-group-type_values</axsl:attribute><axsl:attribute name="name">
      person-group/@person-group-type value constraints.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M121"/><svrl:active-pattern><axsl:attribute name="id">fn_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M122"/><svrl:active-pattern><axsl:attribute name="id">fn</axsl:attribute><axsl:attribute name="name">
      Make sure fn-type is valid against a white list.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M123"/><svrl:active-pattern><axsl:attribute name="id">fn-group</axsl:attribute><axsl:attribute name="name">fn-group</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M124"/><svrl:active-pattern><axsl:attribute name="id">xhtml-table</axsl:attribute><axsl:attribute name="name">
      Tables should be fully tagged. tr elements are not supposed to be declared
      at toplevel.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M125"/><svrl:active-pattern><axsl:attribute name="id">supplementary-material_mimetype</axsl:attribute><axsl:attribute name="name">
      The attributes mimetype and mime-subtype are required.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M126"/><svrl:active-pattern><axsl:attribute name="id">article_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M127"/><svrl:active-pattern><axsl:attribute name="id">article_article-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for article/@article-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M128"/><svrl:active-pattern><axsl:attribute name="id">article_specific-use-values</axsl:attribute><axsl:attribute name="name">
      The SPS version must be declared in article/@specific-use 
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M129"/><svrl:active-pattern><axsl:attribute name="id">named-content_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M130"/><svrl:active-pattern><axsl:attribute name="id">named-content_content-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for named-content/@content-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M131"/><svrl:active-pattern><axsl:attribute name="id">month</axsl:attribute><axsl:attribute name="name">
      Only integers between 1 and 12.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M132"/><svrl:active-pattern><axsl:attribute name="id">size_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M133"/><svrl:active-pattern><axsl:attribute name="id">size_units-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for size/@units
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M134"/><svrl:active-pattern><axsl:attribute name="id">list_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M135"/><svrl:active-pattern><axsl:attribute name="id">list_list-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for list/@list-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M136"/><svrl:active-pattern><axsl:attribute name="id">media_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M137"/><svrl:active-pattern><axsl:attribute name="id">ext-link_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present. Also, the value of 
      ext-link-type is validated against a white-list.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M138"/><svrl:active-pattern><axsl:attribute name="id">ext-link_href_values</axsl:attribute><axsl:attribute name="name">
        All URIs are supported, except "file", because we dont want references 
        to local files.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M139"/><svrl:active-pattern><axsl:attribute name="id">element-citation_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present. 
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M140"/><svrl:active-pattern><axsl:attribute name="id">element-citation_publication-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for element-citation/@publication-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M141"/><svrl:active-pattern><axsl:attribute name="id">sub-article_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M142"/><svrl:active-pattern><axsl:attribute name="id">sub-article_article-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for //sub-article/@article-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M143"/><svrl:active-pattern><axsl:attribute name="id">response_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M144"/><svrl:active-pattern><axsl:attribute name="id">response_response-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for //response/@response-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M145"/><svrl:active-pattern><axsl:attribute name="id">response_related-article_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present

      @id is mandatory for all use cases, so it is not implemented in this 
      pattern.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M146"/><svrl:active-pattern><axsl:attribute name="id">related-article_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M147"/><svrl:active-pattern><axsl:attribute name="id">related-article_correction_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present for corrections.
      http://docs.scielo.org/projects/scielo-publishing-schema/pt_BR/1.5-branch/narr/errata.html#errata
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M148"/><svrl:active-pattern><axsl:attribute name="id">related-article-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for //related-article/@related-article-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M149"/><svrl:active-pattern><axsl:attribute name="id">correction_related-article</axsl:attribute><axsl:attribute name="name">
      Articles of type correction must specify the article it relates to.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M150"/><svrl:active-pattern><axsl:attribute name="id">correction_article-type</axsl:attribute><axsl:attribute name="name">
      Ensure related-article[@related-article-type='corrected-article' is 
      defined only for correction article types.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M151"/><svrl:active-pattern><axsl:attribute name="id">inbrief_related-article</axsl:attribute><axsl:attribute name="name">
      Articles of type in-brief must specify the article it relates to.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M152"/><svrl:active-pattern><axsl:attribute name="id">inbrief_article-type</axsl:attribute><axsl:attribute name="name">
      Ensure related-article[@related-article-type='article-reference' is 
      defined only for in-brief article types.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M153"/><svrl:active-pattern><axsl:attribute name="id">funding-group</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M154"/><svrl:active-pattern><axsl:attribute name="id">funding-group_elements</axsl:attribute><axsl:attribute name="name">
      Make sure mandatory child elements are present.  
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M155"/><svrl:active-pattern><axsl:attribute name="id">aff_country-attrs</axsl:attribute><axsl:attribute name="name">
      Ensure the attribute 'country' is present for all //aff/country elements.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M156"/><svrl:active-pattern><axsl:attribute name="id">aff_country</axsl:attribute><axsl:attribute name="name">
      //aff/country elements cannot be empty.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M157"/><svrl:active-pattern><axsl:attribute name="id">ref</axsl:attribute><axsl:attribute name="name">
      element-citation and mixed-citation are mandatory in ref.
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M158"/><svrl:active-pattern><axsl:attribute name="id">ref_notempty</axsl:attribute><axsl:attribute name="name">
    Check if the element's text is at least one character long.
  </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M159"/><svrl:active-pattern><axsl:attribute name="id">contrib-id_attributes</axsl:attribute><axsl:attribute name="name">
      Make sure some attributes are present
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M160"/><svrl:active-pattern><axsl:attribute name="id">contrib-id-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for //contrib/contrib-id/@contrib-id-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M161"/><svrl:active-pattern><axsl:attribute name="id">source_cardinality</axsl:attribute><axsl:attribute name="name">source_cardinality</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M162"/><svrl:active-pattern><axsl:attribute name="id">size_cardinality</axsl:attribute><axsl:attribute name="name">size_cardinality</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M163"/><svrl:active-pattern><axsl:attribute name="id">month_cardinality</axsl:attribute><axsl:attribute name="name">month_cardinality</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M164"/><svrl:active-pattern><axsl:attribute name="id">issue_cardinality</axsl:attribute><axsl:attribute name="name">issue_cardinality</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M165"/><svrl:active-pattern><axsl:attribute name="id">chapter-title_cardinality</axsl:attribute><axsl:attribute name="name">chapter-title_cardinality</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M166"/><svrl:active-pattern><axsl:attribute name="id">element-citation_cardinality</axsl:attribute><axsl:attribute name="name">element-citation_cardinality</axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M167"/><svrl:active-pattern><axsl:attribute name="id">related-article_ext-link-type-values</axsl:attribute><axsl:attribute name="name">
      Allowed values for //related-article/@ext-link-type
    </axsl:attribute><axsl:apply-templates/></svrl:active-pattern><axsl:apply-templates select="/" mode="M168"/></svrl:schematron-output></axsl:template>

<!--SCHEMATRON PATTERNS-->


<!--PATTERN journal-id_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/journal-meta/journal-id" priority="1000" mode="M54"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta/journal-id"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M54"/></axsl:template><axsl:template match="text()" priority="-1" mode="M54"/><axsl:template match="@*|node()" priority="-2" mode="M54"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M54"/></axsl:template>

<!--PATTERN journal-id_has_publisher-id
      There exists one journal-id[@journal-id-type='publisher-id'].
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      There exists one journal-id[@journal-id-type='publisher-id'].
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/journal-meta" priority="1000" mode="M55"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="journal-id[@journal-id-type='publisher-id']"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="journal-id[@journal-id-type='publisher-id']"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'journal-meta': Missing element journal-id with journal-id-type="publisher-id".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M55"/></axsl:template><axsl:template match="text()" priority="-1" mode="M55"/><axsl:template match="@*|node()" priority="-2" mode="M55"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M55"/></axsl:template>

<!--PATTERN journal-id_values-->


	<!--RULE -->
<axsl:template match="article/front/journal-meta/journal-id[@journal-id-type]" priority="1000" mode="M56"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta/journal-id[@journal-id-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@journal-id-type = 'nlm-ta' or                     @journal-id-type = 'publisher-id'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@journal-id-type = 'nlm-ta' or @journal-id-type = 'publisher-id'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'journal-id', attribute journal-id-type: Invalid value "<axsl:text/><axsl:value-of select="@journal-id-type"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M56"/></axsl:template><axsl:template match="text()" priority="-1" mode="M56"/><axsl:template match="@*|node()" priority="-2" mode="M56"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M56"/></axsl:template>

<!--PATTERN has_journal-title_and_abbrev-journal-title-->


	<!--RULE -->
<axsl:template match="article/front/journal-meta" priority="1001" mode="M57"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="journal-title-group"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="journal-title-group"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'journal-meta': Missing element journal-title-group.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M57"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/journal-meta/journal-title-group" priority="1000" mode="M57"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta/journal-title-group"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="journal-title"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="journal-title"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'journal-title-group': Missing element journal-title.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="abbrev-journal-title[@abbrev-type='publisher']"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="abbrev-journal-title[@abbrev-type='publisher']"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'journal-title-group': Missing element abbrev-journal-title with abbrev-type="publisher".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M57"/></axsl:template><axsl:template match="text()" priority="-1" mode="M57"/><axsl:template match="@*|node()" priority="-2" mode="M57"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M57"/></axsl:template>

<!--PATTERN journal-title_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/journal-meta/journal-title-group/journal-title" priority="1000" mode="M58"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta/journal-title-group/journal-title"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M58"/></axsl:template><axsl:template match="text()" priority="-1" mode="M58"/><axsl:template match="@*|node()" priority="-2" mode="M58"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M58"/></axsl:template>

<!--PATTERN abbrev-journal-title_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/journal-meta/journal-title-group/abbrev-journal-title" priority="1000" mode="M59"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta/journal-title-group/abbrev-journal-title"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M59"/></axsl:template><axsl:template match="text()" priority="-1" mode="M59"/><axsl:template match="@*|node()" priority="-2" mode="M59"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M59"/></axsl:template>

<!--PATTERN publisher-->


	<!--RULE -->
<axsl:template match="article/front/journal-meta" priority="1000" mode="M60"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="publisher"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="publisher"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'journal-meta': Missing element publisher.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M60"/></axsl:template><axsl:template match="text()" priority="-1" mode="M60"/><axsl:template match="@*|node()" priority="-2" mode="M60"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M60"/></axsl:template>

<!--PATTERN publisher_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/journal-meta/publisher/publisher-name" priority="1000" mode="M61"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta/publisher/publisher-name"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M61"/></axsl:template><axsl:template match="text()" priority="-1" mode="M61"/><axsl:template match="@*|node()" priority="-2" mode="M61"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M61"/></axsl:template>

<!--PATTERN article_categories-->


	<!--RULE -->
<axsl:template match="article/front/article-meta" priority="1000" mode="M62"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="article-categories"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="article-categories"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing element article-categories.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M62"/></axsl:template><axsl:template match="text()" priority="-1" mode="M62"/><axsl:template match="@*|node()" priority="-2" mode="M62"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M62"/></axsl:template>

<!--PATTERN fpage_or_elocation-id-->


	<!--RULE -->
<axsl:template match="article/front/article-meta" priority="1000" mode="M63"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="fpage or elocation-id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="fpage or elocation-id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing elements fpage or elocation-id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M63"/></axsl:template><axsl:template match="text()" priority="-1" mode="M63"/><axsl:template match="@*|node()" priority="-2" mode="M63"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M63"/></axsl:template>

<!--PATTERN fpage_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/fpage" priority="1000" mode="M64"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/fpage"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M64"/></axsl:template><axsl:template match="text()" priority="-1" mode="M64"/><axsl:template match="@*|node()" priority="-2" mode="M64"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M64"/></axsl:template>

<!--PATTERN elocation-id_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/elocation-id" priority="1000" mode="M65"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/elocation-id"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M65"/></axsl:template><axsl:template match="text()" priority="-1" mode="M65"/><axsl:template match="@*|node()" priority="-2" mode="M65"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M65"/></axsl:template>

<!--PATTERN issn_pub_type_epub_or_ppub-->


	<!--RULE -->
<axsl:template match="article/front/journal-meta" priority="1000" mode="M66"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="issn[@pub-type='epub'] or issn[@pub-type='ppub']"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="issn[@pub-type='epub'] or issn[@pub-type='ppub']"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'journal-meta': Missing element issn with pub-type=("epub" or "ppub").
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M66"/></axsl:template><axsl:template match="text()" priority="-1" mode="M66"/><axsl:template match="@*|node()" priority="-2" mode="M66"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M66"/></axsl:template>

<!--PATTERN issn_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/journal-meta/issn" priority="1000" mode="M67"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/journal-meta/issn"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M67"/></axsl:template><axsl:template match="text()" priority="-1" mode="M67"/><axsl:template match="@*|node()" priority="-2" mode="M67"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M67"/></axsl:template>

<!--PATTERN article-id_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/article-id" priority="1000" mode="M68"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/article-id"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M68"/></axsl:template><axsl:template match="text()" priority="-1" mode="M68"/><axsl:template match="@*|node()" priority="-2" mode="M68"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M68"/></axsl:template>

<!--PATTERN article-id_attributes
      Mandatory attributes are present.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Mandatory attributes are present.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta" priority="1000" mode="M69"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="article-id[@pub-id-type]"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="article-id[@pub-id-type]"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing element article-id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M69"/></axsl:template><axsl:template match="text()" priority="-1" mode="M69"/><axsl:template match="@*|node()" priority="-2" mode="M69"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M69"/></axsl:template>

<!--PATTERN article-id_values
      Values are known.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Values are known.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/article-id[@pub-id-type] | article/sub-article[@article-type='translation']/front-stub/article-id[@pub-id-type]" priority="1000" mode="M70"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/article-id[@pub-id-type] | article/sub-article[@article-type='translation']/front-stub/article-id[@pub-id-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@pub-id-type = 'doi' or                      @pub-id-type = 'other' or                      @pub-id-type = 'publisher-id'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@pub-id-type = 'doi' or @pub-id-type = 'other' or @pub-id-type = 'publisher-id'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-id', attribute pub-id-type: Invalid value "<axsl:text/><axsl:value-of select="@pub-id-type"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M70"/></axsl:template><axsl:template match="text()" priority="-1" mode="M70"/><axsl:template match="@*|node()" priority="-2" mode="M70"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M70"/></axsl:template>

<!--PATTERN subj_group
      Make sure only one heading is provided per language, in subj-group.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure only one heading is provided per language, in subj-group.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/article-categories" priority="1000" mode="M71"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/article-categories"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be only one element subj-group with subj-group-type="heading".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M71"/></axsl:template><axsl:template match="text()" priority="-1" mode="M71"/><axsl:template match="@*|node()" priority="-2" mode="M71"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M71"/></axsl:template>

<!--PATTERN subj_group_subarticle_pt
      Make sure only one heading is provided per language, in subj-group.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure only one heading is provided per language, in subj-group.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/sub-article[@xml:lang='pt']/front-stub/article-categories" priority="1000" mode="M72"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/sub-article[@xml:lang='pt']/front-stub/article-categories"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be only one element subj-group with subj-group-type="heading".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M72"/></axsl:template><axsl:template match="text()" priority="-1" mode="M72"/><axsl:template match="@*|node()" priority="-2" mode="M72"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M72"/></axsl:template>

<!--PATTERN subj_group_subarticle_es
      Make sure only one heading is provided per language, in subj-group.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure only one heading is provided per language, in subj-group.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/sub-article[@xml:lang='es']/front-stub/article-categories" priority="1000" mode="M73"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/sub-article[@xml:lang='es']/front-stub/article-categories"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be only one element subj-group with subj-group-type="heading".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M73"/></axsl:template><axsl:template match="text()" priority="-1" mode="M73"/><axsl:template match="@*|node()" priority="-2" mode="M73"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M73"/></axsl:template>

<!--PATTERN subj_group_subarticle_en
      Make sure only one heading is provided per language, in subj-group.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure only one heading is provided per language, in subj-group.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/sub-article[@xml:lang='en']/front-stub/article-categories" priority="1000" mode="M74"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/sub-article[@xml:lang='en']/front-stub/article-categories"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be only one element subj-group with subj-group-type="heading".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M74"/></axsl:template><axsl:template match="text()" priority="-1" mode="M74"/><axsl:template match="@*|node()" priority="-2" mode="M74"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M74"/></axsl:template>

<!--PATTERN subj_group_subarticle_fr
      Make sure only one heading is provided per language, in subj-group.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure only one heading is provided per language, in subj-group.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/sub-article[@xml:lang='fr']/front-stub/article-categories" priority="1000" mode="M75"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/sub-article[@xml:lang='fr']/front-stub/article-categories"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(subj-group[@subj-group-type='heading'] | subj-group//subj-group[@subj-group-type='heading']) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be only one element subj-group with subj-group-type="heading".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M75"/></axsl:template><axsl:template match="text()" priority="-1" mode="M75"/><axsl:template match="@*|node()" priority="-2" mode="M75"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M75"/></axsl:template>

<!--PATTERN abstract-->


	<!--RULE -->
<axsl:template match="article[@article-type='research-article'] | article[@article-type='review-article']" priority="1001" mode="M76"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article[@article-type='research-article'] | article[@article-type='review-article']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(front/article-meta/abstract | front/article-meta/trans-abstract) &gt; 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(front/article-meta/abstract | front/article-meta/trans-abstract) &gt; 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing element abstract.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M76"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/abstract" priority="1000" mode="M76"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/abstract"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(@xml:lang)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(@xml:lang)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'abstract': Unexpected attribute xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M76"/></axsl:template><axsl:template match="text()" priority="-1" mode="M76"/><axsl:template match="@*|node()" priority="-2" mode="M76"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M76"/></axsl:template>

<!--PATTERN abstract_abstract-type_values-->


	<!--RULE -->
<axsl:template match="//trans-abstract[@abstract-type]" priority="1000" mode="M77"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//trans-abstract[@abstract-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@abstract-type = 'graphical'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@abstract-type = 'graphical'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'trans-abstract', attribute abstract-type: Invalid value "<axsl:text/><axsl:value-of select="@abstract-type"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M77"/></axsl:template><axsl:template match="text()" priority="-1" mode="M77"/><axsl:template match="@*|node()" priority="-2" mode="M77"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M77"/></axsl:template>

<!--PATTERN article-title-->


	<!--RULE -->
<axsl:template match="article/front/article-meta/title-group/article-title |                     article/back/ref-list/ref/element-citation/article-title" priority="1000" mode="M78"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/title-group/article-title |                     article/back/ref-list/ref/element-citation/article-title"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(@xml:lang)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(@xml:lang)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-title': Unexpected attribute xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M78"/></axsl:template><axsl:template match="text()" priority="-1" mode="M78"/><axsl:template match="@*|node()" priority="-2" mode="M78"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M78"/></axsl:template>

<!--PATTERN aff_contenttypes-->


	<!--RULE -->
<axsl:template match="article/front/article-meta/aff/institution" priority="1001" mode="M79"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/aff/institution"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@content-type='original' or                      @content-type='orgname' or                     @content-type='orgdiv1' or                     @content-type='orgdiv2' or                     @content-type='normalized'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@content-type='original' or @content-type='orgname' or @content-type='orgdiv1' or @content-type='orgdiv2' or @content-type='normalized'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute content-type: Invalid value "<axsl:text/><axsl:value-of select="@content-type"/><axsl:text/>". 
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M79"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/aff" priority="1000" mode="M79"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/aff"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(institution[@content-type='original']) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(institution[@content-type='original']) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': Must have exactly one element institution with content-type="original".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M79"/></axsl:template><axsl:template match="text()" priority="-1" mode="M79"/><axsl:template match="@*|node()" priority="-2" mode="M79"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M79"/></axsl:template>

<!--PATTERN aff_contenttypes_contribgroup-->


	<!--RULE -->
<axsl:template match="article/front/article-meta/contrib-group/aff/institution" priority="1001" mode="M80"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/contrib-group/aff/institution"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@content-type='original' or                      @content-type='orgname' or                     @content-type='orgdiv1' or                     @content-type='orgdiv2' or                     @content-type='normalized'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@content-type='original' or @content-type='orgname' or @content-type='orgdiv1' or @content-type='orgdiv2' or @content-type='normalized'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute content-type: Invalid value "<axsl:text/><axsl:value-of select="@content-type"/><axsl:text/>". 
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M80"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/contrib-group/aff" priority="1000" mode="M80"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/contrib-group/aff"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(institution[@content-type='original']) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(institution[@content-type='original']) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': Must have exactly one element institution with content-type="original".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M80"/></axsl:template><axsl:template match="text()" priority="-1" mode="M80"/><axsl:template match="@*|node()" priority="-2" mode="M80"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M80"/></axsl:template>

<!--PATTERN kwdgroup_lang
      Make sure all kwd-group elements have xml:lang attribute.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all kwd-group elements have xml:lang attribute.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/kwd-group" priority="1000" mode="M81"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/kwd-group"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xml:lang"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xml:lang"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'kwd-group': Missing attribute xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M81"/></axsl:template><axsl:template match="text()" priority="-1" mode="M81"/><axsl:template match="@*|node()" priority="-2" mode="M81"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M81"/></axsl:template>

<!--PATTERN counts_tables
      Make sure the total number of tables are correct.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the total number of tables are correct.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/counts/table-count" priority="1000" mode="M82"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/counts/table-count"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@count = count(//table-wrap)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@count = count(//table-wrap)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'table-count': Wrong value in table-count.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M82"/></axsl:template><axsl:template match="text()" priority="-1" mode="M82"/><axsl:template match="@*|node()" priority="-2" mode="M82"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M82"/></axsl:template>

<!--PATTERN counts_refs
      Make sure the total number of refs are correct.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the total number of refs are correct.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/counts/ref-count" priority="1000" mode="M83"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/counts/ref-count"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@count = count(//ref)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@count = count(//ref)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ref-count': Wrong value in ref-count.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M83"/></axsl:template><axsl:template match="text()" priority="-1" mode="M83"/><axsl:template match="@*|node()" priority="-2" mode="M83"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M83"/></axsl:template>

<!--PATTERN counts_figs
      Make sure the total number of figures are correct.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the total number of figures are correct.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/counts/fig-count" priority="1000" mode="M84"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/counts/fig-count"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@count = count(//fig)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@count = count(//fig)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'fig-count': Wrong value in fig-count.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M84"/></axsl:template><axsl:template match="text()" priority="-1" mode="M84"/><axsl:template match="@*|node()" priority="-2" mode="M84"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M84"/></axsl:template>

<!--PATTERN counts_equations
      Make sure the total number of equations are correct.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the total number of equations are correct.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/counts/equation-count" priority="1000" mode="M85"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/counts/equation-count"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@count = count(//disp-formula)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@count = count(//disp-formula)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'equation-count': Wrong value in equation-count.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M85"/></axsl:template><axsl:template match="text()" priority="-1" mode="M85"/><axsl:template match="@*|node()" priority="-2" mode="M85"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M85"/></axsl:template>

<!--PATTERN counts_pages
      Make sure the total number of pages are correct.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the total number of pages are correct.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/counts/page-count" priority="1000" mode="M86"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/counts/page-count"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="(/article/front/article-meta/lpage = 0 and                      /article/front/article-meta/fpage = 0 and                      @count = 0) or                       (regexp:test(/article/front/article-meta/fpage, '\D', 'i') or                       regexp:test(/article/front/article-meta/lpage, '\D', 'i')) or                      string-length(/article/front/article-meta/elocation-id) &gt; 0 or                      (@count = ((/article/front/article-meta/lpage - /article/front/article-meta/fpage) + 1))"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="(/article/front/article-meta/lpage = 0 and /article/front/article-meta/fpage = 0 and @count = 0) or (regexp:test(/article/front/article-meta/fpage, '\D', 'i') or regexp:test(/article/front/article-meta/lpage, '\D', 'i')) or string-length(/article/front/article-meta/elocation-id) &gt; 0 or (@count = ((/article/front/article-meta/lpage - /article/front/article-meta/fpage) + 1))"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'page-count': Wrong value in page-count.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M86"/></axsl:template><axsl:template match="text()" priority="-1" mode="M86"/><axsl:template match="@*|node()" priority="-2" mode="M86"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M86"/></axsl:template>

<!--PATTERN pub-date_date_type
      Restrict the valid values of @date-type.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Restrict the valid values of @date-type.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/pub-date[@date-type]" priority="1001" mode="M87"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/pub-date[@date-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@date-type = 'pub' or                     @date-type = 'collection'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@date-type = 'pub' or @date-type = 'collection'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date', attribute date-type: Invalid value "<axsl:text/><axsl:value-of select="@date-type"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M87"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/pub-date" priority="1000" mode="M87"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/pub-date"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@date-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@date-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Missing attribute date-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M87"/></axsl:template><axsl:template match="text()" priority="-1" mode="M87"/><axsl:template match="@*|node()" priority="-2" mode="M87"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M87"/></axsl:template>

<!--PATTERN pub-date_type_pub
      Make sure there exists at least one pub-date of type pub.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure there exists at least one pub-date of type pub.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta" priority="1000" mode="M88"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(pub-date[@date-type = 'pub']) &gt; 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(pub-date[@date-type = 'pub']) &gt; 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing element pub-date with date-type="pub".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M88"/></axsl:template><axsl:template match="text()" priority="-1" mode="M88"/><axsl:template match="@*|node()" priority="-2" mode="M88"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M88"/></axsl:template>

<!--PATTERN pub-date_publication_format
      Restrict the valid values of @publication-format.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Restrict the valid values of @publication-format.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/pub-date[@publication-format]" priority="1001" mode="M89"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/pub-date[@publication-format]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@publication-format = 'electronic'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@publication-format = 'electronic'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date', attribute publication-format: Invalid value "<axsl:text/><axsl:value-of select="@publication-format"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M89"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/pub-date" priority="1000" mode="M89"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/pub-date"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@publication-format"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@publication-format"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Missing attribute publication-format.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M89"/></axsl:template><axsl:template match="text()" priority="-1" mode="M89"/><axsl:template match="@*|node()" priority="-2" mode="M89"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M89"/></axsl:template>

<!--PATTERN pub-date_type_collection
      Assert rules on pub-date[@date-type="collection"] child elements.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Assert rules on pub-date[@date-type="collection"] child elements.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/pub-date[@date-type = 'collection']" priority="1000" mode="M90"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/pub-date[@date-type = 'collection']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(day)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(day)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Unexpected element day.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="year"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="year"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Missing element year.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M90"/></axsl:template><axsl:template match="text()" priority="-1" mode="M90"/><axsl:template match="@*|node()" priority="-2" mode="M90"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M90"/></axsl:template>

<!--PATTERN pub-date_type_not_collection
      Assert rules on pub-date[@date-type != "pub"] child elements.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Assert rules on pub-date[@date-type != "pub"] child elements.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/pub-date[@date-type != 'collection']" priority="1000" mode="M91"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/pub-date[@date-type != 'collection']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="day"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="day"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Missing element day.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="month"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="month"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Missing element month.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="year"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="year"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Missing element year.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(season)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(season)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'pub-date': Unexpected element season.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M91"/></axsl:template><axsl:template match="text()" priority="-1" mode="M91"/><axsl:template match="@*|node()" priority="-2" mode="M91"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M91"/></axsl:template>

<!--PATTERN volume_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/volume" priority="1000" mode="M92"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/volume"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M92"/></axsl:template><axsl:template match="text()" priority="-1" mode="M92"/><axsl:template match="@*|node()" priority="-2" mode="M92"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M92"/></axsl:template>

<!--PATTERN issue_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/issue" priority="1000" mode="M93"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/issue"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M93"/></axsl:template><axsl:template match="text()" priority="-1" mode="M93"/><axsl:template match="@*|node()" priority="-2" mode="M93"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M93"/></axsl:template>

<!--PATTERN supplement
      Make sure the supplement is not present.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the supplement is not present.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta" priority="1000" mode="M94"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(supplement)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(supplement)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Unexpected element supplement.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M94"/></axsl:template><axsl:template match="text()" priority="-1" mode="M94"/><axsl:template match="@*|node()" priority="-2" mode="M94"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M94"/></axsl:template>

<!--PATTERN elocation-id
      Allow elocation-id to be present only when fpage is absent.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allow elocation-id to be present only when fpage is absent.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/elocation-id | article/back/ref-list/ref/element-citation/elocation-id" priority="1000" mode="M95"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/elocation-id | article/back/ref-list/ref/element-citation/elocation-id"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(following-sibling::fpage)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(following-sibling::fpage)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Unexpected element elocation-id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M95"/></axsl:template><axsl:template match="text()" priority="-1" mode="M95"/><axsl:template match="@*|node()" priority="-2" mode="M95"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M95"/></axsl:template>

<!--PATTERN history
      Restrict the valid values of history/date/[@date-type].
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Restrict the valid values of history/date/[@date-type].
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/history/date" priority="1000" mode="M96"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/history/date"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@date-type = 'received' or                      @date-type = 'accepted' or                     @date-type = 'corrected' or                     @date-type = 'pub' or                     @date-type = 'preprint' or                     @date-type = 'retracted' or                     @date-type = 'rev-request' or                     @date-type = 'rev-recd'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@date-type = 'received' or @date-type = 'accepted' or @date-type = 'corrected' or @date-type = 'pub' or @date-type = 'preprint' or @date-type = 'retracted' or @date-type = 'rev-request' or @date-type = 'rev-recd'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'date', attribute date-type: Invalid value "<axsl:text/><axsl:value-of select="@date-type"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M96"/></axsl:template><axsl:template match="text()" priority="-1" mode="M96"/><axsl:template match="@*|node()" priority="-2" mode="M96"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M96"/></axsl:template>

<!--PATTERN product
      Allow product to be present only when article-type is book-review or product-review.
      Also, make sure product[@product-type='book'].
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allow product to be present only when article-type is book-review or product-review.
      Also, make sure product[@product-type='book'].
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/product" priority="1000" mode="M97"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/product"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="/article[@article-type='book-review']"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="/article[@article-type='book-review']"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Unexpected element product.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@product-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@product-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'product': Missing attribute product-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M97"/></axsl:template><axsl:template match="text()" priority="-1" mode="M97"/><axsl:template match="@*|node()" priority="-2" mode="M97"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M97"/></axsl:template>

<!--PATTERN product_product-type_values
      Make sure the supplied values are valid.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the supplied values are valid.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/product[@product-type]" priority="1000" mode="M98"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/product[@product-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@product-type = 'book' or                     @product-type = 'article' or                     @product-type = 'issue' or                     @product-type = 'website' or                     @product-type = 'film' or                     @product-type = 'software' or                     @product-type = 'hardware' or                     @product-type = 'other'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@product-type = 'book' or @product-type = 'article' or @product-type = 'issue' or @product-type = 'website' or @product-type = 'film' or @product-type = 'software' or @product-type = 'hardware' or @product-type = 'other'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'product', attribute product-type: Invalid value "<axsl:text/><axsl:value-of select="@product-type"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M98"/></axsl:template><axsl:template match="text()" priority="-1" mode="M98"/><axsl:template match="@*|node()" priority="-2" mode="M98"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M98"/></axsl:template>

<!--PATTERN sectitle
      Make sure all sections have a title element.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all sections have a title element.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/body/sec" priority="1000" mode="M99"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/body/sec"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(title) &gt; 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(title) &gt; 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'sec': Missing element title.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M99"/></axsl:template><axsl:template match="text()" priority="-1" mode="M99"/><axsl:template match="@*|node()" priority="-2" mode="M99"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M99"/></axsl:template>

<!--PATTERN paragraph
      Make sure paragraphs have no id attr.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure paragraphs have no id attr.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//p" priority="1000" mode="M100"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//p"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(@id)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(@id)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'p': Unexpected attribute id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M100"/></axsl:template><axsl:template match="text()" priority="-1" mode="M100"/><axsl:template match="@*|node()" priority="-2" mode="M100"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M100"/></axsl:template>

<!--PATTERN xref-reftype-integrity-aff
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='aff']" priority="1000" mode="M101"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='aff']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //aff/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //aff/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M101"/></axsl:template><axsl:template match="text()" priority="-1" mode="M101"/><axsl:template match="@*|node()" priority="-2" mode="M101"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M101"/></axsl:template>

<!--PATTERN xref-reftype-integrity-app
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='app']" priority="1000" mode="M102"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='app']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //app/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //app/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M102"/></axsl:template><axsl:template match="text()" priority="-1" mode="M102"/><axsl:template match="@*|node()" priority="-2" mode="M102"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M102"/></axsl:template>

<!--PATTERN xref-reftype-integrity-author-notes
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='author-notes']" priority="1000" mode="M103"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='author-notes']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //author-notes/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //author-notes/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M103"/></axsl:template><axsl:template match="text()" priority="-1" mode="M103"/><axsl:template match="@*|node()" priority="-2" mode="M103"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M103"/></axsl:template>

<!--PATTERN xref-reftype-integrity-bibr
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='bibr']" priority="1000" mode="M104"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='bibr']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //ref/@id | //element-citation/@id | //mixed-citation/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //ref/@id | //element-citation/@id | //mixed-citation/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M104"/></axsl:template><axsl:template match="text()" priority="-1" mode="M104"/><axsl:template match="@*|node()" priority="-2" mode="M104"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M104"/></axsl:template>

<!--PATTERN xref-reftype-integrity-contrib
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='contrib']" priority="1000" mode="M105"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='contrib']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //contrib/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //contrib/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M105"/></axsl:template><axsl:template match="text()" priority="-1" mode="M105"/><axsl:template match="@*|node()" priority="-2" mode="M105"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M105"/></axsl:template>

<!--PATTERN xref-reftype-integrity-corresp
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='corresp']" priority="1000" mode="M106"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='corresp']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //corresp/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //corresp/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M106"/></axsl:template><axsl:template match="text()" priority="-1" mode="M106"/><axsl:template match="@*|node()" priority="-2" mode="M106"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M106"/></axsl:template>

<!--PATTERN xref-reftype-integrity-disp-formula
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='disp-formula']" priority="1000" mode="M107"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='disp-formula']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //disp-formula/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //disp-formula/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M107"/></axsl:template><axsl:template match="text()" priority="-1" mode="M107"/><axsl:template match="@*|node()" priority="-2" mode="M107"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M107"/></axsl:template>

<!--PATTERN xref-reftype-integrity-fig
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='fig']" priority="1000" mode="M108"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='fig']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //fig/@id | //fig-group/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //fig/@id | //fig-group/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M108"/></axsl:template><axsl:template match="text()" priority="-1" mode="M108"/><axsl:template match="@*|node()" priority="-2" mode="M108"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M108"/></axsl:template>

<!--PATTERN xref-reftype-integrity-fn
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='fn']" priority="1000" mode="M109"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='fn']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //fn/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //fn/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M109"/></axsl:template><axsl:template match="text()" priority="-1" mode="M109"/><axsl:template match="@*|node()" priority="-2" mode="M109"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M109"/></axsl:template>

<!--PATTERN xref-reftype-integrity-sec
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='sec']" priority="1000" mode="M110"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='sec']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //sec/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //sec/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M110"/></axsl:template><axsl:template match="text()" priority="-1" mode="M110"/><axsl:template match="@*|node()" priority="-2" mode="M110"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M110"/></axsl:template>

<!--PATTERN xref-reftype-integrity-supplementary-material
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='supplementary-material']" priority="1000" mode="M111"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='supplementary-material']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //supplementary-material/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //supplementary-material/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M111"/></axsl:template><axsl:template match="text()" priority="-1" mode="M111"/><axsl:template match="@*|node()" priority="-2" mode="M111"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M111"/></axsl:template>

<!--PATTERN xref-reftype-integrity-table
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='table']" priority="1000" mode="M112"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='table']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //table-wrap/@id | //table-wrap-group/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //table-wrap/@id | //table-wrap-group/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M112"/></axsl:template><axsl:template match="text()" priority="-1" mode="M112"/><axsl:template match="@*|node()" priority="-2" mode="M112"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M112"/></axsl:template>

<!--PATTERN xref-reftype-integrity-table-fn
      Make sure all references to are reachable.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all references to are reachable.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type='table-fn']" priority="1000" mode="M113"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type='table-fn']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@rid = //table-wrap-foot/fn/@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@rid = //table-wrap-foot/fn/@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>', attribute rid: Mismatching id value '<axsl:text/><axsl:value-of select="@rid"/><axsl:text/>' of type '<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M113"/></axsl:template><axsl:template match="text()" priority="-1" mode="M113"/><axsl:template match="@*|node()" priority="-2" mode="M113"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M113"/></axsl:template>

<!--PATTERN xref-reftype-values
      Validate the ref-type value against a list.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Validate the ref-type value against a list.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//xref[@ref-type]" priority="1000" mode="M114"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//xref[@ref-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@ref-type = 'aff' or                     @ref-type = 'app' or                     @ref-type = 'author-notes' or                     @ref-type = 'bibr' or                      @ref-type = 'contrib' or                     @ref-type = 'corresp' or                     @ref-type = 'disp-formula' or                     @ref-type = 'fig' or                      @ref-type = 'fn' or                     @ref-type = 'sec' or                     @ref-type = 'supplementary-material' or                     @ref-type = 'table' or                     @ref-type = 'table-fn' or                     @ref-type = 'boxed-text'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@ref-type = 'aff' or @ref-type = 'app' or @ref-type = 'author-notes' or @ref-type = 'bibr' or @ref-type = 'contrib' or @ref-type = 'corresp' or @ref-type = 'disp-formula' or @ref-type = 'fig' or @ref-type = 'fn' or @ref-type = 'sec' or @ref-type = 'supplementary-material' or @ref-type = 'table' or @ref-type = 'table-fn' or @ref-type = 'boxed-text'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'xref', attribute ref-type: Invalid value "<axsl:text/><axsl:value-of select="@ref-type"/><axsl:text/>".
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M114"/></axsl:template><axsl:template match="text()" priority="-1" mode="M114"/><axsl:template match="@*|node()" priority="-2" mode="M114"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M114"/></axsl:template>

<!--PATTERN caption_title
      Make sure all captions have a title element.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all captions have a title element.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//caption" priority="1000" mode="M115"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//caption"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="title and string-length(title) &gt; 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="title and string-length(title) &gt; 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'caption': Missing element title.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M115"/></axsl:template><axsl:template match="text()" priority="-1" mode="M115"/><axsl:template match="@*|node()" priority="-2" mode="M115"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M115"/></axsl:template>

<!--PATTERN license_attributes
      Make sure all mandatory attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure all mandatory attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/permissions/license" priority="1000" mode="M116"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/permissions/license"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@license-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@license-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'license': Missing attribute license-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xlink:href"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xlink:href"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'license': Missing attribute xlink:href.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xml:lang"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xml:lang"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'license': Missing attribute xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M116"/></axsl:template><axsl:template match="text()" priority="-1" mode="M116"/><axsl:template match="@*|node()" priority="-2" mode="M116"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M116"/></axsl:template>

<!--PATTERN license
      Make sure the document has a permissions element, and a valid
      license (represented as a known href).

      Valid licenses are:
        - http://creativecommons.org/licenses/by-nc/4.0/
        - http://creativecommons.org/licenses/by-nc/3.0/
        - http://creativecommons.org/licenses/by/4.0/
        - http://creativecommons.org/licenses/by/3.0/
        - http://creativecommons.org/licenses/by-nc-nd/4.0/
        - http://creativecommons.org/licenses/by-nc-nd/3.0/
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure the document has a permissions element, and a valid
      license (represented as a known href).

      Valid licenses are:
        - http://creativecommons.org/licenses/by-nc/4.0/
        - http://creativecommons.org/licenses/by-nc/3.0/
        - http://creativecommons.org/licenses/by/4.0/
        - http://creativecommons.org/licenses/by/3.0/
        - http://creativecommons.org/licenses/by-nc-nd/4.0/
        - http://creativecommons.org/licenses/by-nc-nd/3.0/
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta" priority="1002" mode="M117"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="permissions"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="permissions"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing element permissions.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M117"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/permissions" priority="1001" mode="M117"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/permissions"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="license"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="license"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'permissions': Missing element license.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M117"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/permissions/license[@license-type and @xlink:href]" priority="1000" mode="M117"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/permissions/license[@license-type and @xlink:href]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@license-type = 'open-access'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@license-type = 'open-access'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'license', attribute license-type: Invalid value '<axsl:text/><axsl:value-of select="@license-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="regexp:test(@xlink:href, '^https?://creativecommons\.org/licenses/')"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="regexp:test(@xlink:href, '^https?://creativecommons\.org/licenses/')"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'license', attribute xlink:href: Invalid value '<axsl:text/><axsl:value-of select="@xlink:href"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xml:lang = 'en' or @xml:lang = /article/@xml:lang"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xml:lang = 'en' or @xml:lang = /article/@xml:lang"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'license', attribute xml:lang: Must be 'en' or match with article/@xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M117"/></axsl:template><axsl:template match="text()" priority="-1" mode="M117"/><axsl:template match="@*|node()" priority="-2" mode="M117"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M117"/></axsl:template>

<!--PATTERN ack
      Ack elements cannot be organized as sections (sec).
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Ack elements cannot be organized as sections (sec).
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ack" priority="1000" mode="M118"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ack"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(sec)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(sec)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
          Element 'ack': Unexpected element sec.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M118"/></axsl:template><axsl:template match="text()" priority="-1" mode="M118"/><axsl:template match="@*|node()" priority="-2" mode="M118"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M118"/></axsl:template>

<!--PATTERN element-citation
      - Make sure name, etal and collab are not child of element-citation.
      - element-citation can be only child of ref elements.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      - Make sure name, etal and collab are not child of element-citation.
      - element-citation can be only child of ref elements.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation" priority="1001" mode="M119"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(name)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(name)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'element-citation': Unexpected element name.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(etal)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(etal)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'element-citation': Unexpected element etal.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(collab)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(collab)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'element-citation': Unexpected element collab.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M119"/></axsl:template>

	<!--RULE -->
<axsl:template match="//element-citation" priority="1000" mode="M119"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//element-citation"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="parent::ref"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="parent::ref"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Unexpected element 'element-citation': Allowed only as child of ref elements.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M119"/></axsl:template><axsl:template match="text()" priority="-1" mode="M119"/><axsl:template match="@*|node()" priority="-2" mode="M119"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M119"/></axsl:template>

<!--PATTERN person-group
      Make sure person-group-type is present.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure person-group-type is present.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation/person-group |                     article/front/article-meta/product/person-group" priority="1000" mode="M120"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation/person-group |                     article/front/article-meta/product/person-group"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@person-group-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@person-group-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'person-group': Missing attribute person-group-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) = 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) = 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'person-group': Unexpected text content. 
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M120"/></axsl:template><axsl:template match="text()" priority="-1" mode="M120"/><axsl:template match="@*|node()" priority="-2" mode="M120"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M120"/></axsl:template>

<!--PATTERN person-group-type_values
      person-group/@person-group-type value constraints.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      person-group/@person-group-type value constraints.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation/person-group[@person-group-type] |                     article/front/article-meta/product/person-group[@person-group-type]" priority="1000" mode="M121"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation/person-group[@person-group-type] |                     article/front/article-meta/product/person-group[@person-group-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@person-group-type = 'author' or                     @person-group-type = 'compiler' or                      @person-group-type = 'editor' or                     @person-group-type = 'translator'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@person-group-type = 'author' or @person-group-type = 'compiler' or @person-group-type = 'editor' or @person-group-type = 'translator'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'person-group', attribute person-group-type: Invalid value '<axsl:text/><axsl:value-of select="@person-group-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M121"/></axsl:template><axsl:template match="text()" priority="-1" mode="M121"/><axsl:template match="@*|node()" priority="-2" mode="M121"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M121"/></axsl:template>

<!--PATTERN fn_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/author-notes/fn |                     article/back/fn-group/fn" priority="1000" mode="M122"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/author-notes/fn |                     article/back/fn-group/fn"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@fn-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@fn-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'fn': Missing attribute fn-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M122"/></axsl:template><axsl:template match="text()" priority="-1" mode="M122"/><axsl:template match="@*|node()" priority="-2" mode="M122"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M122"/></axsl:template>

<!--PATTERN fn
      Make sure fn-type is valid against a white list.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure fn-type is valid against a white list.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/fn-group/fn[@fn-type]" priority="1001" mode="M123"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/fn-group/fn[@fn-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@fn-type = 'abbr' or                     @fn-type = 'com' or                      @fn-type = 'financial-disclosure' or                     @fn-type = 'supported-by' or                     @fn-type = 'presented-at' or                     @fn-type = 'supplementary-material' or                     @fn-type = 'other'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@fn-type = 'abbr' or @fn-type = 'com' or @fn-type = 'financial-disclosure' or @fn-type = 'supported-by' or @fn-type = 'presented-at' or @fn-type = 'supplementary-material' or @fn-type = 'other'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'fn', attribute fn-type: Invalid value '<axsl:text/><axsl:value-of select="@fn-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M123"/></axsl:template>

	<!--RULE -->
<axsl:template match="article/front/article-meta/author-notes/fn[@fn-type]" priority="1000" mode="M123"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/author-notes/fn[@fn-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@fn-type = 'author' or                     @fn-type = 'con' or                     @fn-type = 'conflict' or                     @fn-type = 'corresp' or                     @fn-type = 'current-aff' or                     @fn-type = 'deceased' or                     @fn-type = 'edited-by' or                      @fn-type = 'equal' or                     @fn-type = 'on-leave' or                     @fn-type = 'participating-researchers' or                     @fn-type = 'present-address' or                     @fn-type = 'previously-at' or                     @fn-type = 'study-group-members' or                     @fn-type = 'other' or                     @fn-type = 'presented-at' or                      @fn-type = 'presented-by'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@fn-type = 'author' or @fn-type = 'con' or @fn-type = 'conflict' or @fn-type = 'corresp' or @fn-type = 'current-aff' or @fn-type = 'deceased' or @fn-type = 'edited-by' or @fn-type = 'equal' or @fn-type = 'on-leave' or @fn-type = 'participating-researchers' or @fn-type = 'present-address' or @fn-type = 'previously-at' or @fn-type = 'study-group-members' or @fn-type = 'other' or @fn-type = 'presented-at' or @fn-type = 'presented-by'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'fn', attribute fn-type: Invalid value '<axsl:text/><axsl:value-of select="@fn-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M123"/></axsl:template><axsl:template match="text()" priority="-1" mode="M123"/><axsl:template match="@*|node()" priority="-2" mode="M123"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M123"/></axsl:template>

<!--PATTERN fn-group-->


	<!--RULE -->
<axsl:template match="article/back/fn-group" priority="1000" mode="M124"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/fn-group"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(title) &lt; 2"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(title) &lt; 2"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'fn-group': There must be zero or one element title.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M124"/></axsl:template><axsl:template match="text()" priority="-1" mode="M124"/><axsl:template match="@*|node()" priority="-2" mode="M124"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M124"/></axsl:template>

<!--PATTERN xhtml-table
      Tables should be fully tagged. tr elements are not supposed to be declared
      at toplevel.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Tables should be fully tagged. tr elements are not supposed to be declared
      at toplevel.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//table" priority="1000" mode="M125"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//table"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(tr)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(tr)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'table': Unexpected element tr.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(tbody//th)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(tbody//th)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'table': Unexpected element th inside tbody.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(thead//td)"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(thead//td)"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'table': Unexpected element td inside thead.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M125"/></axsl:template><axsl:template match="text()" priority="-1" mode="M125"/><axsl:template match="@*|node()" priority="-2" mode="M125"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M125"/></axsl:template>

<!--PATTERN supplementary-material_mimetype
      The attributes mimetype and mime-subtype are required.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      The attributes mimetype and mime-subtype are required.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//supplementary-material" priority="1000" mode="M126"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//supplementary-material"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@mimetype"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@mimetype"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'supplementary-material': Missing attribute mimetype.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@mime-subtype"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@mime-subtype"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'supplementary-material': Missing attribute mime-subtype.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M126"/></axsl:template><axsl:template match="text()" priority="-1" mode="M126"/><axsl:template match="@*|node()" priority="-2" mode="M126"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M126"/></axsl:template>

<!--PATTERN article_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article" priority="1000" mode="M127"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@article-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@article-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article': Missing attribute article-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xml:lang"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xml:lang"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article': Missing attribute xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@dtd-version"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@dtd-version"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article': Missing attribute dtd-version.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@specific-use"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@specific-use"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article': Missing SPS version at the attribute specific-use.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M127"/></axsl:template><axsl:template match="text()" priority="-1" mode="M127"/><axsl:template match="@*|node()" priority="-2" mode="M127"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M127"/></axsl:template>

<!--PATTERN article_article-type-values
      Allowed values for article/@article-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for article/@article-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="article[@article-type]" priority="1000" mode="M128"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article[@article-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@article-type = 'addendum' or             @article-type = 'research-article' or             @article-type = 'review-article' or             @article-type = 'letter' or             @article-type = 'article-commentary' or             @article-type = 'brief-report' or             @article-type = 'rapid-communication' or             @article-type = 'oration' or             @article-type = 'discussion' or             @article-type = 'editorial' or             @article-type = 'interview' or             @article-type = 'correction' or             @article-type = 'guidelines' or             @article-type = 'other' or             @article-type = 'obituary' or             @article-type = 'case-report' or             @article-type = 'book-review' or             @article-type = 'reply' or             @article-type = 'retraction' or             @article-type = 'partial-retraction' or             @article-type = 'clinical-trial' or             @article-type = 'announcement' or             @article-type = 'calendar' or             @article-type = 'in-brief' or             @article-type = 'book-received' or             @article-type = 'news' or             @article-type = 'reprint' or             @article-type = 'meeting-report' or             @article-type = 'abstract' or             @article-type = 'product-review' or             @article-type = 'dissertation' or             @article-type = 'translation' or             @article-type = 'data-article'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@article-type = 'addendum' or @article-type = 'research-article' or @article-type = 'review-article' or @article-type = 'letter' or @article-type = 'article-commentary' or @article-type = 'brief-report' or @article-type = 'rapid-communication' or @article-type = 'oration' or @article-type = 'discussion' or @article-type = 'editorial' or @article-type = 'interview' or @article-type = 'correction' or @article-type = 'guidelines' or @article-type = 'other' or @article-type = 'obituary' or @article-type = 'case-report' or @article-type = 'book-review' or @article-type = 'reply' or @article-type = 'retraction' or @article-type = 'partial-retraction' or @article-type = 'clinical-trial' or @article-type = 'announcement' or @article-type = 'calendar' or @article-type = 'in-brief' or @article-type = 'book-received' or @article-type = 'news' or @article-type = 'reprint' or @article-type = 'meeting-report' or @article-type = 'abstract' or @article-type = 'product-review' or @article-type = 'dissertation' or @article-type = 'translation' or @article-type = 'data-article'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article', attribute article-type: Invalid value '<axsl:text/><axsl:value-of select="@article-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M128"/></axsl:template><axsl:template match="text()" priority="-1" mode="M128"/><axsl:template match="@*|node()" priority="-2" mode="M128"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M128"/></axsl:template>

<!--PATTERN article_specific-use-values
      The SPS version must be declared in article/@specific-use 
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      The SPS version must be declared in article/@specific-use 
    </svrl:text>

	<!--RULE -->
<axsl:template match="article[@specific-use]" priority="1000" mode="M129"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article[@specific-use]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@specific-use = 'sps-1.9'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@specific-use = 'sps-1.9'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article', attribute specific-use: Invalid value '<axsl:text/><axsl:value-of select="@specific-use"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M129"/></axsl:template><axsl:template match="text()" priority="-1" mode="M129"/><axsl:template match="@*|node()" priority="-2" mode="M129"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M129"/></axsl:template>

<!--PATTERN named-content_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/aff/addr-line/named-content" priority="1000" mode="M130"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/aff/addr-line/named-content"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@content-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@content-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'named-content': Missing attribute content-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M130"/></axsl:template><axsl:template match="text()" priority="-1" mode="M130"/><axsl:template match="@*|node()" priority="-2" mode="M130"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M130"/></axsl:template>

<!--PATTERN named-content_content-type-values
      Allowed values for named-content/@content-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for named-content/@content-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/aff/addr-line/named-content[@content-type]" priority="1000" mode="M131"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/aff/addr-line/named-content[@content-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@content-type = 'city' or                     @content-type = 'state'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@content-type = 'city' or @content-type = 'state'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'named-content', attribute content-type: Invalid value '<axsl:text/><axsl:value-of select="@content-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M131"/></axsl:template><axsl:template match="text()" priority="-1" mode="M131"/><axsl:template match="@*|node()" priority="-2" mode="M131"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M131"/></axsl:template>

<!--PATTERN month
      Only integers between 1 and 12.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Only integers between 1 and 12.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//month" priority="1000" mode="M132"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//month"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="regexp:test(current(), '^(0?[1-9]{1}|[10-12]{2})$')"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="regexp:test(current(), '^(0?[1-9]{1}|[10-12]{2})$')"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'month': Invalid value '<axsl:text/><axsl:value-of select="current()"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M132"/></axsl:template><axsl:template match="text()" priority="-1" mode="M132"/><axsl:template match="@*|node()" priority="-2" mode="M132"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M132"/></axsl:template>

<!--PATTERN size_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/product/size |                     article/back/ref-list/ref/element-citation/size" priority="1000" mode="M133"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/product/size |                     article/back/ref-list/ref/element-citation/size"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@units"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@units"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'size': Missing attribute units.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M133"/></axsl:template><axsl:template match="text()" priority="-1" mode="M133"/><axsl:template match="@*|node()" priority="-2" mode="M133"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M133"/></axsl:template>

<!--PATTERN size_units-values
      Allowed values for size/@units
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for size/@units
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/product/size[@units] |                     article/back/ref-list/ref/element-citation/size[@units]" priority="1000" mode="M134"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/product/size[@units] |                     article/back/ref-list/ref/element-citation/size[@units]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@units = 'pages'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@units = 'pages'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'size', attribute units: Invalid value '<axsl:text/><axsl:value-of select="@units"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M134"/></axsl:template><axsl:template match="text()" priority="-1" mode="M134"/><axsl:template match="@*|node()" priority="-2" mode="M134"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M134"/></axsl:template>

<!--PATTERN list_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="//list" priority="1000" mode="M135"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//list"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@list-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@list-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'list': Missing attribute list-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M135"/></axsl:template><axsl:template match="text()" priority="-1" mode="M135"/><axsl:template match="@*|node()" priority="-2" mode="M135"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M135"/></axsl:template>

<!--PATTERN list_list-type-values
      Allowed values for list/@list-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for list/@list-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="//list[@list-type]" priority="1000" mode="M136"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//list[@list-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@list-type = 'order' or                     @list-type = 'bullet' or                     @list-type = 'alpha-lower' or                     @list-type = 'alpha-upper' or                     @list-type = 'roman-lower' or                     @list-type = 'roman-upper' or                     @list-type = 'simple'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@list-type = 'order' or @list-type = 'bullet' or @list-type = 'alpha-lower' or @list-type = 'alpha-upper' or @list-type = 'roman-lower' or @list-type = 'roman-upper' or @list-type = 'simple'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'list', attribute list-type: Invalid value '<axsl:text/><axsl:value-of select="@list-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M136"/></axsl:template><axsl:template match="text()" priority="-1" mode="M136"/><axsl:template match="@*|node()" priority="-2" mode="M136"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M136"/></axsl:template>

<!--PATTERN media_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="//media" priority="1000" mode="M137"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//media"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@mime-subtype"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@mime-subtype"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'media': Missing attribute mime-subtype.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@mimetype"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@mimetype"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'media': Missing attribute mimetype.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xlink:href"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xlink:href"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'media': Missing attribute xlink:href.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M137"/></axsl:template><axsl:template match="text()" priority="-1" mode="M137"/><axsl:template match="@*|node()" priority="-2" mode="M137"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M137"/></axsl:template>

<!--PATTERN ext-link_attributes
      Make sure some attributes are present. Also, the value of 
      ext-link-type is validated against a white-list.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present. Also, the value of 
      ext-link-type is validated against a white-list.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//ext-link" priority="1000" mode="M138"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//ext-link"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@ext-link-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@ext-link-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ext-link': Missing attribute ext-link-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@ext-link-type = 'uri' or                     @ext-link-type = 'clinical-trial'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@ext-link-type = 'uri' or @ext-link-type = 'clinical-trial'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ext-link', attribute ext-link-type: Invalid value '<axsl:text/><axsl:value-of select="@ext-link-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xlink:href"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xlink:href"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ext-link': Missing attribute xlink:href.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M138"/></axsl:template><axsl:template match="text()" priority="-1" mode="M138"/><axsl:template match="@*|node()" priority="-2" mode="M138"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M138"/></axsl:template>

<!--PATTERN ext-link_href_values
        All URIs are supported, except "file", because we dont want references 
        to local files.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
        All URIs are supported, except "file", because we dont want references 
        to local files.
    </svrl:text>

	<!--RULE -->
<axsl:template match="//ext-link[@ext-link-type='uri']" priority="1000" mode="M139"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//ext-link[@ext-link-type='uri']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="regexp:test(@xlink:href, '^[a-zA-Z][a-zA-Z0-9+\.-]+:', 'i')"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="regexp:test(@xlink:href, '^[a-zA-Z][a-zA-Z0-9+\.-]+:', 'i')"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ext-link', attribute xlink:href: Missing URI scheme in '<axsl:text/><axsl:value-of select="@xlink:href"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(starts-with(@xlink:href, 'file:'))"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(starts-with(@xlink:href, 'file:'))"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ext-link', attribute xlink:href: Invalid URI scheme 'file'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M139"/></axsl:template><axsl:template match="text()" priority="-1" mode="M139"/><axsl:template match="@*|node()" priority="-2" mode="M139"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M139"/></axsl:template>

<!--PATTERN element-citation_attributes
      Make sure some attributes are present. 
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present. 
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation" priority="1000" mode="M140"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@publication-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@publication-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'element-citation': Missing attribute publication-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M140"/></axsl:template><axsl:template match="text()" priority="-1" mode="M140"/><axsl:template match="@*|node()" priority="-2" mode="M140"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M140"/></axsl:template>

<!--PATTERN element-citation_publication-type-values
      Allowed values for element-citation/@publication-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for element-citation/@publication-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation[@publication-type]" priority="1000" mode="M141"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation[@publication-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@publication-type = 'journal' or                     @publication-type = 'book' or                     @publication-type = 'webpage' or                     @publication-type = 'thesis' or                     @publication-type = 'confproc' or                     @publication-type = 'patent' or                     @publication-type = 'report' or                     @publication-type = 'software' or                     @publication-type = 'legal-doc' or                     @publication-type = 'newspaper' or                     @publication-type = 'other' or                     @publication-type = 'database' or                     @publication-type = 'data'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@publication-type = 'journal' or @publication-type = 'book' or @publication-type = 'webpage' or @publication-type = 'thesis' or @publication-type = 'confproc' or @publication-type = 'patent' or @publication-type = 'report' or @publication-type = 'software' or @publication-type = 'legal-doc' or @publication-type = 'newspaper' or @publication-type = 'other' or @publication-type = 'database' or @publication-type = 'data'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'element-citation', attribute publication-type: Invalid value '<axsl:text/><axsl:value-of select="@publication-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M141"/></axsl:template><axsl:template match="text()" priority="-1" mode="M141"/><axsl:template match="@*|node()" priority="-2" mode="M141"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M141"/></axsl:template>

<!--PATTERN sub-article_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//sub-article" priority="1000" mode="M142"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//sub-article"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@article-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@article-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'sub-article': Missing attribute article-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xml:lang"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xml:lang"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'sub-article': Missing attribute xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'sub-article': Missing attribute id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M142"/></axsl:template><axsl:template match="text()" priority="-1" mode="M142"/><axsl:template match="@*|node()" priority="-2" mode="M142"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M142"/></axsl:template>

<!--PATTERN sub-article_article-type-values
      Allowed values for //sub-article/@article-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for //sub-article/@article-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="//sub-article[@article-type]" priority="1000" mode="M143"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//sub-article[@article-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@article-type = 'abstract' or                      @article-type = 'letter' or                      @article-type = 'reply' or                      @article-type = 'translation'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@article-type = 'abstract' or @article-type = 'letter' or @article-type = 'reply' or @article-type = 'translation'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'sub-article', attribute article-type: Invalid value '<axsl:text/><axsl:value-of select="@article-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M143"/></axsl:template><axsl:template match="text()" priority="-1" mode="M143"/><axsl:template match="@*|node()" priority="-2" mode="M143"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M143"/></axsl:template>

<!--PATTERN response_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//response" priority="1000" mode="M144"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//response"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@response-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@response-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'response': Missing attribute response-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xml:lang"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xml:lang"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'response': Missing attribute xml:lang.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'response': Missing attribute id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M144"/></axsl:template><axsl:template match="text()" priority="-1" mode="M144"/><axsl:template match="@*|node()" priority="-2" mode="M144"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M144"/></axsl:template>

<!--PATTERN response_response-type-values
      Allowed values for //response/@response-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for //response/@response-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="//response[@response-type]" priority="1000" mode="M145"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//response[@response-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@response-type = 'addendum' or                      @response-type = 'discussion' or                      @response-type = 'reply'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@response-type = 'addendum' or @response-type = 'discussion' or @response-type = 'reply'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'response', attribute response-type: Invalid value '<axsl:text/><axsl:value-of select="@response-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M145"/></axsl:template><axsl:template match="text()" priority="-1" mode="M145"/><axsl:template match="@*|node()" priority="-2" mode="M145"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M145"/></axsl:template>

<!--PATTERN response_related-article_attributes
      Make sure some attributes are present

      @id is mandatory for all use cases, so it is not implemented in this 
      pattern.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present

      @id is mandatory for all use cases, so it is not implemented in this 
      pattern.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//response[@response-type='reply']//related-article" priority="1000" mode="M146"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//response[@response-type='reply']//related-article"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@related-article-type = 'commentary-article'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@related-article-type = 'commentary-article'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article': Missing attribute related-article-type of type 'commentary-article'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@vol"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@vol"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article': Missing attribute vol.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@page or @elocation-id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@page or @elocation-id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article': Missing attribute page or elocation-id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M146"/></axsl:template><axsl:template match="text()" priority="-1" mode="M146"/><axsl:template match="@*|node()" priority="-2" mode="M146"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M146"/></axsl:template>

<!--PATTERN related-article_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//related-article" priority="1000" mode="M147"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//related-article"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@related-article-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@related-article-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article': Missing attribute related-article-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@id"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@id"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article': Missing attribute id.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M147"/></axsl:template><axsl:template match="text()" priority="-1" mode="M147"/><axsl:template match="@*|node()" priority="-2" mode="M147"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M147"/></axsl:template>

<!--PATTERN related-article_correction_attributes
      Make sure some attributes are present for corrections.
      http://docs.scielo.org/projects/scielo-publishing-schema/pt_BR/1.5-branch/narr/errata.html#errata
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present for corrections.
      http://docs.scielo.org/projects/scielo-publishing-schema/pt_BR/1.5-branch/narr/errata.html#errata
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//related-article[@related-article-type = 'corrected-article']" priority="1000" mode="M148"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//related-article[@related-article-type = 'corrected-article']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@ext-link-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@ext-link-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article': Missing attribute ext-link-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@xlink:href"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@xlink:href"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article': Missing attribute xlink:href.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M148"/></axsl:template><axsl:template match="text()" priority="-1" mode="M148"/><axsl:template match="@*|node()" priority="-2" mode="M148"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M148"/></axsl:template>

<!--PATTERN related-article-type-values
      Allowed values for //related-article/@related-article-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for //related-article/@related-article-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="//related-article[@related-article-type]" priority="1000" mode="M149"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="//related-article[@related-article-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@related-article-type = 'corrected-article' or                      @related-article-type = 'commentary-article' or                     @related-article-type = 'letter' or                     @related-article-type = 'partial-retraction' or                     @related-article-type = 'retracted-article'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@related-article-type = 'corrected-article' or @related-article-type = 'commentary-article' or @related-article-type = 'letter' or @related-article-type = 'partial-retraction' or @related-article-type = 'retracted-article'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article', attribute related-article-type: Invalid value '<axsl:text/><axsl:value-of select="@related-article-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M149"/></axsl:template><axsl:template match="text()" priority="-1" mode="M149"/><axsl:template match="@*|node()" priority="-2" mode="M149"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M149"/></axsl:template>

<!--PATTERN correction_related-article
      Articles of type correction must specify the article it relates to.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Articles of type correction must specify the article it relates to.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article[@article-type='correction']/front/article-meta" priority="1000" mode="M150"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article[@article-type='correction']/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="related-article/@related-article-type = 'corrected-article'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="related-article/@related-article-type = 'corrected-article'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing element related-article with related-article-type='corrected-article'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M150"/></axsl:template><axsl:template match="text()" priority="-1" mode="M150"/><axsl:template match="@*|node()" priority="-2" mode="M150"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M150"/></axsl:template>

<!--PATTERN correction_article-type
      Ensure related-article[@related-article-type='corrected-article' is 
      defined only for correction article types.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Ensure related-article[@related-article-type='corrected-article' is 
      defined only for correction article types.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article[@article-type != 'correction']/front/article-meta/related-article" priority="1000" mode="M151"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article[@article-type != 'correction']/front/article-meta/related-article"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(@related-article-type = 'corrected-article')"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(@related-article-type = 'corrected-article')"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article', attribute related-article-type: Invalid value 'corrected-article' for article-type '<axsl:text/><axsl:value-of select="/article/@article-type"/><axsl:text/>'. 
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M151"/></axsl:template><axsl:template match="text()" priority="-1" mode="M151"/><axsl:template match="@*|node()" priority="-2" mode="M151"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M151"/></axsl:template>

<!--PATTERN inbrief_related-article
      Articles of type in-brief must specify the article it relates to.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Articles of type in-brief must specify the article it relates to.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article[@article-type='in-brief']/front/article-meta" priority="1000" mode="M152"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article[@article-type='in-brief']/front/article-meta"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="related-article/@related-article-type = 'article-reference'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="related-article/@related-article-type = 'article-reference'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'article-meta': Missing element related-article with related-article-type='article-reference'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M152"/></axsl:template><axsl:template match="text()" priority="-1" mode="M152"/><axsl:template match="@*|node()" priority="-2" mode="M152"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M152"/></axsl:template>

<!--PATTERN inbrief_article-type
      Ensure related-article[@related-article-type='article-reference' is 
      defined only for in-brief article types.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Ensure related-article[@related-article-type='article-reference' is 
      defined only for in-brief article types.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article[@article-type != 'in-brief']/front/article-meta/related-article" priority="1000" mode="M153"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article[@article-type != 'in-brief']/front/article-meta/related-article"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="not(@related-article-type = 'article-reference')"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="not(@related-article-type = 'article-reference')"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article', attribute related-article-type: Invalid value 'article-reference' for article-type '<axsl:text/><axsl:value-of select="/article/@article-type"/><axsl:text/>'. 
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M153"/></axsl:template><axsl:template match="text()" priority="-1" mode="M153"/><axsl:template match="@*|node()" priority="-2" mode="M153"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M153"/></axsl:template>

<!--PATTERN funding-group-->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron"/>

	<!--RULE -->
<axsl:template match="article/back//fn[@fn-type='financial-disclosure']" priority="1000" mode="M154"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back//fn[@fn-type='financial-disclosure']"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="/article/front/article-meta/funding-group/funding-statement"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="/article/front/article-meta/funding-group/funding-statement"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'fn': Missing element funding-statement.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M154"/></axsl:template><axsl:template match="text()" priority="-1" mode="M154"/><axsl:template match="@*|node()" priority="-2" mode="M154"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M154"/></axsl:template>

<!--PATTERN funding-group_elements
      Make sure mandatory child elements are present.  
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure mandatory child elements are present.  
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/funding-group" priority="1000" mode="M155"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/funding-group"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="award-group"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="award-group"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'funding-group': Missing element award-group.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M155"/></axsl:template><axsl:template match="text()" priority="-1" mode="M155"/><axsl:template match="@*|node()" priority="-2" mode="M155"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M155"/></axsl:template>

<!--PATTERN aff_country-attrs
      Ensure the attribute 'country' is present for all //aff/country elements.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Ensure the attribute 'country' is present for all //aff/country elements.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/aff/country" priority="1000" mode="M156"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/aff/country"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@country"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@country"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'country': Missing attribute country.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M156"/></axsl:template><axsl:template match="text()" priority="-1" mode="M156"/><axsl:template match="@*|node()" priority="-2" mode="M156"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M156"/></axsl:template>

<!--PATTERN aff_country
      //aff/country elements cannot be empty.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      //aff/country elements cannot be empty.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/aff/country" priority="1000" mode="M157"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/aff/country"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) &gt; 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) &gt; 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'country': Missing text content.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M157"/></axsl:template><axsl:template match="text()" priority="-1" mode="M157"/><axsl:template match="@*|node()" priority="-2" mode="M157"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M157"/></axsl:template>

<!--PATTERN ref
      element-citation and mixed-citation are mandatory in ref.
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      element-citation and mixed-citation are mandatory in ref.
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ref-list/ref" priority="1000" mode="M158"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="mixed-citation"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="mixed-citation"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ref': Missing element mixed-citation.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose>

		<!--ASSERT -->
<axsl:choose><axsl:when test="element-citation"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="element-citation"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'ref': Missing element element-citation.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M158"/></axsl:template><axsl:template match="text()" priority="-1" mode="M158"/><axsl:template match="@*|node()" priority="-2" mode="M158"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M158"/></axsl:template>

<!--PATTERN ref_notempty
    Check if the element's text is at least one character long.
  -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
    Check if the element's text is at least one character long.
  </svrl:text>

	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/mixed-citation" priority="1000" mode="M159"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/mixed-citation"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="string-length(normalize-space(text())) != 0"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="string-length(normalize-space(text())) != 0"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
      Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': <axsl:text/><axsl:value-of select="'Element cannot be empty.'"/><axsl:text/>
    </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M159"/></axsl:template><axsl:template match="text()" priority="-1" mode="M159"/><axsl:template match="@*|node()" priority="-2" mode="M159"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M159"/></axsl:template>

<!--PATTERN contrib-id_attributes
      Make sure some attributes are present
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Make sure some attributes are present
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//contrib-group/contrib-id" priority="1000" mode="M160"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//contrib-group/contrib-id"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@contrib-id-type"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@contrib-id-type"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'contrib-id': Missing attribute contrib-id-type.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M160"/></axsl:template><axsl:template match="text()" priority="-1" mode="M160"/><axsl:template match="@*|node()" priority="-2" mode="M160"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M160"/></axsl:template>

<!--PATTERN contrib-id-type-values
      Allowed values for //contrib/contrib-id/@contrib-id-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for //contrib/contrib-id/@contrib-id-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="article//contrib-group/contrib-id[@contrib-id-type]" priority="1000" mode="M161"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article//contrib-group/contrib-id[@contrib-id-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@contrib-id-type = 'lattes' or                      @contrib-id-type = 'orcid' or                      @contrib-id-type = 'researchid' or                     @contrib-id-type = 'scopus'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@contrib-id-type = 'lattes' or @contrib-id-type = 'orcid' or @contrib-id-type = 'researchid' or @contrib-id-type = 'scopus'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'contrib-id', attribute contrib-id-type: Invalid value '<axsl:text/><axsl:value-of select="@contrib-id-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M161"/></axsl:template><axsl:template match="text()" priority="-1" mode="M161"/><axsl:template match="@*|node()" priority="-2" mode="M161"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M161"/></axsl:template>

<!--PATTERN source_cardinality-->


	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation |                                        article/front/article-meta/product" priority="1000" mode="M162"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation |                                        article/front/article-meta/product"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(source) &lt; 2"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(source) &lt; 2"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be zero or one element <axsl:text/><axsl:value-of select="name(source)"/><axsl:text/>.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M162"/></axsl:template><axsl:template match="text()" priority="-1" mode="M162"/><axsl:template match="@*|node()" priority="-2" mode="M162"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M162"/></axsl:template>

<!--PATTERN size_cardinality-->


	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation |                                        article/front/article-meta/product" priority="1000" mode="M163"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation |                                        article/front/article-meta/product"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(size) &lt; 2"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(size) &lt; 2"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be zero or one element <axsl:text/><axsl:value-of select="name(size)"/><axsl:text/>.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M163"/></axsl:template><axsl:template match="text()" priority="-1" mode="M163"/><axsl:template match="@*|node()" priority="-2" mode="M163"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M163"/></axsl:template>

<!--PATTERN month_cardinality-->


	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation" priority="1000" mode="M164"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(month) &lt; 2"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(month) &lt; 2"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be zero or one element <axsl:text/><axsl:value-of select="name(month)"/><axsl:text/>.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M164"/></axsl:template><axsl:template match="text()" priority="-1" mode="M164"/><axsl:template match="@*|node()" priority="-2" mode="M164"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M164"/></axsl:template>

<!--PATTERN issue_cardinality-->


	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation" priority="1000" mode="M165"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(issue) &lt; 2"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(issue) &lt; 2"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be zero or one element <axsl:text/><axsl:value-of select="name(issue)"/><axsl:text/>.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M165"/></axsl:template><axsl:template match="text()" priority="-1" mode="M165"/><axsl:template match="@*|node()" priority="-2" mode="M165"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M165"/></axsl:template>

<!--PATTERN chapter-title_cardinality-->


	<!--RULE -->
<axsl:template match="article/back/ref-list/ref/element-citation |                                        article/front/article-meta/product" priority="1000" mode="M166"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref/element-citation |                                        article/front/article-meta/product"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(chapter-title) &lt; 2"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(chapter-title) &lt; 2"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be zero or one element <axsl:text/><axsl:value-of select="name(chapter-title)"/><axsl:text/>.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M166"/></axsl:template><axsl:template match="text()" priority="-1" mode="M166"/><axsl:template match="@*|node()" priority="-2" mode="M166"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M166"/></axsl:template>

<!--PATTERN element-citation_cardinality-->


	<!--RULE -->
<axsl:template match="article/back/ref-list/ref" priority="1000" mode="M167"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/back/ref-list/ref"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="count(element-citation) = 1"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="count(element-citation) = 1"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element '<axsl:text/><axsl:value-of select="name(.)"/><axsl:text/>': There must be only one element <axsl:text/><axsl:value-of select="name(element-citation)"/><axsl:text/>.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M167"/></axsl:template><axsl:template match="text()" priority="-1" mode="M167"/><axsl:template match="@*|node()" priority="-2" mode="M167"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M167"/></axsl:template>

<!--PATTERN related-article_ext-link-type-values
      Allowed values for //related-article/@ext-link-type
    -->
<svrl:text xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron">
      Allowed values for //related-article/@ext-link-type
    </svrl:text>

	<!--RULE -->
<axsl:template match="article/front/article-meta/related-article[@ext-link-type]" priority="1000" mode="M168"><svrl:fired-rule xmlns:svrl="http://purl.oclc.org/dsdl/svrl" context="article/front/article-meta/related-article[@ext-link-type]"/>

		<!--ASSERT -->
<axsl:choose><axsl:when test="@ext-link-type = 'doi'"/><axsl:otherwise><svrl:failed-assert xmlns:svrl="http://purl.oclc.org/dsdl/svrl" xmlns:xs="http://www.w3.org/2001/XMLSchema" xmlns:schold="http://www.ascc.net/xml/schematron" test="@ext-link-type = 'doi'"><axsl:attribute name="location"><axsl:apply-templates select="." mode="schematron-get-full-path"/></axsl:attribute><svrl:text>
        Element 'related-article', attribute ext-link-type: Invalid value '<axsl:text/><axsl:value-of select="@ext-link-type"/><axsl:text/>'.
      </svrl:text></svrl:failed-assert></axsl:otherwise></axsl:choose><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M168"/></axsl:template><axsl:template match="text()" priority="-1" mode="M168"/><axsl:template match="@*|node()" priority="-2" mode="M168"><axsl:apply-templates select="@*|*|comment()|processing-instruction()" mode="M168"/></axsl:template></axsl:stylesheet>