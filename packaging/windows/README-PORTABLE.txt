O2XML PARA WINDOWS — VERSÃO PORTÁTIL
====================================

1. Extraia todo o conteúdo do arquivo ZIP para uma pasta local.
2. Mantenha O2XML.exe, xml-libxml2-helper.exe e a pasta resources juntos.
3. Execute O2XML.exe. Não é necessário instalar nem usar privilégios de
   administrador.

Requisitos
----------

- Windows 10 ou Windows 11 de 64 bits.
- Microsoft Edge WebView2 Runtime instalado. Ele já acompanha a maioria das
  instalações atualizadas do Windows 10 e 11.

Segurança e integridade
-----------------------

- Compare o SHA-256 do ZIP com o arquivo .sha256 publicado na mesma release.
- O pacote não deve ser executado diretamente de dentro do ZIP.
- Builds ainda não assinados podem exibir um aviso do Microsoft Defender
  SmartScreen. A assinatura Authenticode será necessária para distribuição
  pública sem esse aviso de editor desconhecido.
- Não remova o helper XML: validação DTD/XSD/RELAX NG, XPath e transformações
  dependem dele.

Licença
-------

O O2XML é disponibilizado sob licença MIT ou Apache-2.0, à escolha do usuário.
Consulte LICENSE-MIT, LICENSE-APACHE e DEPENDENCIES.md incluídos neste pacote.
