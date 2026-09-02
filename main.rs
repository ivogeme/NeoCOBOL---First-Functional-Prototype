use rust_decimal::Decimal;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::str::FromStr;

// ================================================================
// NeoCOBOL 0.5.0
//
// Linguagem simples, legível e rigorosa.
//
// Pipeline:
//
// .neo
//   ↓
// Lexer
//   ↓
// Tokens
//   ↓
// Parser
//   ↓
// AST
//   ↓
// Análise semântica
//   ↓
// Runtime
//
// A arquitetura continua preparada para o futuro compilador.
// ================================================================


// ================================================================
// 1. TIPOS
// ================================================================

#[derive(Debug, Clone, PartialEq)]
enum Tipo {
    Decimal,
    String,
    Boolean,
    Vazio,
}

impl Tipo {
    fn nome(&self) -> &'static str {
        match self {
            Tipo::Decimal => "decimal",
            Tipo::String => "string",
            Tipo::Boolean => "boolean",
            Tipo::Vazio => "vazio",
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
    Vazio,
}

impl NeoValor {
    fn tipo(&self) -> Tipo {
        match self {
            NeoValor::Decimal(_) => Tipo::Decimal,
            NeoValor::String(_) => Tipo::String,
            NeoValor::Boolean(_) => Tipo::Boolean,
            NeoValor::Vazio => Tipo::Vazio,
        }
    }

