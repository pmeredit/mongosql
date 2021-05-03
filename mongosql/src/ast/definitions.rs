use linked_hash_map::LinkedHashMap;

#[derive(PartialEq, Debug, Clone)]
pub enum Query {
    Select(SelectQuery),
    Set(SetQuery),
}

#[derive(PartialEq, Debug, Clone)]
pub struct SelectQuery {
    pub select_clause: SelectClause,
    pub from_clause: Option<Datasource>,
    pub where_clause: Option<Expression>,
    pub group_by_clause: Option<GroupByClause>,
    pub having_clause: Option<Expression>,
    pub order_by_clause: Option<OrderByClause>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SetQuery {
    pub left: Box<Query>,
    pub op: SetOperator,
    pub right: Box<Query>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SetOperator {
    Union,
    UnionAll,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SelectClause {
    pub set_quantifier: SetQuantifier,
    pub body: SelectBody,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SetQuantifier {
    All,
    Distinct,
}

#[derive(PartialEq, Debug, Clone)]
pub enum SelectBody {
    Standard(Vec<SelectExpression>),
    Values(Vec<SelectValuesExpression>),
}

#[derive(PartialEq, Debug, Clone)]
pub enum SelectValuesExpression {
    Expression(Expression),
    Substar(SubstarExpr),
}

#[derive(PartialEq, Debug, Clone)]
pub enum SelectExpression {
    Star,
    Substar(SubstarExpr),
    Aliased(AliasedExpr),
}

#[derive(PartialEq, Debug, Clone)]
pub struct SubstarExpr {
    pub datasource: String,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Datasource {
    Array(ArraySource),
    Collection(CollectionSource),
    Derived(DerivedSource),
    Join(JoinSource),
}

#[derive(PartialEq, Debug, Clone)]
pub struct ArraySource {
    pub array: Vec<Expression>,
    pub alias: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct CollectionSource {
    pub database: Option<String>,
    pub collection: String,
    pub alias: Option<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct DerivedSource {
    pub query: Box<Query>,
    pub alias: String,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AliasedExpr {
    pub expr: Expression,
    pub alias: Option<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct JoinSource {
    pub join_type: JoinType,
    pub left: Box<Datasource>,
    pub right: Box<Datasource>,
    pub condition: Option<Expression>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum JoinType {
    Left,
    Right,
    Cross,
    Inner,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Expression {
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Between(BetweenExpr),
    Case(CaseExpr),
    Function(FunctionExpr),
    Cast(CastExpr),
    Array(Vec<Expression>),
    Subquery(Box<Query>),
    Exists(Box<Query>),
    SubqueryComparison(SubqueryComparisonExpr),
    Document(LinkedHashMap<String, Expression>),
    Access(AccessExpr),
    Subpath(SubpathExpr),
    Identifier(String),
    Is(IsExpr),
    Like(LikeExpr),
    Literal(Literal),
    Tuple(Vec<Expression>),
    TypeAssertion(TypeAssertionExpr),
}

#[derive(PartialEq, Debug, Clone)]
pub struct CastExpr {
    pub expr: Box<Expression>,
    pub to: Type,
    pub on_null: Option<Box<Expression>>,
    pub on_error: Option<Box<Expression>>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expression>,
    pub op: BinaryOp,
    pub right: Box<Expression>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expression>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct BetweenExpr {
    pub expr: Box<Expression>,
    pub min: Box<Expression>,
    pub max: Box<Expression>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct CaseExpr {
    pub expr: Option<Box<Expression>>,
    pub when_branch: Vec<WhenBranch>,
    pub else_branch: Option<Box<Expression>>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct WhenBranch {
    pub when: Box<Expression>,
    pub then: Box<Expression>,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SubqueryQuantifier {
    All,
    Any,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SubqueryComparisonExpr {
    pub expr: Box<Expression>,
    pub op: BinaryOp,
    pub quantifier: SubqueryQuantifier,
    pub subquery: Box<Query>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionExpr {
    pub function: FunctionName,
    pub args: Vec<FunctionArg>,
    pub set_quantifier: Option<SetQuantifier>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct FunctionName(pub String);

#[derive(PartialEq, Debug, Clone)]
pub enum FunctionArg {
    Star,
    Expr(Expression),
    Extract(ExtractSpec),
    Trim(TrimSpec),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ExtractSpec {
    TimezoneHour,
    TimezoneMinute,
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TrimSpec {
    Leading,
    Trailing,
    Both,
}

#[derive(PartialEq, Debug, Clone)]
pub struct AccessExpr {
    pub expr: Box<Expression>,
    pub subfield: Box<Expression>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SubpathExpr {
    pub expr: Box<Expression>,
    pub subpath: String,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum TypeOrMissing {
    Type(Type),
    Missing,
}

#[derive(PartialEq, Debug, Clone)]
pub struct IsExpr {
    pub expr: Box<Expression>,
    pub target_type: TypeOrMissing,
}

#[derive(PartialEq, Debug, Clone)]
pub struct LikeExpr {
    pub expr: Box<Expression>,
    pub pattern: Box<Expression>,
    pub escape: Option<String>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct TypeAssertionExpr {
    pub expr: Box<Expression>,
    pub target_type: Type,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum UnaryOp {
    Pos,
    Neg,
    Not,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum BinaryOp {
    Add,
    And,
    Concat,
    Div,
    Eq,
    Gt,
    Gte,
    In,
    Lt,
    Lte,
    Mul,
    Neq,
    NotIn,
    Or,
    Sub,
}

#[derive(PartialEq, Debug, Clone)]
pub struct GroupByClause {
    pub keys: Vec<AliasedExpr>,
    pub aggregations: Vec<AliasedExpr>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct OrderByClause {
    pub sort_specs: Vec<SortSpec>,
}

#[derive(PartialEq, Debug, Clone)]
pub struct SortSpec {
    pub key: SortKey,
    pub direction: SortDirection,
}

#[derive(PartialEq, Debug, Clone)]
pub enum SortKey {
    Simple(Expression),
    Positional(u32),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    Null,
    Boolean(bool),
    String(String),
    Integer(i32),
    Long(i64),
    Double(f64),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Type {
    Array,
    BinData,
    Boolean,
    Datetime,
    DbPointer,
    Decimal128,
    Document,
    Double,
    Int32,
    Int64,
    Javascript,
    JavascriptWithScope,
    MaxKey,
    MinKey,
    Null,
    ObjectId,
    RegularExpression,
    String,
    Symbol,
    Timestamp,
    Undefined,
}
