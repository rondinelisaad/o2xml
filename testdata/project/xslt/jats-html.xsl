<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0"
  xmlns:xsl="http://www.w3.org/1999/XSL/Transform">
  <xsl:output method="html" encoding="UTF-8"/>
  <xsl:param name="article_lang"/>
  <xsl:template match="/article">
    <html>
      <head>
        <title><xsl:value-of select="front/article-meta/title-group/article-title"/></title>
      </head>
      <body>
        <h1><xsl:value-of select="front/article-meta/title-group/article-title"/></h1>
        <p id="article-language"><xsl:value-of select="$article_lang"/></p>
      </body>
    </html>
  </xsl:template>
</xsl:stylesheet>
