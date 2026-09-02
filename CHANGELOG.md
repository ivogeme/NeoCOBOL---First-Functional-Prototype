# Changelog

Todas as mudanças importantes da NeoCOBOL serão documentadas neste arquivo.

## [0.5.0] - 2026-09-02

### Adicionado

- Sistema básico de tipos com `decimal`, `string` e `boolean`.
- Operadores matemáticos.
- Operador módulo `%`.
- Operadores de comparação.
- Operadores lógicos `and`, `or` e `not`.
- Atribuições compostas:
  - `+=`
  - `-=`
  - `*=`
  - `/=`
  - `%=`
- Estruturas condicionais `if`, `else` e `end`.
- Estrutura de repetição `while`.
- Funções com parâmetros.
- `return`.
- Chamadas de funções.
- Strings com concatenação.
- Interpolação de strings usando `{variavel}`.
- Comentários usando `#`.
- Análise semântica.
- AST.
- Sistema de erros com códigos NEO.
- Runtime próprio.
- Operações decimais usando `rust_decimal`.

### Melhorias

- Separação mais clara entre lexer, parser, análise semântica e runtime.
- Mensagens de erro mais específicas.
- Verificação de tipos antes da execução.
- Melhor organização interna da linguagem.
- Suporte a programas maiores e mais estruturados.

### Limitações conhecidas

- Parâmetros de funções ainda não possuem tipos declarados.
- Tipos de retorno das funções ainda não são declarados.
- A análise semântica dos corpos das funções ainda precisa ser aprimorada.
- Ainda não existe compilação para código nativo.
- Ainda não existe biblioteca padrão.
- Ainda não existem arrays ou estruturas de dados complexas.
- Ainda não existe acesso a arquivos ou hardware.

## [0.4.0]

Versão anterior da linguagem.

## [0.3.0]

Versão anterior da linguagem.