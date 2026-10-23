# Manual de validação de XML no O2XML 0.3.1

Este manual descreve como abrir, validar e corrigir um documento XML na versão
0.3.1 do O2XML. O exemplo utilizado é o artigo
`0001-3765-aabc-98-03-e20251145.xml`, fornecido para teste.

> **Escopo da versão 0.3.1:** o O2XML verifica a boa formação de qualquer XML e
> valida automaticamente documentos que declaram as DTDs JATS Publishing 1.1 ou
> 1.3 incluídas no aplicativo. A conformidade com o SciELO Publishing Schema
> (SPS) é uma etapa separada e requer um projeto configurado com o catálogo, a
> gramática e as regras Schematron oficiais da versão SPS correspondente.

## 1. Antes de começar

1. Instale ou extraia o pacote do O2XML 0.3.1 adequado ao sistema operacional.
   No Windows, o pacote portátil pode ser extraído em qualquer pasta gravável;
   mantenha `O2XML.exe`, o helper e a pasta `resources` juntos.
2. Preserve uma cópia do XML original antes de corrigir erros.
3. Abra o O2XML e confirme **Idioma → Português (Brasil)**.

O aplicativo processa recursos XML locais e empacotados com política offline.
Ele não baixa automaticamente DTDs, schemas ou regras da internet.

## 2. Abrir o arquivo de exemplo

Há duas formas de abrir o documento:

- clique em **Abrir…**, selecione
  `0001-3765-aabc-98-03-e20251145.xml` e confirme; ou
- arraste o arquivo para a janela do O2XML.

O botão **Abrir…** aceita vários arquivos na mesma seleção. Cada documento é
exibido em uma aba e pode ser fechado pelo botão **×** da própria aba. Os
atalhos `Ctrl+O`/`Cmd+O`, `Ctrl+S`/`Cmd+S` e `Ctrl+W`/`Cmd+W` abrem, salvam e
fecham o documento ativo.

Depois da abertura, o O2XML inicia a análise automaticamente. Durante a edição,
uma nova análise é agendada após uma pequena pausa de digitação. O botão
**Validar** força uma nova análise imediatamente.

## 3. Entender os três níveis de validação

| Resultado | O que foi verificado | Como identificar |
|---|---|---|
| XML bem formado | Sintaxe XML, abertura e fechamento de elementos, atributos, entidades e caracteres | Barra de status e painel **Problemas** |
| JATS válido | Estrutura e valores permitidos pela DTD JATS declarada | **Framework de validação → JATS 1.1: aprovado** |
| SPS avaliado | Regras adicionais do SPS | Etapa **SPS** aprovada ou reprovada, independentemente do resultado JATS |

Um XML bem formado não é necessariamente JATS válido. Da mesma forma, um XML
JATS válido não é necessariamente compatível com o SPS.

## 4. Validar o exemplo como documento JATS 1.1

O início do arquivo de exemplo contém:

```xml
<!DOCTYPE article PUBLIC
  "-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.1 20151215//EN"
  "http://jats.nlm.nih.gov/publishing/1.1/JATS-journalpublishing1.dtd">
<article article-type="research-article"
         dtd-version="1.1"
         specific-use="sps-1.9"
         xml:lang="en">
```

O identificador público permite que o O2XML reconheça a JATS Publishing 1.1 e
use a cópia local da DTD. Não é necessário acesso à internet.

Procedimento:

1. deixe a aba do exemplo ativa;
2. clique em **Validar**;
3. consulte **Framework de validação**, na barra lateral;
4. abra o painel **Problemas**, na parte inferior;
5. clique em um diagnóstico para posicionar o cursor no trecho correspondente.

O painel **Estrutura** permite navegar pela árvore XML. O **Inspetor JATS**
apresenta as partes editoriais do artigo, como metadados, corpo e referências.

## 5. Resultado observado no arquivo de exemplo

O arquivo é bem formado, mas não passa integralmente na DTD JATS 1.1. A
validação local encontrou na linha 5239:

```xml
<fn fn-type="data-availability"
    specific-use="data-available-upon-request"
    id="fne1">
```

O valor `data-availability` não pertence à lista de valores admitidos para o
atributo `fn-type` na DTD JATS Publishing 1.1. Ele é uma exigência específica
do SPS para esse tipo de nota. Portanto, o resultado esperado da etapa JATS é:

- **JATS 1.1: falhou**;
- um erro no painel **Problemas**, associado à linha 5239; e
- a barra de status indicando falha na validação.

### Como interpretar o conflito JATS × SPS

Não substitua `data-availability` apenas para zerar a etapa JATS. Fazer isso pode
eliminar uma informação exigida pelo SPS e produzir um documento editorialmente
incorreto.

Ao abrir o artigo dentro de um projeto **SPS-1.9**, o O2XML executa as etapas
separadamente:

- a etapa **JATS 1.1** pode falhar por causa da extensão exigida pelo SPS; e
- a etapa **SPS 1.9** continua sendo executada e apresenta seu próprio resultado.

Um erro na etapa JATS não bloqueia a etapa SPS. Somente um XML malformado — por
exemplo, com uma tag não fechada — impede que as validações estruturais seguintes
sejam executadas com segurança. O resultado geral pode continuar indicando
falha porque há um diagnóstico JATS, mas o resultado SPS permanece visível e
deve ser interpretado separadamente.

