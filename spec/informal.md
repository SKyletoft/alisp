# ALISP
This is an informal specification of the alisp language semantics to
serve as a plan for an Agda formalisation. It does not cover the
syntax more than absolutely required. Assume normal S-expressions. The
syntax only provides us with lists, atoms/identifiers, quotes,
quasiquotes and unquotes. There are no lambda literals, macro literals
or similar.

It's a minimal initial version. Once it works, more number types,
strings, arrays and maps are to be added.

# The AST
There is some more typical sugar and restrictions on what's considered
an atom in the parser, but this is what we're working with here. The
parser will never generate lambda or macro nodes, but other
preprocessing steps may.

expr ::= atom | number | pair | quote | quasiquote | unquote | lambda | macro
atom ::= string
number ::= natural
pair ::= "(" expr expr ")"
quote ::= "'" expr
quasiquote ::= "`" expr
unquote ::= "," expr
lambda ::= atom* expr* expr
macro ::= atom* expr* expr

# Values
ALISP is homoiconic, meaning code is data and vice-versa. The AST
types are our value types. Lists are cons-cell lists of pairs ending
in the atom "nil".

# Values and references
ALISP has a form of mutable reference semantics and therefore we need
to differentiate between references and values.

A value is an AST node and a reference is a handle to a
value. Subexpressions are done through references and are not
considered a part of the object. A reference is always valid.  The
value that a reference refers to may be mutated or replaced and this
is then visible through all references to that value.

# Scopes
Only the current function's local variables are accessible and in
scope by name at any moment. There is no global scope for
names. Object references are always usable though.
The "lambda" and "macro" builtin functions resolve captures to object
references when creating lambda or macro function objects to get
around this for seemingly global names, as described below.

# Evaluation of expressions

## Quotes
Quotes evaluate to their wrapped expression and evaluation does not
recurse into it.

## Quasiquotes
Quasiquotes evaluate to their inner expression with unquotes replaced
by their inner expressions when directly evaluated and themselves when
indirectly evaluated.

* Unquotes (,) are invalid outside of quasiquotes. Inside of a
  quasiquote their wrapped statement gets evaluated and spliced into
  their surroundings when the quasiquote gets evaluated.

## Function calls
We have two types of functions, called lambda and macro. Functions
calls are of the shape of a cons-cell, nil-terminated proper linked
list. The first element, the function being called is always
evaluated. The evaluation of the rest depends on the type of function
being called.
### Lambda functions
If it's a lambda function object we then evaluate each item in order
until we reach the nil terminator and then create a new scope
containing only those items bound to the names taken from the lambda
object's parameter list and evaluate the statements that make up the
body of the function and returning the final expression. There is no
global, always accessible scope. "Global" objects are bound as
captures at lambda creation time as described below.
### Macro functions
If it's a macro function object we bind each argument as unevaluated
code/data to the names of the parameter list and evaluate the
statements that make up the body of the macro function WITHOUT
creating a new scope, we just continue with everything else still in
scope. The final expression is evaluated and returned as code. We then
remove our arguments from the scope and evaluate the code returned
from the macro as if it were the expression written in place of the
macro. Macros are unhygienic on purpose.

## Builtins
### Builtin functions
Just the basic mathsy ones, not included in the formalisation for now.
### Builtin macros
We have four builtin macro functions: "lambda", "macro", "set",
"declare" (and a set of builtin lambda functions for maths and such
that we don't need to model for now). Having the lambda and macro
functions distinguish us from a typical lisp as those treat lambda as
a keyword that required an eval step before any generic list can be
evaluated. We don't need the eval step because lambda is a builtin
function that transforms data into code.
* "macro" is a macro function that returns a macro function object. The
first argument must be a cons-cell linked list of identifiers that
makes up the parameter list. The following arguments are then the
statements and return expression of the lambda function. Same
shadowing/free-variable rules as with lambdas apply. Because the
return value is evaluated before returned where it is then typically
evaluated again, it is most often a quasiquote.
This function fails when the parameter list doesn't follow the
required format or when there is no return expression.
* "lambda" is a macro function that returns a lambda function
object. The first argument must be a cons-cell linked list of
identifiers that makes up the parameter list. The following arguments
are then the statements and return expression of the lambda
function. Going through the statements and return expression, any
variables (unquoted atoms/identifiers) that are not in the parameter
list (or the parameter list of any nested "lambda" call) are to be
evaluated and replaced by their value from the current scope. The
references are therefore inlined and we don't capture identifier names
and have a clean scope for each nested call. All function calls to
macro function objects are called here, but the final evaluation of
the returned code is not done, that only occurs when the returned
lambda function object is actually called.
This function fails when the parameter list doesn't follow the
required format or when there is no return expression.
Example:
```
(declare 'sum-many (macro (x y z) (declare 'mid (+ y z))
		  `(+ ,x (+ (+ ,mid ,y) ,z))))
(lambda (x) (sum-many x 1 2))
> lambda-object {
	parameters: [x]
	stmts: []
	ret: (+ x (+ (+ 3 1) 2))
}
```
* "set" is a macro function which takes two arguments, of which the
first must evaluate to a quoted identifier/atom that is already
declared. It updates the object reference the identifier is bound to
to take the value of the second argument once evaluated.
This function fails when the parameter list doesn't follow the
required format or when there is no return expression.
* "declare" is a macro function that takes two arguments, of which the
first must evaluate to a quoted indentifier (though evaluation is
still not done before declare is called). If the identifier is unbound
it adds it the the scope pointing to a new object reference bound to
the value the second argument evaluates to. If the identifier is
already bound it updates the identifier to point to a new object
reference which points to the value of the second argument, leaving
aliasing bindings to the same object reference intact.  This function
fails when the parameter list doesn't follow the required format or
when there is no return expression.
### Builtin values
There are two predefined variables: t and nil. They are both defined
as themselves but quoted.
* "t" is 't.
* "nil" is 'nil.

## Failure
In practice we will just terminate or have the interpreter return an
error when the language causes a failure. It's still unclear how this
is to be formalised.
