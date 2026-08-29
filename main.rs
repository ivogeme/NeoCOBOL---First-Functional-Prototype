use rust_decimal::Decimal;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::str::FromStr;

// ================================================================
// NeoCOBOL 0.3.0
// Um compilador/interpretador inicial da linguagem NeoCOBOL.
//
// Tudo fica propositalmente em um único arquivo nesta fase.
// ================================================================


// ================================================================
// 1. TIPOS DA LINGUAGEM
// ================================================================

#[derive(Debug, Clone, PartialEq)]
enum Tipo {
    Decimal,
    String,
    Boolean,
}

impl Tipo {
    fn nome(&self) -> &'static str {
        match self {
            Tipo::Decimal => "decimal",
            Tipo::String => "string",
            Tipo::Boolean => "boolean",
        }
    }
}


// ================================================================
// 2. VALORES
// ================================================================

#[derive(Debug, Clone, PartialEq)]
enum NeoValor {
    Decimal(Decimal),
    String(String),
    Boolean(bool),
}

impl NeoValor {
    fn tipo(&self) -> Tipo {
        match self {
            NeoValor::Decimal(_) => Tipo::Decimal,
            NeoValor::String(_) => Tipo::String,
            NeoValor::Boolean(_) => Tipo::Boolean,
        }
    }

    fn texto(&self) -> String {
        match self {
            NeoValor::Decimal(valor) => valor.to_string(),
            NeoValor::String(valor) => valor.clone(),
            NeoValor::Boolean(valor) => valor.to_string(),
        }
    }
}


// ================================================================
// 3. ERROS
// ================================================================

#[derive(Debug)]
enum NeoErro {
    Lexico {
        linha: usize,
        mensagem: String,
    },

    Sintaxe {
        linha: usize,
        mensagem: String,
    },

    Tipo {
        linha: usize,
        esperado: String,
        recebido: String,
    },

    Runtime {
        linha: usize,
        mensagem: String,
    },
}

impl std::fmt::Display for NeoErro {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {

        match self {

            NeoErro::Lexico {
                linha,
                mensagem,
            } => write!(
                f,
                "NEO1001 - Erro léxico na linha {}: {}",
                linha,
                mensagem
            ),

            NeoErro::Sintaxe {
                linha,
                mensagem,
            } => write!(
                f,
                "NEO1002 - Erro de sintaxe na linha {}: {}",
                linha,
                mensagem
            ),

            NeoErro::Tipo {
                linha,
                esperado,
                recebido,
            } => write!(
                f,
                "NEO2001 - Tipo incompatível na linha {}: esperado {}, recebido {}",
                linha,
                esperado,
                recebido
            ),

            NeoErro::Runtime {
                linha,
                mensagem,
            } => write!(
                f,
                "NEO3001 - Erro de execução na linha {}: {}",
                linha,
                mensagem
            ),
        }
    }
}


// ================================================================
// 4. TOKENS
// ================================================================

#[derive(Debug, Clone, PartialEq)]
enum TokenKind {
    Decimal,
    String,
    Boolean,

    Display,
    If,
    Else,
    End,
    While,

    And,
    Or,
    Not,

    Identificador,
    Numero,
    Texto,

    Mais,
    Menos,
    Multiplica,
    Divide,

    Igual,
    IgualIgual,
    Diferente,

    Maior,
    Menor,
    MaiorIgual,
    MenorIgual,

    AbrePar,
    FechaPar,

    Fim,
}

#[derive(Debug, Clone)]
struct Token {
    kind: TokenKind,
    texto: String,
    linha: usize,
}


// ================================================================
// 5. LEXER
// ================================================================