## 6. Validar como SPS 1.9

O atributo `specific-use="sps-1.9"` informa a intenção do documento, mas não
ativa nem substitui as regras SPS. Para executar as duas etapas separadamente,
prepare um projeto local com os recursos oficiais e suas respectivas licenças:

```text
meu-projeto/
├── project.toml
├── catalog.xml
├── xml/
│   └── 0001-3765-aabc-98-03-e20251145.xml
├── schemas/
│   └── JATS-journalpublishing1.dtd
└── schematron/
    └── sps-1.9.sch
```

Exemplo mínimo de `project.toml`:

```toml
[project]
name = "Validação SPS 1.9"

[xml]
catalog = "catalog.xml"

[validation]
framework = "SPS-1.9"
```

O `catalog.xml` deve resolver os identificadores lógicos esperados pelo O2XML
para os arquivos locais revisados:

```xml
<?xml version="1.0"?>
<catalog xmlns="urn:oasis:names:tc:entity:xmlns:xml:catalog">
  <system
    systemId="urn:o2xml:framework:sps:1.9:jats"
    uri="schemas/JATS-journalpublishing1.dtd"/>
  <uri
    name="urn:o2xml:framework:sps:1.9:schematron"
    uri="schematron/sps-1.9.sch"/>
</catalog>
```

Para usar o projeto:

1. clique em **Projeto…** e selecione `project.toml`;
2. abra o XML localizado dentro da pasta do projeto;
3. clique em **Validar**;
4. confira separadamente as etapas **JATS 1.1** e **SPS 1.9** em **Framework de
   validação**, mesmo que a primeira tenha falhado; e
5. examine os diagnósticos de cada origem no painel **Problemas**.

Se a etapa SPS aparecer como **indisponível**, verifique o catálogo, os caminhos
dos arquivos e a disponibilidade do processador Schematron configurado. A
versão 0.3.1 não baixa nem fornece silenciosamente o conjunto oficial de regras
SPS.

## 7. Documentos SPS 1.10 e JATS 1.3

O O2XML 0.3.1 também inclui a DTD oficial JATS Publishing 1.3 para uso offline.
Um documento SPS 1.10 pode declarar:

```xml
<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE article PUBLIC
  "-//NLM//DTD JATS (Z39.96) Journal Publishing DTD v1.3 20210610//EN"
  "https://jats.nlm.nih.gov/publishing/1.3/JATS-journalpublishing1-3.dtd">
<article xmlns:xlink="http://www.w3.org/1999/xlink"
         xmlns:mml="http://www.w3.org/1998/Math/MathML"
         dtd-version="1.3"
         specific-use="sps-1.10"
         article-type="research-article"
         xml:lang="en">
```

O identificador é reconhecido automaticamente e a validação JATS 1.3 ocorre sem
acesso à internet. Para executar também as regras SPS 1.10, abra um projeto com
`framework = "SPS-1.10"` e mapeie no catálogo os identificadores:

- `urn:o2xml:framework:sps:1.10:jats` para a gramática adotada pelo SPS 1.10; e
- `urn:o2xml:framework:sps:1.10:schematron` para as regras Schematron SPS 1.10.

Assim como no SPS 1.9, as etapas JATS 1.3 e SPS 1.10 são executadas e exibidas
independentemente.

## 8. Interpretar e navegar pelos problemas

Cada linha do painel **Problemas** apresenta:

- gravidade: **ERRO**, **AVISO** ou **INFO**;
- origem, como **boa formação**, **JATS 1.1** ou **SPS 1.9**;
- mensagem do mecanismo de validação; e
- linha e coluna aproximadas no documento.

Clique no problema para navegar ao XML. Corrija um erro por vez, começando por
problemas de boa formação; erros sintáticos podem impedir a execução das etapas
JATS e SPS.

## 9. Conferência final

Antes de entregar o documento, confirme:

- [ ] o arquivo foi salvo com o nome esperado;
- [ ] o contador do painel **Problemas** está em zero ou os avisos remanescentes
      foram justificados;
- [ ] o resultado da etapa JATS correspondente ao `DOCTYPE` foi analisado e
      eventuais incompatibilidades específicas do SPS foram justificadas;
- [ ] a etapa SPS correta foi executada e seu resultado foi avaliado
      separadamente do resultado JATS;
- [ ] as figuras e materiais suplementares referenciados existem junto ao XML;
- [ ] o documento corrigido foi reaberto e validado novamente; e
- [ ] a cópia original foi preservada.

## 10. Limitações importantes da versão 0.3.1

- A validação automática embutida cobre as DTDs JATS Publishing 1.1 e 1.3
  reconhecidas pelo identificador declarado no documento.
- `specific-use="sps-1.9"` não equivale a uma validação SPS.
- Um erro JATS não bloqueia a etapa SPS; somente XML malformado impede as etapas
  de framework.
- Os recursos oficiais SPS precisam ser instalados e mapeados em um catálogo de
  projeto; as licenças e a procedência desses recursos devem ser respeitadas.
- O O2XML mantém recursos remotos bloqueados por padrão. Essa proteção evita
  XXE, acessos arbitrários a arquivos e downloads inesperados durante a
  validação.
- Um resultado sem erros comprova apenas as etapas que aparecem como executadas
  e aprovadas na interface.

