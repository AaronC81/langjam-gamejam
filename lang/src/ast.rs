#[derive(Debug, Clone)]
pub enum Declaration {
    GameInitDeclaration {
        body: Vec<Statement>,
    },
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
    InstanceVarDeclaration {
        name: String,
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
    Identifier(String),
    InstanceVarIdentifier(String), // @var

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
}
