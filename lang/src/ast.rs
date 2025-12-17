#[derive(Debug, Clone)]
pub enum Declaration {
    EntityDeclaration {
        name: String,
        body: Vec<Declaration>,
    },
    ConstructorDeclaration {
        body: Vec<Statement>,
    },
    TickDeclaration {
        body: Vec<Statement>,
    },
    DrawDeclaration {
        body: Vec<Statement>,
    },
    InstanceVarDeclaration {
        names: Vec<String>,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<String>,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone)]
pub enum Statement {
    Expression(Expression),
    IfConditional {
        condition: Expression,
        true_body: Vec<Statement>,
        false_body: Option<Vec<Statement>>,
    },
    EachLoop {
        variable: String,
        source: Expression,
        body: Vec<Statement>,
    },
    Assignment {
        target: Expression,
        value: Expression,
    },
    Return(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    ThisLiteral,
    NullLiteral,
    NumberLiteral(f64),
    BooleanLiteral(bool),
    ArrayLiteral(Vec<Expression>),
    Identifier(String),
    InstanceVarIdentifier(String), // @var

    SpriteLiteral(Sprite),

    FunctionCall {
        target: Box<Expression>,
        name: String,
        arguments: Vec<Expression>,
    },
    BinaryOperation {
        left: Box<Expression>,
        right: Box<Expression>,
        operator: BinaryOperator,
    },

    AddEntity {
        // TODO: constructor parameters probably necessary later
        name: String,
    },

    Echo(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,

    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEquals,
    GreaterThanOrEquals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sprite {
    pub width: usize,
    pub height: usize,
    
    // Laid out like:
    //
    //   0 1 2
    //   3 4 5
    //   6 7 8
    //
    pub pixels: Vec<Pixel>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pixel {
    Clear,
    Set,
}