fn lex(codigo: &str) -> Result<Vec<Token>, NeoErro> {

    let mut tokens = Vec::new();

    for (indice, linha) in codigo.lines().enumerate() {

        let numero_linha = indice + 1;

        let mut chars =
            linha.chars().peekable();

        while let Some(&c) = chars.peek() {

            // Espaços
            if c.is_whitespace() {
                chars.next();
                continue;
            }

            // Comentários
            if c == '#' {
                break;
            }

            // Strings
            if c == '"' {

                chars.next();

                let mut texto =
                    String::new();

                let mut fechou =
                    false;

                while let Some(&ch) =
                    chars.peek()
                {
                    chars.next();

                    if ch == '"' {
                        fechou = true;
                        break;
                    }

                    texto.push(ch);
                }

                if !fechou {
                    return Err(
                        NeoErro::Lexico {
                            linha: numero_linha,
                            mensagem:
                                "String não terminada."
                                    .into(),
                        }
                    );
                }

                tokens.push(Token {
                    kind: TokenKind::Texto,
                    texto,
                    linha: numero_linha,
                });

                continue;
            }

            // Números
            if c.is_ascii_digit() {

                let mut numero =
                    String::new();

                while let Some(&ch) =
                    chars.peek()
                {
                    if ch.is_ascii_digit()
                        || ch == '.'
                    {
                        numero.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                tokens.push(Token {
                    kind: TokenKind::Numero,
                    texto: numero,
                    linha: numero_linha,
                });

                continue;
            }

            // Identificadores / palavras-chave
            if c.is_alphabetic()
                || c == '_'
            {
                let mut palavra =
                    String::new();

                while let Some(&ch) =
                    chars.peek()
                {
                    if ch.is_alphanumeric()
                        || ch == '_'
                    {
                        palavra.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let kind =
                    match palavra.as_str() {

                        "decimal" =>
                            TokenKind::Decimal,

                        "string" =>
                            TokenKind::String,

                        "boolean" =>
                            TokenKind::Boolean,

                        "display" =>
                            TokenKind::Display,

                        "if" =>
                            TokenKind::If,

                        "else" =>
                            TokenKind::Else,

                        "end" =>
                            TokenKind::End,

                        "while" =>
                            TokenKind::While,

                        "and" =>
                            TokenKind::And,

                        "or" =>
                            TokenKind::Or,

                        "not" =>
                            TokenKind::Not,

                        "true" |
                        "false" =>
                            TokenKind::Boolean,

                        _ =>
                            TokenKind::Identificador,
                    };

                tokens.push(Token {
                    kind,
                    texto: palavra,
                    linha: numero_linha,
                });

                continue;
            }

            // Operadores
            match c {

                '+' => {
                    chars.next();

                    tokens.push(Token {
                        kind: TokenKind::Mais,
                        texto: "+".into(),
                        linha: numero_linha,
                    });
                }

                '-' => {
                    chars.next();

                    tokens.push(Token {
                        kind: TokenKind::Menos,
                        texto: "-".into(),
                        linha: numero_linha,
                    });
                }

                '*' => {
                    chars.next();

                    tokens.push(Token {
                        kind:
                            TokenKind::Multiplica,
                        texto: "*".into(),
                        linha: numero_linha,
                    });
                }

                '/' => {
                    chars.next();

                    tokens.push(Token {
                        kind:
                            TokenKind::Divide,
                        texto: "/".into(),
                        linha: numero_linha,
                    });
                }

                '(' => {
                    chars.next();

                    tokens.push(Token {
                        kind:
                            TokenKind::AbrePar,
                        texto: "(".into(),
                        linha: numero_linha,
                    });
                }

                ')' => {
                    chars.next();

                    tokens.push(Token {
                        kind:
                            TokenKind::FechaPar,
                        texto: ")".into(),
                        linha: numero_linha,
                    });
                }

                '=' => {

                    chars.next();

                    if chars.peek() ==
                        Some(&'=')
                    {
                        chars.next();

                        tokens.push(Token {
                            kind:
                                TokenKind::IgualIgual,
                            texto: "==".into(),
                            linha: numero_linha,
                        });
                    } else {

                        tokens.push(Token {
                            kind:
                                TokenKind::Igual,
                            texto: "=".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '!' => {

                    chars.next();

                    if chars.peek() ==
                        Some(&'=')
                    {
                        chars.next();

                        tokens.push(Token {
                            kind:
                                TokenKind::Diferente,
                            texto: "!=".into(),
                            linha: numero_linha,
                        });
                    } else {

                        return Err(
                            NeoErro::Lexico {
                                linha: numero_linha,
                                mensagem:
                                    "Use != para diferença."
                                        .into(),
                            }
                        );
                    }
                }

                '>' => {

                    chars.next();

                    if chars.peek() ==
                        Some(&'=')
                    {
                        chars.next();

                        tokens.push(Token {
                            kind:
                                TokenKind::MaiorIgual,
                            texto: ">=".into(),
                            linha: numero_linha,
                        });
                    } else {

                        tokens.push(Token {
                            kind:
                                TokenKind::Maior,
                            texto: ">".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '<' => {

                    chars.next();

                    if chars.peek() ==
                        Some(&'=')
                    {
                        chars.next();

                        tokens.push(Token {
                            kind:
                                TokenKind::MenorIgual,
                            texto: "<=".into(),
                            linha: numero_linha,
                        });
                    } else {

                        tokens.push(Token {
                            kind:
                                TokenKind::Menor,
                            texto: "<".into(),
                            linha: numero_linha,
                        });
                    }
                }

                _ => {

                    return Err(
                        NeoErro::Lexico {
                            linha: numero_linha,
                            mensagem:
                                format!(
                                    "Caractere desconhecido '{}'.",
                                    c
                                ),
                        }
                    );
                }
            }
        }
    }

    tokens.push(Token {
        kind: TokenKind::Fim,
        texto: String::new(),
        linha: codigo.lines().count() + 1,
    });

    Ok(tokens)
}


// ================================================================
// 6. AST
// ================================================================

#[derive(Debug, Clone)]
enum Expressao {

    Decimal(Decimal),

    String(String),

    Boolean(bool),

    Variavel(String),

    Soma(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Subtracao(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Multiplicacao(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Divisao(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Maior(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Menor(
        Box<Expressao>,
        Box<Expressao>,
    ),

    MaiorIgual(
        Box<Expressao>,
        Box<Expressao>,
    ),

    MenorIgual(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Igual(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Diferente(
        Box<Expressao>,
        Box<Expressao>,
    ),

    And(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Or(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Not(Box<Expressao>),
}

#[derive(Debug, Clone)]
enum Instrucao {

    Declarar {
        tipo: Tipo,
        nome: String,
        valor: Expressao,
        linha: usize,
    },

    Atribuir {
        nome: String,
        valor: Expressao,
        linha: usize,
    },

    Display {
        valor: Expressao,
        linha: usize,
    },

    If {
        condicao: Expressao,
        verdadeiro: Vec<Instrucao>,
        falso: Vec<Instrucao>,
        linha: usize,
    },

    While {
        condicao: Expressao,
        bloco: Vec<Instrucao>,
        linha: usize,
    },
}


// ================================================================
// 7. PARSER
// ================================================================

struct Parser {
    tokens: Vec<Token>,
    atual: usize,
}

impl Parser {

    fn novo(tokens: Vec<Token>) -> Self {

        Self {
            tokens,
            atual: 0,
        }
    }

    fn atual(&self) -> &Token {
        &self.tokens[self.atual]
    }

    fn avancar(&mut self) {

        if self.atual <
            self.tokens.len() - 1
        {
            self.atual += 1;
        }
    }

    fn consumir(
        &mut self,
        esperado: TokenKind,
    ) -> Result<Token, NeoErro> {

        let token =
            self.atual().clone();

        if token.kind == esperado {

            self.avancar();

            Ok(token)

        } else {

            Err(
                NeoErro::Sintaxe {
                    linha: token.linha,
                    mensagem:
                        format!(
                            "Esperado {:?}, encontrado '{}'.",
                            esperado,
                            token.texto
                        ),
                }
            )
        }
    }

    fn analisar(
        &mut self,
    ) -> Result<Vec<Instrucao>, NeoErro> {

        let mut resultado =
            Vec::new();

        while self.atual().kind
            != TokenKind::Fim
        {
            resultado.push(
                self.instrucao()?
            );
        }

        Ok(resultado)
    }

    fn instrucao(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {

        match self.atual().kind.clone() {

            TokenKind::Decimal
            | TokenKind::String
            | TokenKind::Boolean =>
                self.declaracao(),

            TokenKind::Display =>
                self.display(),

            TokenKind::If =>
                self.if_instrucao(),

            TokenKind::While =>
                self.while_instrucao(),

            TokenKind::Identificador =>
                self.atribuicao(),

            _ => {

                let token =
                    self.atual().clone();

                Err(
                    NeoErro::Sintaxe {
                        linha: token.linha,
                        mensagem:
                            format!(
                                "Instrução inesperada '{}'.",
                                token.texto
                            ),
                    }
                )
            }
        }
    }

    fn declaracao(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {

        let tipo =
            match self.atual().kind {

                TokenKind::Decimal =>
                    Tipo::Decimal,

                TokenKind::String =>
                    Tipo::String,

                TokenKind::Boolean =>
                    Tipo::Boolean,

                _ =>
                    unreachable!(),
            };

        let linha =
            self.atual().linha;

        self.avancar();

        let nome =
            self.consumir(
                TokenKind::Identificador
            )?;

        self.consumir(
            TokenKind::Igual
        )?;

        let valor =
            self.expressao()?;

        Ok(
            Instrucao::Declarar {
                tipo,
                nome: nome.texto,
                valor,
                linha,
            }
        )
    }

    fn atribuicao(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {

        let nome =
            self.consumir(
                TokenKind::Identificador
            )?;

        let linha =
            nome.linha;

        self.consumir(
            TokenKind::Igual
        )?;

        let valor =
            self.expressao()?;

        Ok(
            Instrucao::Atribuir {
                nome: nome.texto,
                valor,
                linha,
            }
        )
    }

    fn display(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {

        let linha =
            self.atual().linha;

        self.consumir(
            TokenKind::Display
        )?;

        let valor =
            self.expressao()?;

        Ok(
            Instrucao::Display {
                valor,
                linha,
            }
        )
    }

    fn if_instrucao(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {

        let linha =
            self.atual().linha;

        self.consumir(
            TokenKind::If
        )?;

        let condicao =
            self.expressao()?;

        let mut verdadeiro =
            Vec::new();

        while self.atual().kind
            != TokenKind::Else
            && self.atual().kind
                != TokenKind::End
            && self.atual().kind
                != TokenKind::Fim
        {
            verdadeiro.push(
                self.instrucao()?
            );
        }

        let mut falso =
            Vec::new();

        if self.atual().kind
            == TokenKind::Else
        {
            self.avancar();

            while self.atual().kind
                != TokenKind::End
                && self.atual().kind
                    != TokenKind::Fim
            {
                falso.push(
                    self.instrucao()?
                );
            }
        }

        self.consumir(
            TokenKind::End
        )?;

        Ok(
            Instrucao::If {
                condicao,
                verdadeiro,
                falso,
                linha,
            }
        )
    }

    fn while_instrucao(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {

        let linha =
            self.atual().linha;

        self.consumir(
            TokenKind::While
        )?;

        let condicao =
            self.expressao()?;

        let mut bloco =
            Vec::new();

        while self.atual().kind
            != TokenKind::End
            && self.atual().kind
                != TokenKind::Fim
        {
            bloco.push(
                self.instrucao()?
            );
        }

        self.consumir(
            TokenKind::End
        )?;

        Ok(
            Instrucao::While {
                condicao,
                bloco,
                linha,
            }
        )
    }


    // ============================================================
    // EXPRESSÕES
    // ============================================================

    fn expressao(
        &mut self,
    ) -> Result<Expressao, NeoErro> {
        self.or()
    }

    fn or(
        &mut self,
    ) -> Result<Expressao, NeoErro> {

        let mut esquerda =
            self.and()?;

        while self.atual().kind
            == TokenKind::Or
        {
            self.avancar();

            let direita =
                self.and()?;

            esquerda =
                Expressao::Or(
                    Box::new(esquerda),
                    Box::new(direita),
                );
        }

        Ok(esquerda)
    }

    fn and(
        &mut self,
    ) -> Result<Expressao, NeoErro> {

        let mut esquerda =
            self.comparacao()?;

        while self.atual().kind
            == TokenKind::And
        {
            self.avancar();

            let direita =
                self.comparacao()?;

            esquerda =
                Expressao::And(
                    Box::new(esquerda),
                    Box::new(direita),
                );
        }

        Ok(esquerda)
    }

    fn comparacao(
        &mut self,
    ) -> Result<Expressao, NeoErro> {

        let mut esquerda =
            self.termo()?;

        loop {

            let operacao =
                match self.atual().kind {

                    TokenKind::Maior =>
                        Some(0),

                    TokenKind::Menor =>
                        Some(1),

                    TokenKind::MaiorIgual =>
                        Some(2),

                    TokenKind::MenorIgual =>
                        Some(3),

                    TokenKind::IgualIgual =>
                        Some(4),

                    TokenKind::Diferente =>
                        Some(5),

                    _ => None,
                };

            let Some(op) =
                operacao
            else {
                break;
            };

            self.avancar();

            let direita =
                self.termo()?;

            esquerda =
                match op {

                    0 =>
                        Expressao::Maior(
                            Box::new(esquerda),
                            Box::new(direita),
                        ),

                    1 =>
                        Expressao::Menor(
                            Box::new(esquerda),
                            Box::new(direita),
                        ),

                    2 =>
                        Expressao::MaiorIgual(
                            Box::new(esquerda),
                            Box::new(direita),
                        ),

                    3 =>
                        Expressao::MenorIgual(
                            Box::new(esquerda),
                            Box::new(direita),
                        ),

                    4 =>
                        Expressao::Igual(
                            Box::new(esquerda),
                            Box::new(direita),
                        ),

                    _ =>
                        Expressao::Diferente(
                            Box::new(esquerda),
                            Box::new(direita),
                        ),
                };
        }

        Ok(esquerda)
    }

    fn termo(
        &mut self,
    ) -> Result<Expressao, NeoErro> {

        let mut esquerda =
            self.fator()?;

        loop {

            match self.atual().kind {

                TokenKind::Mais => {

                    self.avancar();

                    let direita =
                        self.fator()?;

                    esquerda =
                        Expressao::Soma(
                            Box::new(esquerda),
                            Box::new(direita),
                        );
                }

                TokenKind::Menos => {

                    self.avancar();

                    let direita =
                        self.fator()?;

                    esquerda =
                        Expressao::Subtracao(
                            Box::new(esquerda),
                            Box::new(direita),
                        );
                }

                _ => break,
            }
        }

        Ok(esquerda)
    }

    fn fator(
        &mut self,
    ) -> Result<Expressao, NeoErro> {

        let mut esquerda =
            self.unario()?;

        loop {

            match self.atual().kind {

                TokenKind::Multiplica => {

                    self.avancar();

                    let direita =
                        self.unario()?;

                    esquerda =
                        Expressao::Multiplicacao(
                            Box::new(esquerda),
                            Box::new(direita),
                        );
                }

                TokenKind::Divide => {

                    self.avancar();

                    let direita =
                        self.unario()?;

                    esquerda =
                        Expressao::Divisao(
                            Box::new(esquerda),
                            Box::new(direita),
                        );
                }

                _ => break,
            }
        }

        Ok(esquerda)
    }

    fn unario(
        &mut self,
    ) -> Result<Expressao, NeoErro> {

        if self.atual().kind
            == TokenKind::Not
        {
            self.avancar();

            return Ok(
                Expressao::Not(
                    Box::new(
                        self.unario()?
                    )
                )
            );
        }

        self.primario()
    }

    fn primario(
        &mut self,
    ) -> Result<Expressao, NeoErro> {

        let token =
            self.atual().clone();

        match token.kind {

            TokenKind::Numero => {

                self.avancar();

                let valor =
                    Decimal::from_str(
                        &token.texto
                    )
                    .map_err(|_| {
                        NeoErro::Sintaxe {
                            linha: token.linha,
                            mensagem:
                                "Número decimal inválido."
                                    .into(),
                        }
                    })?;

                Ok(
                    Expressao::Decimal(
                        valor
                    )
                )
            }

            TokenKind::Texto => {

                self.avancar();

                Ok(
                    Expressao::String(
                        token.texto
                    )
                )
            }

            TokenKind::Boolean => {

                self.avancar();

                Ok(
                    Expressao::Boolean(
                        token.texto == "true"
                    )
                )
            }

            TokenKind::Identificador => {

                self.avancar();

                Ok(
                    Expressao::Variavel(
                        token.texto
                    )
                )
            }

            TokenKind::AbrePar => {

                self.avancar();

                let resultado =
                    self.expressao()?;

                self.consumir(
                    TokenKind::FechaPar
                )?;

                Ok(resultado)
            }

            _ => {

                Err(
                    NeoErro::Sintaxe {
                        linha: token.linha,
                        mensagem:
                            format!(
                                "Expressão inesperada '{}'.",
                                token.texto
                            ),
                    }
                )
            }
        }
    }
}


// ================================================================
// 8. RUNTIME
// ================================================================

struct Runtime {
    variaveis:
        HashMap<String, NeoValor>,
}

impl Runtime {

    fn novo() -> Self {

        Self {
            variaveis:
                HashMap::new(),
        }
    }

    fn executar(
        &mut self,
        programa: &[Instrucao],
    ) -> Result<(), NeoErro> {

        for instrucao in programa {
            self.executar_instrucao(
                instrucao
            )?;
        }

        Ok(())
    }

    fn executar_instrucao(
        &mut self,
        instrucao: &Instrucao,
    ) -> Result<(), NeoErro> {

        match instrucao {

            Instrucao::Declarar {
                tipo,
                nome,
                valor,
                linha,
            } => {

                if self.variaveis
                    .contains_key(nome)
                {
                    return Err(
                        NeoErro::Runtime {
                            linha: *linha,
                            mensagem:
                                format!(
                                    "A variável '{}' já foi declarada.",
                                    nome
                                ),
                        }
                    );
                }

                let resultado =
                    self.avaliar(
                        valor,
                        *linha
                    )?;

                if resultado.tipo()
                    != *tipo
                {
                    return Err(
                        NeoErro::Tipo {
                            linha: *linha,
                            esperado:
                                tipo.nome()
                                    .into(),
                            recebido:
                                resultado
                                    .tipo()
                                    .nome()
                                    .into(),
                        }
                    );
                }

                self.variaveis.insert(
                    nome.clone(),
                    resultado
                );
            }

            Instrucao::Atribuir {
                nome,
                valor,
                linha,
            } => {

                let antigo =
                    match self.variaveis
                        .get(nome)
                    {
                        Some(v) =>
                            v.clone(),

                        None =>
                            return Err(
                                NeoErro::Runtime {
                                    linha: *linha,
                                    mensagem:
                                        format!(
                                            "Variável '{}' não existe.",
                                            nome
                                        ),
                                }
                            ),
                    };

                let novo =
                    self.avaliar(
                        valor,
                        *linha
                    )?;

                if antigo.tipo()
                    != novo.tipo()
                {
                    return Err(
                        NeoErro::Tipo {
                            linha: *linha,
                            esperado:
                                antigo
                                    .tipo()
                                    .nome()
                                    .into(),
                            recebido:
                                novo
                                    .tipo()
                                    .nome()
                                    .into(),
                        }
                    );
                }

                self.variaveis.insert(
                    nome.clone(),
                    novo
                );
            }

            Instrucao::Display {
                valor,
                linha,
            } => {

                let resultado =
                    self.avaliar(
                        valor,
                        *linha
                    )?;

                let texto =
                    self.interpolar(
                        &resultado.texto()
                    );

                println!(
                    "{}",
                    texto
                );
            }

            Instrucao::If {
                condicao,
                verdadeiro,
                falso,
                linha,
            } => {

                let resultado =
                    self.avaliar(
                        condicao,
                        *linha
                    )?;

                match resultado {

                    NeoValor::Boolean(true) => {
                        self.executar(
                            verdadeiro
                        )?;
                    }

                    NeoValor::Boolean(false) => {
                        self.executar(
                            falso
                        )?;
                    }

                    outro => {
                        return Err(
                            NeoErro::Tipo {
                                linha: *linha,
                                esperado:
                                    "boolean".into(),
                                recebido:
                                    outro
                                        .tipo()
                                        .nome()
                                        .into(),
                            }
                        );
                    }
                }
            }

            Instrucao::While {
                condicao,
                bloco,
                linha,
            } => {

                loop {

                    let resultado =
                        self.avaliar(
                            condicao,
                            *linha
                        )?;

                    match resultado {

                        NeoValor::Boolean(true) => {
                            self.executar(
                                bloco
                            )?;
                        }

                        NeoValor::Boolean(false) => {
                            break;
                        }

                        outro => {
                            return Err(
                                NeoErro::Tipo {
                                    linha: *linha,
                                    esperado:
                                        "boolean"
                                            .into(),
                                    recebido:
                                        outro
                                            .tipo()
                                            .nome()
                                            .into(),
                                }
                            );
                        }
                    }
                }
            }
        }

        Ok(())
    }


    // ============================================================
    // AVALIAÇÃO DE EXPRESSÕES
    // ============================================================

    fn avaliar(
        &self,
        expressao: &Expressao,
        linha: usize,
    ) -> Result<NeoValor, NeoErro> {

        match expressao {

            Expressao::Decimal(v) =>
                Ok(
                    NeoValor::Decimal(*v)
                ),

            Expressao::String(v) =>
                Ok(
                    NeoValor::String(
                        v.clone()
                    )
                ),

            Expressao::Boolean(v) =>
                Ok(
                    NeoValor::Boolean(*v)
                ),

            Expressao::Variavel(nome) =>
                match self.variaveis
                    .get(nome)
                {
                    Some(valor) =>
                        Ok(valor.clone()),

                    None =>
                        Err(
                            NeoErro::Runtime {
                                linha,
                                mensagem:
                                    format!(
                                        "Variável '{}' não existe.",
                                        nome
                                    ),
                            }
                        )
                },

            Expressao::Soma(a, b) => {

                let a =
                    self.avaliar(
                        a,
                        linha
                    )?;

                let b =
                    self.avaliar(
                        b,
                        linha
                    )?;

                match (a, b) {

                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b)
                    ) =>
                        Ok(
                            NeoValor::Decimal(
                                a + b
                            )
                        ),

                    (
                        NeoValor::String(a),
                        NeoValor::String(b)
                    ) =>
                        Ok(
                            NeoValor::String(
                                format!(
                                    "{}{}",
                                    a,
                                    b
                                )
                            )
                        ),

                    (a, b) =>
                        Err(
                            NeoErro::Tipo {
                                linha,
                                esperado:
                                    "decimal + decimal ou string + string"
                                        .into(),
                                recebido:
                                    format!(
                                        "{} + {}",
                                        a.tipo()
                                            .nome(),
                                        b.tipo()
                                            .nome()
                                    ),
                            }
                        )
                }
            }

            Expressao::Subtracao(a, b) =>
                self.operacao_decimal(
                    a,
                    b,
                    linha,
                    |x, y| x - y,
                    "decimal - decimal"
                ),

            Expressao::Multiplicacao(a, b) =>
                self.operacao_decimal(
                    a,
                    b,
                    linha,
                    |x, y| x * y,
                    "decimal * decimal"
                ),

            Expressao::Divisao(a, b) => {

                let a =
                    self.avaliar(
                        a,
                        linha
                    )?;

                let b =
                    self.avaliar(
                        b,
                        linha
                    )?;

                match (a, b) {

                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b)
                    ) => {

                        if b.is_zero() {
                            return Err(
                                NeoErro::Runtime {
                                    linha,
                                    mensagem:
                                        "Divisão por zero."
                                            .into(),
                                }
                            );
                        }

                        Ok(
                            NeoValor::Decimal(
                                a / b
                            )
                        )
                    }

                    (a, b) =>
                        Err(
                            NeoErro::Tipo {
                                linha,
                                esperado:
                                    "decimal / decimal"
                                        .into(),
                                recebido:
                                    format!(
                                        "{} / {}",
                                        a.tipo()
                                            .nome(),
                                        b.tipo()
                                            .nome()
                                    ),
                            }
                        )
                }
            }

            Expressao::Maior(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x > y
                ),

            Expressao::Menor(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x < y
                ),

            Expressao::MaiorIgual(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x >= y
                ),

            Expressao::MenorIgual(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x <= y
                ),

            Expressao::Igual(a, b) => {

                let a =
                    self.avaliar(
                        a,
                        linha
                    )?;

                let b =
                    self.avaliar(
                        b,
                        linha
                    )?;

                Ok(
                    NeoValor::Boolean(
                        a == b
                    )
                )
            }

            Expressao::Diferente(a, b) => {

                let a =
                    self.avaliar(
                        a,
                        linha
                    )?;

                let b =
                    self.avaliar(
                        b,
                        linha
                    )?;

                Ok(
                    NeoValor::Boolean(
                        a != b
                    )
                )
            }

            Expressao::And(a, b) => {

                let a =
                    self.obter_booleano(
                        a,
                        linha
                    )?;

                let b =
                    self.obter_booleano(
                        b,
                        linha
                    )?;

                Ok(
                    NeoValor::Boolean(
                        a && b
                    )
                )
            }

            Expressao::Or(a, b) => {

                let a =
                    self.obter_booleano(
                        a,
                        linha
                    )?;

                let b =
                    self.obter_booleano(
                        b,
                        linha
                    )?;

                Ok(
                    NeoValor::Boolean(
                        a || b
                    )
                )
            }

            Expressao::Not(a) => {

                let valor =
                    self.obter_booleano(
                        a,
                        linha
                    )?;

                Ok(
                    NeoValor::Boolean(
                        !valor
                    )
                )
            }
        }
    }

    fn operacao_decimal<F>(
        &self,
        a: &Expressao,
        b: &Expressao,
        linha: usize,
        operacao: F,
        nome: &str,
    ) -> Result<NeoValor, NeoErro>
    where
        F: Fn(
            Decimal,
            Decimal
        ) -> Decimal,
    {
        let a =
            self.avaliar(
                a,
                linha
            )?;

        let b =
            self.avaliar(
                b,
                linha
            )?;

        match (a, b) {

            (
                NeoValor::Decimal(a),
                NeoValor::Decimal(b)
            ) =>
                Ok(
                    NeoValor::Decimal(
                        operacao(a, b)
                    )
                ),

            (a, b) =>
                Err(
                    NeoErro::Tipo {
                        linha,
                        esperado:
                            nome.into(),
                        recebido:
                            format!(
                                "{} e {}",
                                a.tipo().nome(),
                                b.tipo().nome()
                            ),
                    }
                )
        }
    }

    fn comparar<F>(
        &self,
        a: &Expressao,
        b: &Expressao,
        linha: usize,
        operacao: F,
    ) -> Result<NeoValor, NeoErro>
    where
        F: Fn(
            Decimal,
            Decimal
        ) -> bool,
    {
        let a =
            self.avaliar(
                a,
                linha
            )?;

        let b =
            self.avaliar(
                b,
                linha
            )?;

        match (a, b) {

            (
                NeoValor::Decimal(a),
                NeoValor::Decimal(b)
            ) =>
                Ok(
                    NeoValor::Boolean(
                        operacao(a, b)
                    )
                ),

            (a, b) =>
                Err(
                    NeoErro::Tipo {
                        linha,
                        esperado:
                            "dois valores decimal"
                                .into(),
                        recebido:
                            format!(
                                "{} e {}",
                                a.tipo().nome(),
                                b.tipo().nome()
                            ),
                    }
                )
        }
    }

    fn obter_booleano(
        &self,
        expressao: &Expressao,
        linha: usize,
    ) -> Result<bool, NeoErro> {

        match self.avaliar(
            expressao,
            linha
        )? {

            NeoValor::Boolean(v) =>
                Ok(v),

            outro =>
                Err(
                    NeoErro::Tipo {
                        linha,
                        esperado:
                            "boolean".into(),
                        recebido:
                            outro
                                .tipo()
                                .nome()
                                .into(),
                    }
                )
        }
    }


    // ============================================================
    // INTERPOLAÇÃO
    // ============================================================

    fn interpolar(
        &self,
        texto: &str,
    ) -> String {

        let mut resultado =
            texto.to_string();

        for (nome, valor)
            in &self.variaveis
        {
            let marcador =
                format!(
                    "{{{}}}",
                    nome
                );

            resultado =
                resultado.replace(
                    &marcador,
                    &valor.texto()
                );
        }

        resultado
    }
}


// ================================================================
// 9. COMPILAR E EXECUTAR ARQUIVO
// ================================================================

fn executar_arquivo(
    caminho: &str,
) -> Result<(), NeoErro> {

    let codigo =
        fs::read_to_string(
            caminho
        )
        .map_err(|erro| {
            NeoErro::Runtime {
                linha: 0,
                mensagem:
                    format!(
                        "Não foi possível abrir '{}': {}",
                        caminho,
                        erro
                    ),
            }
        })?;

    println!(
        "NeoCOBOL 0.3.0"
    );

    println!(
        "Compilando: {}",
        caminho
    );

    // Lexer
    let tokens =
        lex(&codigo)?;

    // Parser
    let mut parser =
        Parser::novo(tokens);

    let programa =
        parser.analisar()?;

    println!(
        "Análise concluída."
    );

    // Runtime
    let mut runtime =
        Runtime::novo();

    runtime.executar(
        &programa
    )?;

    println!(
        "\nPrograma executado com sucesso."
    );

    Ok(())
}


// ================================================================
// 10. MAIN
// ================================================================

fn main() {

    let argumentos:
        Vec<String> =
        env::args().collect();

    if argumentos.len() < 2 {

        println!(
            "NeoCOBOL 0.3.0"
        );

        println!(
            "Uso:"
        );

        println!(
            "  cargo run -- programa.neo"
        );

        return;
    }

    if let Err(erro) =
        executar_arquivo(
            &argumentos[1]
        )
    {
        eprintln!(
            "\n{}",
            erro
        );

        std::process::exit(1);
    }
}