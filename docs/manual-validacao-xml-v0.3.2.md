# Manual de validação de XML no O2XML 0.3.2

Este manual descreve como abrir, validar e corrigir um documento XML na versão
0.3.2 do O2XML. O exemplo utilizado é o artigo
`0001-3765-aabc-98-03-e20251145.xml`, fornecido para teste.

> **Escopo da versão 0.3.2:** o O2XML verifica a boa formação de qualquer XML,
> valida automaticamente documentos que declaram as DTDs JATS Publishing 1.1 ou
> 1.3 e, ao encontrar `specific-use="sps-1.9"` ou `specific-use="sps-1.10"`,
> também executa offline as regras SPS correspondentes. JATS e SPS permanecem
> etapas independentes.

## 1. Antes de começar

1. Instale ou extraia o pacote do O2XML 0.3.2 adequado ao sistema operacional.
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

Ao abrir o artigo, o O2XML reconhece `specific-use="sps-1.9"` e executa as
etapas separadamente:

- a etapa **JATS 1.1** pode falhar por causa da extensão exigida pelo SPS; e
- a etapa **SPS 1.9** continua sendo executada e apresenta seu próprio resultado.

Um erro na etapa JATS não bloqueia a etapa SPS. Somente um XML malformado — por
exemplo, com uma tag não fechada — impede que as validações estruturais seguintes
sejam executadas com segurança. O resultado geral pode continuar indicando
falha porque há um diagnóstico JATS, mas o resultado SPS permanece visível e
deve ser interpretado separadamente.

## 6. Validar como SPS 1.9

Não é mais necessário criar `project.toml`, copiar Schematron ou configurar um
processador externo para o perfil padrão. Confirme que o elemento raiz contém:

```xml
<article dtd-version="1.1" specific-use="sps-1.9" ...>
```

Depois:

1. abra o XML;
2. clique em **Validar**;
3. confira separadamente **JATS 1.1** e **SPS 1.9** em **Framework de
   validação**; e
4. clique em qualquer ocorrência do painel **Problemas** para selecionar a tag
   correspondente no editor.

As regras Packtools/SPS incluídas no aplicativo funcionam offline. A etapa SPS
é executada mesmo quando a etapa JATS contém erros. Apenas um XML malformado
impede as duas etapas estruturais, pois não existe uma árvore XML segura sobre a
qual aplicar DTD ou Schematron.

## 7. Documentos SPS 1.10 e JATS 1.3

O O2XML 0.3.2 também inclui a DTD oficial JATS Publishing 1.3 e as regras SPS
1.10 para uso offline.
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

Os atributos `dtd-version="1.3"` e `specific-use="sps-1.10"` são reconhecidos
automaticamente. Assim como no SPS 1.9, as etapas JATS 1.3 e SPS 1.10 são
executadas offline e exibidas independentemente.

## 8. Interpretar e navegar pelos problemas

Cada linha do painel **Problemas** apresenta:

- gravidade: **ERRO**, **AVISO** ou **INFO**;
- origem, como **boa formação**, **JATS 1.1** ou **SPS 1.9**;
- mensagem do mecanismo de validação; e
- linha e coluna aproximadas no documento.

Clique no problema para navegar ao XML. O2XML rola até a linha e seleciona a tag
associada, o que permite distinguir uma ocorrência mesmo em documentos com
centenas de elementos `<p>`. A DTD e o SPS podem apresentar várias ocorrências
na mesma validação. Para XML malformado, corrija primeiro o erro sintático
indicado e valide novamente: após uma ruptura da árvore, erros posteriores
podem ser apenas consequência do primeiro.

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

## 10. Limitações importantes da versão 0.3.2

- A validação automática embutida cobre as DTDs JATS Publishing 1.1 e 1.3 e os
  perfis SPS 1.9 e 1.10 reconhecidos pelos identificadores do documento.
- `specific-use` seleciona as regras SPS, mas não substitui a validação JATS.
- Um erro JATS não bloqueia a etapa SPS; somente XML malformado impede as etapas
  de framework.
- Schematron arbitrário de projetos continua sendo uma configuração avançada;
  a execução embutida cobre especificamente os perfis SPS versionados incluídos.
- O O2XML mantém recursos remotos bloqueados por padrão. Essa proteção evita
  XXE, acessos arbitrários a arquivos e downloads inesperados durante a
  validação.
- Um resultado sem erros comprova apenas as etapas que aparecem como executadas
  e aprovadas na interface.