    fn texto(&self) -> String {
        match self {
            NeoValor::Decimal(v) => v.to_string(),
            NeoValor::String(v) => v.clone(),
            NeoValor::Boolean(v) => v.to_string(),
            NeoValor::Vazio => String::new(),
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

    Semantico {
        linha: usize,
        mensagem: String,
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
            } => {
                write!(
                    f,
                    "NEO1001 - Erro léxico na linha {}: {}",
                    linha,
                    mensagem
                )
            }

            NeoErro::Sintaxe {
                linha,
                mensagem,
            } => {
                write!(
                    f,
                    "NEO1002 - Erro de sintaxe na linha {}: {}",
                    linha,
                    mensagem
                )
            }

            NeoErro::Tipo {
                linha,
                esperado,
                recebido,
            } => {
                write!(
                    f,
                    "NEO2001 - Erro de tipo na linha {}: esperado {}, recebido {}",
                    linha,
                    esperado,
                    recebido
                )
            }

            NeoErro::Semantico {
                linha,
                mensagem,
            } => {
                write!(
                    f,
                    "NEO2002 - Erro semântico na linha {}: {}",
                    linha,
                    mensagem
                )
            }

            NeoErro::Runtime {
                linha,
                mensagem,
            } => {
                if *linha == 0 {
                    write!(
                        f,
                        "NEO3001 - Erro de execução: {}",
                        mensagem
                    )
                } else {
                    write!(
                        f,
                        "NEO3001 - Erro de execução na linha {}: {}",
                        linha,
                        mensagem
                    )
                }
            }
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

    Func,
    Return,

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
    Modulo,

    Igual,
    IgualIgual,
    Diferente,

    Maior,
    Menor,
    MaiorIgual,
    MenorIgual,

    MaisIgual,
    MenosIgual,
    MultiplicaIgual,
    DivideIgual,
    ModuloIgual,

    AbrePar,
    FechaPar,
    Virgula,

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
        let mut chars = linha.chars().peekable();

        while let Some(&c) = chars.peek() {
            if c.is_whitespace() {
                chars.next();
                continue;
            }

            // ----------------------------------------------------
            // Comentário
            // ----------------------------------------------------

            if c == '#' {
                break;
            }

            // ----------------------------------------------------
            // String
            // ----------------------------------------------------

            if c == '"' {
                chars.next();

                let mut texto = String::new();
                let mut fechou = false;

                while let Some(&ch) = chars.peek() {
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

            // ----------------------------------------------------
            // Número
            // ----------------------------------------------------

            if c.is_ascii_digit() {
                let mut numero = String::new();
                let mut pontos = 0;

                while let Some(&ch) = chars.peek() {
                    if ch.is_ascii_digit() {
                        numero.push(ch);
                        chars.next();
                    } else if ch == '.' {
                        pontos += 1;

                        if pontos > 1 {
                            return Err(
                                NeoErro::Lexico {
                                    linha: numero_linha,
                                    mensagem:
                                        "Número possui mais de um ponto decimal."
                                            .into(),
                                }
                            );
                        }

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

            // ----------------------------------------------------
            // Identificadores
            // ----------------------------------------------------

            if c.is_alphabetic() || c == '_' {
                let mut palavra = String::new();

                while let Some(&ch) = chars.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        palavra.push(ch);
                        chars.next();
                    } else {
                        break;
                    }
                }

                let kind = match palavra.as_str() {
                    "decimal" => TokenKind::Decimal,
                    "string" => TokenKind::String,

                    "boolean"
                    | "true"
                    | "false" => TokenKind::Boolean,

                    "display" => TokenKind::Display,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "end" => TokenKind::End,
                    "while" => TokenKind::While,

                    "func" => TokenKind::Func,
                    "return" => TokenKind::Return,

                    "and" => TokenKind::And,
                    "or" => TokenKind::Or,
                    "not" => TokenKind::Not,

                    _ => TokenKind::Identificador,
                };

                tokens.push(Token {
                    kind,
                    texto: palavra,
                    linha: numero_linha,
                });

                continue;
            }

            // ----------------------------------------------------
            // Operadores
            // ----------------------------------------------------

            match c {
                '+' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::MaisIgual,
                            texto: "+=".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Mais,
                            texto: "+".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '-' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::MenosIgual,
                            texto: "-=".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Menos,
                            texto: "-".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '*' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::MultiplicaIgual,
                            texto: "*=".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Multiplica,
                            texto: "*".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '/' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::DivideIgual,
                            texto: "/=".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Divide,
                            texto: "/".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '%' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::ModuloIgual,
                            texto: "%=".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Modulo,
                            texto: "%".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '=' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::IgualIgual,
                            texto: "==".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Igual,
                            texto: "=".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '!' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::Diferente,
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

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::MaiorIgual,
                            texto: ">=".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Maior,
                            texto: ">".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '<' => {
                    chars.next();

                    if chars.peek() == Some(&'=') {
                        chars.next();

                        tokens.push(Token {
                            kind: TokenKind::MenorIgual,
                            texto: "<=".into(),
                            linha: numero_linha,
                        });
                    } else {
                        tokens.push(Token {
                            kind: TokenKind::Menor,
                            texto: "<".into(),
                            linha: numero_linha,
                        });
                    }
                }

                '(' => {
                    chars.next();

                    tokens.push(Token {
                        kind: TokenKind::AbrePar,
                        texto: "(".into(),
                        linha: numero_linha,
                    });
                }

                ')' => {
                    chars.next();

                    tokens.push(Token {
                        kind: TokenKind::FechaPar,
                        texto: ")".into(),
                        linha: numero_linha,
                    });
                }

                ',' => {
                    chars.next();

                    tokens.push(Token {
                        kind: TokenKind::Virgula,
                        texto: ",".into(),
                        linha: numero_linha,
                    });
                }

                _ => {
                    return Err(
                        NeoErro::Lexico {
                            linha: numero_linha,
                            mensagem: format!(
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

    Modulo(
        Box<Expressao>,
        Box<Expressao>,
    ),

    Negativo(
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

    Not(
        Box<Expressao>,
    ),

    Chamada {
        nome: String,
        argumentos: Vec<Expressao>,
    },
}


#[derive(Debug, Clone)]
enum OperadorAtribuicao {
    Igual,
    Mais,
    Menos,
    Multiplica,
    Divide,
    Modulo,
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
        operador: OperadorAtribuicao,
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

    Funcao {
        nome: String,
        parametros: Vec<String>,
        bloco: Vec<Instrucao>,
        linha: usize,
    },

    Return {
        valor: Expressao,
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
        if self.atual < self.tokens.len() - 1 {
            self.atual += 1;
        }
    }

    fn consumir(
        &mut self,
        esperado: TokenKind,
    ) -> Result<Token, NeoErro> {
        let token = self.atual().clone();

        if token.kind == esperado {
            self.avancar();
            Ok(token)
        } else {
            Err(
                NeoErro::Sintaxe {
                    linha: token.linha,
                    mensagem: format!(
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
        let mut resultado = Vec::new();

        while self.atual().kind != TokenKind::Fim {
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
            | TokenKind::Boolean => {
                self.declaracao()
            }

            TokenKind::Display =>
                self.display(),

            TokenKind::If =>
                self.if_instrucao(),

            TokenKind::While =>
                self.while_instrucao(),

            TokenKind::Func =>
                self.funcao(),

            TokenKind::Return =>
                self.retorno(),

            TokenKind::Identificador =>
                self.atribuicao(),

            _ => {
                let token =
                    self.atual().clone();

                Err(
                    NeoErro::Sintaxe {
                        linha: token.linha,
                        mensagem: format!(
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

                _ => unreachable!(),
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

        let operador =
            match self.atual().kind {
                TokenKind::Igual =>
                    OperadorAtribuicao::Igual,

                TokenKind::MaisIgual =>
                    OperadorAtribuicao::Mais,

                TokenKind::MenosIgual =>
                    OperadorAtribuicao::Menos,

                TokenKind::MultiplicaIgual =>
                    OperadorAtribuicao::Multiplica,

                TokenKind::DivideIgual =>
                    OperadorAtribuicao::Divide,

                TokenKind::ModuloIgual =>
                    OperadorAtribuicao::Modulo,

                _ => {
                    return Err(
                        NeoErro::Sintaxe {
                            linha,
                            mensagem:
                                "Esperado '=' ou operador de atribuição."
                                    .into(),
                        }
                    );
                }
            };

        self.avancar();

        let valor =
            self.expressao()?;

        Ok(
            Instrucao::Atribuir {
                nome: nome.texto,
                operador,
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

    fn funcao(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {
        let linha =
            self.atual().linha;

        self.consumir(
            TokenKind::Func
        )?;

        let nome =
            self.consumir(
                TokenKind::Identificador
            )?;

        self.consumir(
            TokenKind::AbrePar
        )?;

        let mut parametros =
            Vec::new();

        if self.atual().kind
            != TokenKind::FechaPar
        {
            loop {
                let parametro =
                    self.consumir(
                        TokenKind::Identificador
                    )?;

                parametros.push(
                    parametro.texto
                );

                if self.atual().kind
                    == TokenKind::Virgula
                {
                    self.avancar();
                } else {
                    break;
                }
            }
        }

        self.consumir(
            TokenKind::FechaPar
        )?;

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
            Instrucao::Funcao {
                nome: nome.texto,
                parametros,
                bloco,
                linha,
            }
        )
    }

    fn retorno(
        &mut self,
    ) -> Result<Instrucao, NeoErro> {
        let linha =
            self.atual().linha;

        self.consumir(
            TokenKind::Return
        )?;

        let valor =
            self.expressao()?;

        Ok(
            Instrucao::Return {
                valor,
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

                TokenKind::Modulo => {
                    self.avancar();

                    let direita =
                        self.unario()?;

                    esquerda =
                        Expressao::Modulo(
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

        if self.atual().kind
            == TokenKind::Menos
        {
            self.avancar();

            return Ok(
                Expressao::Negativo(
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

                if self.atual().kind
                    == TokenKind::AbrePar
                {
                    self.avancar();

                    let mut argumentos =
                        Vec::new();

                    if self.atual().kind
                        != TokenKind::FechaPar
                    {
                        loop {
                            argumentos.push(
                                self.expressao()?
                            );

                            if self.atual().kind
                                == TokenKind::Virgula
                            {
                                self.avancar();
                            } else {
                                break;
                            }
                        }
                    }

                    self.consumir(
                        TokenKind::FechaPar
                    )?;

                    return Ok(
                        Expressao::Chamada {
                            nome: token.texto,
                            argumentos,
                        }
                    );
                }

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

            _ => Err(
                NeoErro::Sintaxe {
                    linha: token.linha,
                    mensagem: format!(
                        "Expressão inesperada '{}'.",
                        token.texto
                    ),
                }
            ),
        }
    }
}


// ================================================================
// 8. FUNÇÕES
// ================================================================

#[derive(Clone)]
struct Funcao {
    parametros: Vec<String>,
    bloco: Vec<Instrucao>,
}


// ================================================================
// 9. SÍMBOLOS SEMÂNTICOS
// ================================================================

#[derive(Clone)]
struct Simbolo {
    tipo: Tipo,
}


// ================================================================
// 10. ANALISADOR SEMÂNTICO
// ================================================================

struct AnalisadorSemantico {
    escopos: Vec<HashMap<String, Simbolo>>,
    funcoes: HashMap<String, usize>,
}

impl AnalisadorSemantico {
    fn novo() -> Self {
        Self {
            escopos: vec![
                HashMap::new()
            ],
            funcoes: HashMap::new(),
        }
    }

    fn analisar(
        &mut self,
        programa: &[Instrucao],
    ) -> Result<(), NeoErro> {
        // Primeiro registramos funções.
        for instrucao in programa {
            if let Instrucao::Funcao {
                nome,
                parametros,
                linha,
                ..
            } = instrucao
            {
                if self.funcoes.contains_key(nome) {
                    return Err(
                        NeoErro::Semantico {
                            linha: *linha,
                            mensagem: format!(
                                "A função '{}' já foi declarada.",
                                nome
                            ),
                        }
                    );
                }

                self.funcoes.insert(
                    nome.clone(),
                    parametros.len()
                );
            }
        }

        for instrucao in programa {
            if matches!(
                instrucao,
                Instrucao::Funcao { .. }
            ) {
                continue;
            }

            self.instruacao(instrucao)?;
        }

        Ok(())
    }

    fn instruacao(
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
                if self.escopos
                    .last()
                    .unwrap()
                    .contains_key(nome)
                {
                    return Err(
                        NeoErro::Semantico {
                            linha: *linha,
                            mensagem: format!(
                                "A variável '{}' já foi declarada neste escopo.",
                                nome
                            ),
                        }
                    );
                }

                let tipo_valor =
                    self.expressao(valor)?;

                if tipo_valor != *tipo {
                    return Err(
                        NeoErro::Tipo {
                            linha: *linha,
                            esperado:
                                tipo.nome().into(),
                            recebido:
                                tipo_valor
                                    .nome()
                                    .into(),
                        }
                    );
                }

                self.escopos
                    .last_mut()
                    .unwrap()
                    .insert(
                        nome.clone(),
                        Simbolo {
                            tipo: tipo.clone(),
                        },
                    );
            }

            Instrucao::Atribuir {
                nome,
                operador,
                valor,
                linha,
            } => {
                let tipo_variavel =
                    self.buscar_variavel(
                        nome
                    )
                    .ok_or_else(|| {
                        NeoErro::Semantico {
                            linha: *linha,
                            mensagem: format!(
                                "Variável '{}' não existe.",
                                nome
                            ),
                        }
                    })?;

                let tipo_valor =
                    self.expressao(valor)?;

                match operador {
                    OperadorAtribuicao::Igual => {
                        if tipo_variavel
                            != tipo_valor
                        {
                            return Err(
                                NeoErro::Tipo {
                                    linha: *linha,
                                    esperado:
                                        tipo_variavel
                                            .nome()
                                            .into(),
                                    recebido:
                                        tipo_valor
                                            .nome()
                                            .into(),
                                }
                            );
                        }
                    }

                    OperadorAtribuicao::Mais
                    | OperadorAtribuicao::Menos
                    | OperadorAtribuicao::Multiplica
                    | OperadorAtribuicao::Divide
                    | OperadorAtribuicao::Modulo => {
                        if tipo_variavel
                            != Tipo::Decimal
                            || tipo_valor
                                != Tipo::Decimal
                        {
                            return Err(
                                NeoErro::Tipo {
                                    linha: *linha,
                                    esperado:
                                        "decimal".into(),
                                    recebido:
                                        format!(
                                            "{} e {}",
                                            tipo_variavel
                                                .nome(),
                                            tipo_valor
                                                .nome()
                                        ),
                                }
                            );
                        }
                    }
                }
            }

            Instrucao::Display {
                valor,
                linha,
            } => {
                self.expressao(valor)
                    .map_err(|erro| {
                        match erro {
                            NeoErro::Tipo {
                                linha: _,
                                esperado,
                                recebido,
                            } => {
                                NeoErro::Tipo {
                                    linha: *linha,
                                    esperado,
                                    recebido,
                                }
                            }

                            outro => outro,
                        }
                    })?;
            }

            Instrucao::If {
                condicao,
                verdadeiro,
                falso,
                linha,
            } => {
                let tipo =
                    self.expressao(condicao)?;

                if tipo != Tipo::Boolean {
                    return Err(
                        NeoErro::Tipo {
                            linha: *linha,
                            esperado:
                                "boolean".into(),
                            recebido:
                                tipo.nome().into(),
                        }
                    );
                }

                self.entrar_escopo();

                for instrucao
                    in verdadeiro
                {
                    self.instruacao(
                        instrucao
                    )?;
                }

                self.sair_escopo();

                self.entrar_escopo();

                for instrucao
                    in falso
                {
                    self.instruacao(
                        instrucao
                    )?;
                }

                self.sair_escopo();
            }

            Instrucao::While {
                condicao,
                bloco,
                linha,
            } => {
                let tipo =
                    self.expressao(condicao)?;

                if tipo != Tipo::Boolean {
                    return Err(
                        NeoErro::Tipo {
                            linha: *linha,
                            esperado:
                                "boolean".into(),
                            recebido:
                                tipo.nome().into(),
                        }
                    );
                }

                self.entrar_escopo();

                for instrucao
                    in bloco
                {
                    self.instruacao(
                        instrucao
                    )?;
                }

                self.sair_escopo();
            }

            Instrucao::Funcao {
                ..
            } => {}

            Instrucao::Return {
                valor,
                ..
            } => {
                self.expressao(valor)?;
            }
        }

        Ok(())
    }

    fn expressao(
        &self,
        expressao: &Expressao,
    ) -> Result<Tipo, NeoErro> {
        match expressao {
            Expressao::Decimal(_) =>
                Ok(Tipo::Decimal),

            Expressao::String(_) =>
                Ok(Tipo::String),

            Expressao::Boolean(_) =>
                Ok(Tipo::Boolean),

            Expressao::Variavel(nome) => {
                self.buscar_variavel(nome)
                    .ok_or_else(|| {
                        NeoErro::Semantico {
                            linha: 0,
                            mensagem: format!(
                                "Variável '{}' não existe.",
                                nome
                            ),
                        }
                    })
            }

            Expressao::Negativo(valor) => {
                let tipo =
                    self.expressao(valor)?;

                if tipo != Tipo::Decimal {
                    return Err(
                        NeoErro::Tipo {
                            linha: 0,
                            esperado:
                                "decimal".into(),
                            recebido:
                                tipo.nome().into(),
                        }
                    );
                }

                Ok(Tipo::Decimal)
            }

            Expressao::Soma(a, b) => {
                let a =
                    self.expressao(a)?;

                let b =
                    self.expressao(b)?;

                match (&a, &b) {
                    (
                        Tipo::Decimal,
                        Tipo::Decimal,
                    ) |
                    (
                        Tipo::String,
                        Tipo::String,
                    ) => Ok(a),

                    _ => Err(
                        NeoErro::Tipo {
                            linha: 0,
                            esperado:
                                "decimal + decimal ou string + string"
                                    .into(),
                            recebido:
                                format!(
                                    "{} + {}",
                                    a.nome(),
                                    b.nome()
                                ),
                        }
                    ),
                }
            }

            Expressao::Subtracao(a, b)
            | Expressao::Multiplicacao(a, b)
            | Expressao::Divisao(a, b)
            | Expressao::Modulo(a, b) => {
                let a =
                    self.expressao(a)?;

                let b =
                    self.expressao(b)?;

                if a != Tipo::Decimal
                    || b != Tipo::Decimal
                {
                    return Err(
                        NeoErro::Tipo {
                            linha: 0,
                            esperado:
                                "decimal".into(),
                            recebido:
                                format!(
                                    "{} e {}",
                                    a.nome(),
                                    b.nome()
                                ),
                        }
                    );
                }

                Ok(Tipo::Decimal)
            }

            Expressao::Maior(a, b)
            | Expressao::Menor(a, b)
            | Expressao::MaiorIgual(a, b)
            | Expressao::MenorIgual(a, b) => {
                let a =
                    self.expressao(a)?;

                let b =
                    self.expressao(b)?;

                if a != Tipo::Decimal
                    || b != Tipo::Decimal
                {
                    return Err(
                        NeoErro::Tipo {
                            linha: 0,
                            esperado:
                                "dois valores decimal"
                                    .into(),
                            recebido:
                                format!(
                                    "{} e {}",
                                    a.nome(),
                                    b.nome()
                                ),
                        }
                    );
                }

                Ok(Tipo::Boolean)
            }

            Expressao::Igual(a, b)
            | Expressao::Diferente(a, b) => {
                let a =
                    self.expressao(a)?;

                let b =
                    self.expressao(b)?;

                if a != b {
                    return Err(
                        NeoErro::Tipo {
                            linha: 0,
                            esperado:
                                a.nome().into(),
                            recebido:
                                b.nome().into(),
                        }
                    );
                }

                Ok(Tipo::Boolean)
            }

            Expressao::And(a, b)
            | Expressao::Or(a, b) => {
                let a =
                    self.expressao(a)?;

                let b =
                    self.expressao(b)?;

                if a != Tipo::Boolean
                    || b != Tipo::Boolean
                {
                    return Err(
                        NeoErro::Tipo {
                            linha: 0,
                            esperado:
                                "boolean".into(),
                            recebido:
                                format!(
                                    "{} e {}",
                                    a.nome(),
                                    b.nome()
                                ),
                        }
                    );
                }

                Ok(Tipo::Boolean)
            }

            Expressao::Not(a) => {
                let tipo =
                    self.expressao(a)?;

                if tipo != Tipo::Boolean {
                    return Err(
                        NeoErro::Tipo {
                            linha: 0,
                            esperado:
                                "boolean".into(),
                            recebido:
                                tipo.nome().into(),
                        }
                    );
                }

                Ok(Tipo::Boolean)
            }

            Expressao::Chamada {
                nome,
                argumentos,
            } => {
                let esperado =
                    self.funcoes.get(nome)
                        .ok_or_else(|| {
                            NeoErro::Semantico {
                                linha: 0,
                                mensagem: format!(
                                    "Função '{}' não existe.",
                                    nome
                                ),
                            }
                        })?;

                if argumentos.len()
                    != *esperado
                {
                    return Err(
                        NeoErro::Semantico {
                            linha: 0,
                            mensagem: format!(
                                "Função '{}' espera {} argumento(s), mas recebeu {}.",
                                nome,
                                esperado,
                                argumentos.len()
                            ),
                        }
                    );
                }

                for argumento
                    in argumentos
                {
                    self.expressao(
                        argumento
                    )?;
                }

                // Nesta versão funções
                // ainda não possuem
                // assinatura de retorno
                // na AST.
                Ok(Tipo::Vazio)
            }
        }
    }

    fn buscar_variavel(
        &self,
        nome: &str,
    ) -> Option<Tipo> {
        for escopo
            in self.escopos.iter().rev()
        {
            if let Some(simbolo) =
                escopo.get(nome)
            {
                return Some(
                    simbolo.tipo.clone()
                );
            }
        }

        None
    }

    fn entrar_escopo(&mut self) {
        self.escopos.push(
            HashMap::new()
        );
    }

    fn sair_escopo(&mut self) {
        self.escopos.pop();
    }
}


// ================================================================
// 11. CONTEXTO DE EXECUÇÃO
// ================================================================

struct Runtime {
    variaveis: Vec<HashMap<String, NeoValor>>,
    funcoes: HashMap<String, Funcao>,
}

impl Runtime {
    fn novo() -> Self {
        Self {
            variaveis: vec![
                HashMap::new()
            ],
            funcoes: HashMap::new(),
        }
    }

    fn executar(
        &mut self,
        programa: &[Instrucao],
    ) -> Result<(), NeoErro> {
        // Registrar funções.
        for instrucao in programa {
            if let Instrucao::Funcao {
                nome,
                parametros,
                bloco,
                ..
            } = instrucao
            {
                self.funcoes.insert(
                    nome.clone(),
                    Funcao {
                        parametros:
                            parametros.clone(),
                        bloco:
                            bloco.clone(),
                    },
                );
            }
        }

        for instrucao in programa {
            if matches!(
                instrucao,
                Instrucao::Funcao { .. }
            ) {
                continue;
            }

            self.executar_instrucao(
                instrucao
            )?;
        }

        Ok(())
    }

    fn executar_bloco(
        &mut self,
        bloco: &[Instrucao],
    ) -> Result<Option<NeoValor>, NeoErro> {
        self.variaveis.push(
            HashMap::new()
        );

        for instrucao in bloco {
            if let Some(valor) =
                self.executar_instrucao(
                    instrucao
                )?
            {
                self.variaveis.pop();

                return Ok(Some(valor));
            }
        }

        self.variaveis.pop();

        Ok(None)
    }

    fn executar_instrucao(
        &mut self,
        instrucao: &Instrucao,
    ) -> Result<Option<NeoValor>, NeoErro> {
        match instrucao {
            Instrucao::Declarar {
                nome,
                valor,
                ..
            } => {
                let resultado =
                    self.avaliar(
                        valor,
                        0
                    )?;

                self.variaveis
                    .last_mut()
                    .unwrap()
                    .insert(
                        nome.clone(),
                        resultado,
                    );
            }

            Instrucao::Atribuir {
                nome,
                operador,
                valor,
                linha,
            } => {
                let novo =
                    self.avaliar(
                        valor,
                        *linha
                    )?;

                let escopo_indice =
                    self.encontrar_escopo(nome)
                        .ok_or_else(|| {
                            NeoErro::Runtime {
                                linha: *linha,
                                mensagem:
                                    format!(
                                        "Variável '{}' não existe.",
                                        nome
                                    ),
                            }
                        })?;

                let antigo =
                    self.variaveis[
                        escopo_indice
                    ]
                    .get(nome)
                    .cloned()
                    .unwrap();

                let resultado =
                    self.aplicar_atribuicao(
                        antigo,
                        novo,
                        operador,
                        *linha
                    )?;

                self.variaveis[
                    escopo_indice
                ]
                .insert(
                    nome.clone(),
                    resultado,
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

                println!(
                    "{}",
                    self.interpolar(
                        &resultado.texto()
                    )
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
                        if let Some(valor) =
                            self.executar_bloco(
                                verdadeiro
                            )?
                        {
                            return Ok(Some(
                                valor
                            ));
                        }
                    }

                    NeoValor::Boolean(false) => {
                        if let Some(valor) =
                            self.executar_bloco(
                                falso
                            )?
                        {
                            return Ok(Some(
                                valor
                            ));
                        }
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
                            if let Some(valor) =
                                self.executar_bloco(
                                    bloco
                                )?
                            {
                                return Ok(Some(
                                    valor
                                ));
                            }
                        }

                        NeoValor::Boolean(false) => {
                            break;
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
            }

            Instrucao::Funcao {
                ..
            } => {}

            Instrucao::Return {
                valor,
                linha,
            } => {
                let resultado =
                    self.avaliar(
                        valor,
                        *linha
                    )?;

                return Ok(Some(
                    resultado
                ));
            }
        }

        Ok(None)
    }


    // ============================================================
    // ATRIBUIÇÕES COMPOSTAS
    // ============================================================

    fn aplicar_atribuicao(
        &self,
        antigo: NeoValor,
        novo: NeoValor,
        operador: &OperadorAtribuicao,
        linha: usize,
    ) -> Result<NeoValor, NeoErro> {
        match operador {
            OperadorAtribuicao::Igual => {
                if antigo.tipo()
                    != novo.tipo()
                {
                    return Err(
                        NeoErro::Tipo {
                            linha,
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

                Ok(novo)
            }

            OperadorAtribuicao::Mais => {
                match (antigo, novo) {
                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b),
                    ) =>
                        Ok(
                            NeoValor::Decimal(
                                a + b
                            )
                        ),

                    (
                        NeoValor::String(a),
                        NeoValor::String(b),
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
                                    "tipos iguais compatíveis com +="
                                        .into(),
                                recebido:
                                    format!(
                                        "{} e {}",
                                        a.tipo()
                                            .nome(),
                                        b.tipo()
                                            .nome()
                                    ),
                            }
                        ),
                }
            }

            OperadorAtribuicao::Menos =>
                self.operacao_decimal_valores(
                    antigo,
                    novo,
                    linha,
                    |a, b| a - b,
                ),

            OperadorAtribuicao::Multiplica =>
                self.operacao_decimal_valores(
                    antigo,
                    novo,
                    linha,
                    |a, b| a * b,
                ),

            OperadorAtribuicao::Divide => {
                match (antigo, novo) {
                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b),
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
                                    "decimal /= decimal"
                                        .into(),
                                recebido:
                                    format!(
                                        "{} e {}",
                                        a.tipo()
                                            .nome(),
                                        b.tipo()
                                            .nome()
                                    ),
                            }
                        ),
                }
            }

            OperadorAtribuicao::Modulo => {
                match (antigo, novo) {
                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b),
                    ) => {
                        if b.is_zero() {
                            return Err(
                                NeoErro::Runtime {
                                    linha,
                                    mensagem:
                                        "Módulo por zero."
                                            .into(),
                                }
                            );
                        }

                        Ok(
                            NeoValor::Decimal(
                                a % b
                            )
                        )
                    }

                    (a, b) =>
                        Err(
                            NeoErro::Tipo {
                                linha,
                                esperado:
                                    "decimal %= decimal"
                                        .into(),
                                recebido:
                                    format!(
                                        "{} e {}",
                                        a.tipo()
                                            .nome(),
                                        b.tipo()
                                            .nome()
                                    ),
                            }
                        ),
                }
            }
        }
    }

    fn operacao_decimal_valores<F>(
        &self,
        a: NeoValor,
        b: NeoValor,
        linha: usize,
        operacao: F,
    ) -> Result<NeoValor, NeoErro>
    where
        F: Fn(
            Decimal,
            Decimal
        ) -> Decimal,
    {
        match (a, b) {
            (
                NeoValor::Decimal(a),
                NeoValor::Decimal(b),
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
                            "decimal e decimal"
                                .into(),
                        recebido:
                            format!(
                                "{} e {}",
                                a.tipo()
                                    .nome(),
                                b.tipo()
                                    .nome()
                            ),
                    }
                ),
        }
    }


    // ============================================================
    // AVALIAÇÃO
    // ============================================================

    fn avaliar(
        &mut self,
        expressao: &Expressao,
        linha: usize,
    ) -> Result<NeoValor, NeoErro> {
        match expressao {
            Expressao::Decimal(v) =>
                Ok(
                    NeoValor::Decimal(
                        *v
                    )
                ),

            Expressao::String(v) =>
                Ok(
                    NeoValor::String(
                        v.clone()
                    )
                ),

            Expressao::Boolean(v) =>
                Ok(
                    NeoValor::Boolean(
                        *v
                    )
                ),

            Expressao::Variavel(nome) => {
                for escopo
                    in self.variaveis.iter().rev()
                {
                    if let Some(valor) =
                        escopo.get(nome)
                    {
                        return Ok(
                            valor.clone()
                        );
                    }
                }

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
            }

            Expressao::Negativo(valor) => {
                match self.avaliar(
                    valor,
                    linha
                )? {
                    NeoValor::Decimal(v) =>
                        Ok(
                            NeoValor::Decimal(
                                -v
                            )
                        ),

                    outro =>
                        Err(
                            NeoErro::Tipo {
                                linha,
                                esperado:
                                    "decimal".into(),
                                recebido:
                                    outro
                                        .tipo()
                                        .nome()
                                        .into(),
                            }
                        ),
                }
            }

            Expressao::Soma(a, b) => {
                let a =
                    self.avaliar(a, linha)?;

                let b =
                    self.avaliar(b, linha)?;

                match (a, b) {
                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b),
                    ) =>
                        Ok(
                            NeoValor::Decimal(
                                a + b
                            )
                        ),

                    (
                        NeoValor::String(a),
                        NeoValor::String(b),
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
                        ),
                }
            }

            Expressao::Subtracao(a, b) =>
                self.operacao_decimal(
                    a,
                    b,
                    linha,
                    |x, y| x - y,
                    "decimal - decimal",
                ),

            Expressao::Multiplicacao(a, b) =>
                self.operacao_decimal(
                    a,
                    b,
                    linha,
                    |x, y| x * y,
                    "decimal * decimal",
                ),

            Expressao::Divisao(a, b) => {
                let a =
                    self.avaliar(a, linha)?;

                let b =
                    self.avaliar(b, linha)?;

                match (a, b) {
                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b),
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
                        ),
                }
            }

            Expressao::Modulo(a, b) => {
                let a =
                    self.avaliar(a, linha)?;

                let b =
                    self.avaliar(b, linha)?;

                match (a, b) {
                    (
                        NeoValor::Decimal(a),
                        NeoValor::Decimal(b),
                    ) => {
                        if b.is_zero() {
                            return Err(
                                NeoErro::Runtime {
                                    linha,
                                    mensagem:
                                        "Módulo por zero."
                                            .into(),
                                }
                            );
                        }

                        Ok(
                            NeoValor::Decimal(
                                a % b
                            )
                        )
                    }

                    (a, b) =>
                        Err(
                            NeoErro::Tipo {
                                linha,
                                esperado:
                                    "decimal % decimal"
                                        .into(),
                                recebido:
                                    format!(
                                        "{} % {}",
                                        a.tipo()
                                            .nome(),
                                        b.tipo()
                                            .nome()
                                    ),
                            }
                        ),
                }
            }

            Expressao::Maior(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x > y,
                ),

            Expressao::Menor(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x < y,
                ),

            Expressao::MaiorIgual(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x >= y,
                ),

            Expressao::MenorIgual(a, b) =>
                self.comparar(
                    a,
                    b,
                    linha,
                    |x, y| x <= y,
                ),

            Expressao::Igual(a, b) => {
                let a =
                    self.avaliar(a, linha)?;

                let b =
                    self.avaliar(b, linha)?;

                Ok(
                    NeoValor::Boolean(
                        a == b
                    )
                )
            }

            Expressao::Diferente(a, b) => {
                let a =
                    self.avaliar(a, linha)?;

                let b =
                    self.avaliar(b, linha)?;

                Ok(
                    NeoValor::Boolean(
                        a != b
                    )
                )
            }

            // ----------------------------------------------------
            // AND com curto-circuito
            // ----------------------------------------------------

            Expressao::And(a, b) => {
                let esquerda =
                    self.obter_booleano(
                        a,
                        linha
                    )?;

                if !esquerda {
                    return Ok(
                        NeoValor::Boolean(
                            false
                        )
                    );
                }

                let direita =
                    self.obter_booleano(
                        b,
                        linha
                    )?;

                Ok(
                    NeoValor::Boolean(
                        esquerda && direita
                    )
                )
            }

            // ----------------------------------------------------
            // OR com curto-circuito
            // ----------------------------------------------------

            Expressao::Or(a, b) => {
                let esquerda =
                    self.obter_booleano(
                        a,
                        linha
                    )?;

                if esquerda {
                    return Ok(
                        NeoValor::Boolean(
                            true
                        )
                    );
                }

                let direita =
                    self.obter_booleano(
                        b,
                        linha
                    )?;

                Ok(
                    NeoValor::Boolean(
                        esquerda || direita
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

            Expressao::Chamada {
                nome,
                argumentos,
            } =>
                self.chamar_funcao(
                    nome,
                    argumentos,
                    linha
                ),
        }
    }

    fn operacao_decimal<F>(
        &mut self,
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
            self.avaliar(a, linha)?;

        let b =
            self.avaliar(b, linha)?;

        match (a, b) {
            (
                NeoValor::Decimal(a),
                NeoValor::Decimal(b),
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
                                a.tipo()
                                    .nome(),
                                b.tipo()
                                    .nome()
                            ),
                    }
                ),
        }
    }

    fn comparar<F>(
        &mut self,
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
            self.avaliar(a, linha)?;

        let b =
            self.avaliar(b, linha)?;

        match (a, b) {
            (
                NeoValor::Decimal(a),
                NeoValor::Decimal(b),
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
                                a.tipo()
                                    .nome(),
                                b.tipo()
                                    .nome()
                            ),
                    }
                ),
        }
    }

    fn obter_booleano(
        &mut self,
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
                ),
        }
    }


    // ============================================================
    // CHAMADA DE FUNÇÕES
    // ============================================================

    fn chamar_funcao(
        &mut self,
        nome: &str,
        argumentos: &[Expressao],
        linha: usize,
    ) -> Result<NeoValor, NeoErro> {
        let funcao =
            match self.funcoes.get(nome) {
                Some(f) =>
                    f.clone(),

                None =>
                    return Err(
                        NeoErro::Runtime {
                            linha,
                            mensagem:
                                format!(
                                    "Função '{}' não existe.",
                                    nome
                                ),
                        }
                    ),
            };

        if argumentos.len()
            != funcao.parametros.len()
        {
            return Err(
                NeoErro::Runtime {
                    linha,
                    mensagem:
                        format!(
                            "Função '{}' espera {} argumento(s), mas recebeu {}.",
                            nome,
                            funcao.parametros.len(),
                            argumentos.len()
                        ),
                }
            );
        }

        let mut novo_escopo =
            HashMap::new();

        for (parametro, argumento)
            in funcao
                .parametros
                .iter()
                .zip(argumentos.iter())
        {
            let valor =
                self.avaliar(
                    argumento,
                    linha
                )?;

            novo_escopo.insert(
                parametro.clone(),
                valor
            );
        }

        self.variaveis.push(
            novo_escopo
        );

        let resultado =
            self.executar_bloco_funcao(
                &funcao.bloco
            )?;

        self.variaveis.pop();

        Ok(
            resultado.unwrap_or(
                NeoValor::Vazio
            )
        )
    }

    fn executar_bloco_funcao(
        &mut self,
        bloco: &[Instrucao],
    ) -> Result<Option<NeoValor>, NeoErro> {
        for instrucao in bloco {
            if let Some(valor) =
                self.executar_instrucao(
                    instrucao
                )?
            {
                return Ok(Some(
                    valor
                ));
            }
        }

        Ok(None)
    }


    // ============================================================
    // ESCOPOS
    // ============================================================

    fn encontrar_escopo(
        &self,
        nome: &str,
    ) -> Option<usize> {
        for indice
            in (0..self.variaveis.len())
                .rev()
        {
            if self.variaveis[indice]
                .contains_key(nome)
            {
                return Some(indice);
            }
        }

        None
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

        for escopo
            in &self.variaveis
        {
            for (nome, valor)
                in escopo
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
        }

        resultado
    }
}


// ================================================================
// 12. EXECUÇÃO DO ARQUIVO
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
        "NeoCOBOL 0.5.0"
    );

    println!(
        "Compilando: {}",
        caminho
    );

    // ------------------------------------------------------------
    // Lexer
    // ------------------------------------------------------------

    let tokens =
        lex(&codigo)?;

    println!(
        "Lexer: {} tokens.",
        tokens.len()
    );

    // ------------------------------------------------------------
    // Parser
    // ------------------------------------------------------------

    let mut parser =
        Parser::novo(tokens);

    let programa =
        parser.analisar()?;

    println!(
        "Parser: AST construída."
    );

    // ------------------------------------------------------------
    // Análise semântica
    // ------------------------------------------------------------

    let mut semantico =
        AnalisadorSemantico::novo();

    semantico.analisar(
        &programa
    )?;

    println!(
        "Semântica: programa válido."
    );

    // ------------------------------------------------------------
    // Runtime
    // ------------------------------------------------------------

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
// 13. MAIN
// ================================================================

fn main() {
    let argumentos:
        Vec<String> =
        env::args().collect();

    if argumentos.len() < 2 {
        println!(
            "NeoCOBOL 0.5.0"
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